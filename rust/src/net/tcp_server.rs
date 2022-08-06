//!
//! Server for handling the pixelflut protocol over TCP connections
//!

use actix::fut::wrap_future;
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Supervised};
use std::net::{Ipv4Addr, SocketAddr};

use crate::actor_util::StopActorMsg;
use anyhow::Error;
use bytes::buf::Take;
use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::net::framing::Frame;
use crate::pixmap::pixmap_actor::PixmapActor;
use crate::pixmap::Pixmap;

static LOG_TARGET: &str = "pixelflut.net.tcp";

/// Options which can be given to [`listen`] for detailed configuration
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TcpOptions {
    /// On which address the server should listen
    pub listen_address: SocketAddr,
}

impl Default for TcpOptions {
    fn default() -> Self {
        Self {
            listen_address: SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 1234),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TcpServer<P: Pixmap + Unpin + 'static> {
    options: TcpOptions,
    pixmap_addr: Addr<PixmapActor<P>>,
    clients: Vec<Addr<TcpConnectionHandler<P>>>,
}

impl<P: Pixmap + Unpin + 'static> TcpServer<P> {
    pub fn new(options: TcpOptions, pixmap_addr: Addr<PixmapActor<P>>) -> Self {
        Self {
            options,
            pixmap_addr,
            clients: Vec::new(),
        }
    }

    /// Listen on the tcp port defined through *options* while using the given *pixmap* and *encodings*
    /// as backing data storage
    async fn listen(
        &mut self,
        // encodings: SharedMultiEncodings,
    ) -> tokio::io::Result<()> {
        let listener = TcpListener::bind(self.options.listen_address).await?;
        log::info!("Started tcp server on {}", listener.local_addr().unwrap());

        loop {
            let res = listener.accept().await;
            let (socket, _) = res?;

            // let encodings = encodings.clone();

            let handler_addr = TcpConnectionHandler::new(socket, self.pixmap_addr.clone()).start();
            self.clients.push(handler_addr);
        }
    }
}

impl<P: Pixmap + Unpin + 'static> Actor for TcpServer<P> {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.spawn(wrap_future(async {
            TcpServer::listen(self).await.unwrap();
            ()
        }));
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        for client in self.clients {
            client.send(StopActorMsg {});
        }
    }
}

impl<P: Pixmap + Unpin + 'static> Supervised for TcpServer<P> {}

pub(crate) struct TcpConnectionHandler<P: Pixmap + Unpin + 'static> {
    stream: TcpStream,
    read_buffer: BytesMut,
    pixmap_addr: Addr<PixmapActor<P>>,
}

impl<P: Pixmap + Unpin + 'static> TcpConnectionHandler<P> {
    pub fn new(stream: TcpStream, pixmap_addr: Addr<PixmapActor<P>>) -> Self {
        Self {
            read_buffer: BytesMut::with_capacity(256),
            stream,
            pixmap_addr,
        }
    }

    async fn handle_connection(&mut self) {
        debug!("Client connected {}", self.stream.peer_addr().unwrap());
        loop {
            // receive a frame from the client
            let frame = self.read_frame().await;
            match frame {
                Err(e) => {
                    warn!(target: LOG_TARGET, "Error reading frame: {}", e);
                    return;
                }
                Ok(frame) => {
                    // handle the frame
                    match super::handle_frame(frame, &self.pixmap_addr).await {
                        None => {}
                        Some(response) => {
                            // send back a response
                            match self.write_frame(response).await {
                                Ok(_) => {}
                                Err(e) => {
                                    warn!(target: LOG_TARGET, "Error writing frame: {}", e)
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub(self) async fn read_frame(&mut self) -> std::io::Result<Frame<Take<BytesMut>>> {
        loop {
            match Frame::from_input(self.read_buffer.clone()) {
                Ok((frame, length)) => {
                    // discard the frame from the buffer
                    self.read_buffer.advance(length);
                    return Ok(frame);
                }
                Err(_) => {
                    let n = self.stream.read_buf(&mut self.read_buffer).await?;
                    if n == 0 {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof,
                            Error::msg("eof while reading frame"),
                        ));
                    }
                }
            }
        }
    }

    pub(self) async fn write_frame<B>(&mut self, frame: Frame<B>) -> std::io::Result<()>
    where
        B: Buf,
    {
        self.stream.write_buf(&mut frame.encode()).await?;
        Ok(())
    }
}

impl<P: Pixmap + Unpin + 'static> Actor for TcpConnectionHandler<P> {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.spawn(wrap_future(async { self.handle_connection().await }));
    }
}

impl<P: Pixmap + Unpin + 'static> Handler<StopActorMsg> for TcpConnectionHandler<P> {
    type Result = ();

    fn handle(&mut self, _msg: StopActorMsg, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop()
    }
}
