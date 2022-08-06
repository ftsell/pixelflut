//!
//! Server for handling the pixelflut protocol over TCP connections
//!

use actix::fut::wrap_future;
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Supervised};
use std::net::{Ipv4Addr, SocketAddr};

use crate::actor_util::StopActorMsg;
use crate::net::ClientConnectedMsg;
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

/// A TcpServer listens on a certain port using TCP and serves as a Pixelflut server for clients that connect
/// to it.
#[derive(Debug, Clone)]
pub struct TcpServer<P: Pixmap + Unpin + 'static> {
    options: TcpOptions,
    pixmap_addr: Addr<PixmapActor<P>>,
    clients: Vec<Addr<TcpConnectionHandler<P>>>,
}

impl<P: Pixmap + Unpin + 'static> TcpServer<P> {
    /// Create a new TcpServer
    pub fn new(options: TcpOptions, pixmap_addr: Addr<PixmapActor<P>>) -> Self {
        Self {
            options,
            pixmap_addr,
            clients: Vec::new(),
        }
    }

    /// Listen on the tcp port defined through *options* while using the given *pixmap* and *encodings*
    /// as backing data storage
    async fn listen(self_addr: Addr<Self>, options: TcpOptions, pixmap_addr: Addr<PixmapActor<P>>) {
        let listener = TcpListener::bind(options.listen_address).await.unwrap();
        log::info!("Started tcp server on {}", listener.local_addr().unwrap());

        loop {
            let res = listener.accept().await;
            let (socket, _) = res.unwrap();

            // let encodings = encodings.clone();

            let handler_addr = TcpConnectionHandler::new(socket, pixmap_addr.clone()).start();
            self_addr.send(ClientConnectedMsg { handler_addr }).await.unwrap();
        }
    }
}

impl<P: Pixmap + Unpin + 'static> Actor for TcpServer<P> {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.spawn(wrap_future(Self::listen(
            ctx.address(),
            self.options,
            self.pixmap_addr.clone(),
        )));
    }
}

impl<P: Pixmap + Unpin + 'static> Supervised for TcpServer<P> {}

impl<P: Pixmap + Unpin + 'static> Handler<ClientConnectedMsg<TcpConnectionHandler<P>>> for TcpServer<P> {
    type Result = ();

    fn handle(
        &mut self,
        msg: ClientConnectedMsg<TcpConnectionHandler<P>>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.clients.push(msg.handler_addr)
    }
}

pub(crate) struct TcpConnectionHandler<P: Pixmap + Unpin + 'static> {
    stream: Option<TcpStream>,
    pixmap_addr: Addr<PixmapActor<P>>,
}

impl<P: Pixmap + Unpin + 'static> TcpConnectionHandler<P> {
    pub fn new(stream: TcpStream, pixmap_addr: Addr<PixmapActor<P>>) -> Self {
        Self {
            stream: Some(stream),
            pixmap_addr,
        }
    }

    async fn handle_connection(mut stream: TcpStream, pixmap_addr: Addr<PixmapActor<P>>) {
        debug!("Client connected {}", stream.peer_addr().unwrap());

        let mut read_buffer = BytesMut::with_capacity(256);
        loop {
            // receive a frame from the client
            let frame = Self::read_frame(&mut stream, &mut read_buffer).await;
            match frame {
                Err(e) => {
                    warn!(target: LOG_TARGET, "Error reading frame: {}", e);
                    return;
                }
                Ok(frame) => {
                    // handle the frame
                    match super::handle_frame(frame, &pixmap_addr).await {
                        None => {}
                        Some(response) => {
                            // send back a response
                            match Self::write_frame(&mut stream, response).await {
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

    pub(self) async fn read_frame(
        stream: &mut TcpStream,
        read_buffer: &mut BytesMut,
    ) -> std::io::Result<Frame<Take<BytesMut>>> {
        loop {
            match Frame::from_input(read_buffer.clone()) {
                Ok((frame, length)) => {
                    // discard the frame from the buffer
                    read_buffer.advance(length);
                    return Ok(frame);
                }
                Err(_) => {
                    let n = stream.read_buf(read_buffer).await?;
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

    pub(self) async fn write_frame<B>(stream: &mut TcpStream, frame: Frame<B>) -> std::io::Result<()>
    where
        B: Buf,
    {
        stream.write_buf(&mut frame.encode()).await?;
        Ok(())
    }
}

impl<P: Pixmap + Unpin + 'static> Actor for TcpConnectionHandler<P> {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        if let Some(stream) = self.stream.take() {
            ctx.spawn(wrap_future(Self::handle_connection(
                stream,
                self.pixmap_addr.clone(),
            )));
        }
    }
}

impl<P: Pixmap + Unpin + 'static> Handler<StopActorMsg> for TcpConnectionHandler<P> {
    type Result = ();

    fn handle(&mut self, _msg: StopActorMsg, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop()
    }
}
