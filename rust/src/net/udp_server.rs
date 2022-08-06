//! Server for handling the pixelflut protocol over connectionless UDP datagrams

use actix::fut::wrap_future;
use actix::{Actor, Addr, AsyncContext, Context};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use bytes::{Buf, BytesMut};
use tokio::net::UdpSocket;

use crate::net::framing::Frame;
use crate::pixmap::pixmap_actor::PixmapActor;
use crate::pixmap::Pixmap;

static LOG_TARGET: &str = "pixelflut.net.udp";

/// Options which can be given to [`listen`] for detailed configuration
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct UdpOptions {
    /// On which address the server should listen
    pub listen_address: SocketAddr,
}

impl Default for UdpOptions {
    fn default() -> Self {
        Self {
            listen_address: SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 1234),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UdpServer<P: Pixmap + Unpin + 'static> {
    options: UdpOptions,
    pixmap_addr: Addr<PixmapActor<P>>,
}

impl<P: Pixmap + Unpin + 'static> UdpServer<P> {
    pub fn new(options: UdpOptions, pixmap_addr: Addr<PixmapActor<P>>) -> Self {
        Self { options, pixmap_addr }
    }

    /// Listen on the udp port defined through *options* while using the given *pixmap* and *encodings*
    /// as backing data storage
    pub async fn listen(&self, ctx: &mut <Self as Actor>::Context) -> tokio::io::Result<()> {
        let socket = Arc::new(UdpSocket::bind(self.options.listen_address).await?);
        info!("Started udp listener on {}", socket.local_addr().unwrap());

        loop {
            let pixmap_addr = self.pixmap_addr.clone();
            let socket = socket.clone();
            let mut buffer = BytesMut::with_capacity(1024);

            let res = socket.recv_from(&mut buffer[..]).await;
            let (_num_read, origin) = res?;

            ctx.spawn(wrap_future(async move {
                UdpServer::process_received(pixmap_addr, buffer, origin, socket).await;
            }));
        }
    }

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
                    match super::handle_frame(frame, &pixmap_addr).await {
                        None => {}
                        Some(response) => {
                            // send back a response
                            match socket
                                .send_to(&response.encode(), origin) // TODO Find a cleaner way to convert frame to &[u8]
                                .await
                            {
                                Err(e) => {
                                    warn!(target: LOG_TARGET, "Error writing frame: {}", e);
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
        ctx.spawn(wrap_future(async {
            self.listen(ctx).await.unwrap();
            ()
        }));
    }
}
