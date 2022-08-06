use crate::net::framing::Frame;
use crate::net::udp::UdpOptions;
use crate::pixmap::pixmap_actor::PixmapActor;
use crate::pixmap::Pixmap;
use actix::fut::wrap_future;
use actix::prelude::*;
use bytes::{Buf, BytesMut};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

/// A UdpServer accepts datagram packets containing Pixelflut commands and handles them.
///
/// ## Startup
/// On Actor startup it spawns a future in its context which opens the UDP socket and listens for incoming
/// datagrams.
///
/// ## Shutdown
/// On Actor stop, the UDP socket is automatically closed so that no new datagrams can arrive.
#[derive(Debug, Clone)]
pub struct UdpServer<P: Pixmap + Unpin + 'static> {
    options: UdpOptions,
    pixmap_addr: Addr<PixmapActor<P>>,
    server_task: Option<SpawnHandle>,
}

impl<P: Pixmap + Unpin + 'static> UdpServer<P> {
    /// Create a new UDP Server
    pub fn new(options: UdpOptions, pixmap_addr: Addr<PixmapActor<P>>) -> Self {
        Self {
            server_task: None,
            options,
            pixmap_addr,
        }
    }

    /// Listen on the udp port defined through *options* while using the given *pixmap* and *encodings*
    /// as backing data storage
    pub async fn listen(options: UdpOptions, pixmap_addr: Addr<PixmapActor<P>>) -> tokio::io::Result<()> {
        let socket = Arc::new(UdpSocket::bind(options.listen_address).await?);
        info!("Started udp listener on {}", socket.local_addr().unwrap());

        loop {
            let pixmap_addr = pixmap_addr.clone();
            let socket = socket.clone();
            let mut buffer = BytesMut::with_capacity(1024);

            let res = socket.recv_from(&mut buffer[..]).await;
            let (_num_read, origin) = res?;

            UdpServer::process_received(pixmap_addr, buffer, origin, socket).await;
        }
    }

    /// Process a received chunk of data and send the response back to where it came from
    async fn process_received<B: Buf + Clone>(
        pixmap_addr: Addr<PixmapActor<P>>,
        mut buffer: B,
        origin: SocketAddr,
        socket: Arc<UdpSocket>,
    ) {
        // extract frames from received package
        while buffer.has_remaining() {
            match Frame::from_input(buffer.clone()) {
                Err(_) => return,
                Ok((frame, length)) => {
                    buffer.advance(length);

                    // handle the frame
                    match crate::net::handle_frame(frame, &pixmap_addr).await {
                        None => {}
                        Some(response) => {
                            // send back a response
                            match socket
                                .send_to(&response.encode(), origin) // TODO Find a cleaner way to convert frame to &[u8]
                                .await
                            {
                                Err(e) => {
                                    warn!("Error writing frame: {}", e);
                                    return;
                                }
                                Ok(_) => {}
                            }
                        }
                    }
                }
            }
        }
    }
}

impl<P: Pixmap + Unpin + 'static> Actor for UdpServer<P> {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        if self.server_task.is_some() {
            panic!("UdpServer Actor is starting up but already has a handle for the server future");
        }

        // spawn a future on this actors context which opens the udp socket and listens for incoming datagrams
        let do_listen = Self::listen(self.options, self.pixmap_addr.clone());
        let handle = ctx.spawn(wrap_future(async move {
            do_listen.await.unwrap();
            ()
        }));

        self.server_task = Some(handle);
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        match self.server_task {
            None => panic!("UdpServer Actor is stopping but has no handle for the server future"),
            Some(handle) => ctx.cancel_future(handle),
        };
    }
}
