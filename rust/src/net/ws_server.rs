//!
//! Server for handling the pixelflut protocol over websocket connections
//!
//! This implementation is currently fairly basic and only really intended to be used by [pixelflut-js](https://github.com/ftsell/pixelflut-js)
//!

use actix::fut::wrap_future;
use actix::prelude::*;
use futures_util::SinkExt;
use std::convert::TryInto;
use std::net::{Ipv4Addr, SocketAddr};

use crate::actor_util::StopActorMsg;
use crate::net::ClientConnectedMsg;
use futures_util::stream::StreamExt;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Error as WsError;
use tokio_tungstenite::tungstenite::Message;

use crate::net::framing::Frame;
use crate::pixmap::pixmap_actor::PixmapActor;
use crate::pixmap::Pixmap;

/// Options which can be given to [`listen`] for detailed configuration
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct WsOptions {
    /// On which address the server should listen
    pub listen_address: SocketAddr,
}

impl Default for WsOptions {
    fn default() -> Self {
        Self {
            listen_address: SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 1234),
        }
    }
}

/// A WebSocket server accepts incoming connections, upgrades the protocol to WebSocket and then handles
/// Pixelflut messages that are transmitted via the WebSocket connection
#[derive(Debug, Clone)]
pub struct WsServer<P: Pixmap + Unpin + 'static> {
    options: WsOptions,
    pixmap_addr: Addr<PixmapActor<P>>,
    clients: Vec<Addr<WsConnectionHandler<P>>>,
}

impl<P: Pixmap + Unpin + 'static> WsServer<P> {
    /// Create a new WebSocket server with the given parameters
    pub fn new(options: WsOptions, pixmap_addr: Addr<PixmapActor<P>>) -> Self {
        Self {
            options,
            pixmap_addr,
            clients: Vec::new(),
        }
    }

    /// Listen on the tpc port defined through *options* while using the given *pixmap* and *encodings*
    /// as backing data storage
    pub async fn listen(self_addr: Addr<Self>, options: WsOptions, pixmap_addr: Addr<PixmapActor<P>>) {
        let listener = TcpListener::bind(options.listen_address).await.unwrap();
        info!("Started websocket listener on {}", listener.local_addr().unwrap());

        loop {
            let res = listener.accept().await;
            let (socket, _) = res.unwrap();

            let handler_addr = WsConnectionHandler::new(socket, pixmap_addr.clone()).start();
            self_addr.send(ClientConnectedMsg { handler_addr }).await.unwrap();
        }
    }
}

impl<P: Pixmap + Unpin + 'static> Actor for WsServer<P> {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.spawn(wrap_future(Self::listen(
            ctx.address(),
            self.options,
            self.pixmap_addr.clone(),
        )));
    }
}

impl<P: Pixmap + Unpin + 'static> Supervised for WsServer<P> {}

impl<P: Pixmap + Unpin + 'static> Handler<ClientConnectedMsg<WsConnectionHandler<P>>> for WsServer<P> {
    type Result = ();

    fn handle(
        &mut self,
        msg: ClientConnectedMsg<WsConnectionHandler<P>>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.clients.push(msg.handler_addr);
    }
}

pub(crate) struct WsConnectionHandler<P: Pixmap + Unpin + 'static> {
    uninit_connection: Option<TcpStream>,
    pixmap_addr: Addr<PixmapActor<P>>,
}

impl<P: Pixmap + Unpin + 'static> WsConnectionHandler<P> {
    fn new(connection: TcpStream, pixmap_addr: Addr<PixmapActor<P>>) -> Self {
        Self {
            uninit_connection: Some(connection),
            pixmap_addr,
        }
    }

    async fn handle_connection(pixmap_addr: Addr<PixmapActor<P>>, tcp_connection: TcpStream) {
        debug!("Client connected {}", tcp_connection.peer_addr().unwrap());

        let websocket = tokio_tungstenite::accept_async(tcp_connection).await.unwrap();
        let (mut write, mut read) = websocket.split();

        while let Some(msg) = read.next().await {
            let response = Self::process_received(&pixmap_addr, msg).await.unwrap();
            write.send(response).await.unwrap();
        }
    }

    async fn process_received(
        pixmap_addr: &Addr<PixmapActor<P>>,
        msg: Result<Message, WsError>,
    ) -> Result<Message, WsError>
    where
        P: Pixmap,
    {
        match msg {
            Ok(msg) => match msg {
                Message::Text(msg) => {
                    debug!("Received websocket message: {}", msg);

                    // TODO improve websocket frame handling
                    let frame = Frame::new_from_string(msg);

                    // TODO improve by not sending empty responses
                    match super::handle_frame(frame, &pixmap_addr).await {
                        None => Ok(Message::Text(String::new())),
                        Some(response) => Ok(Message::Text(response.try_into().unwrap())),
                    }
                }
                _ => {
                    warn!("Could not handle websocket message: {}", msg);
                    Ok(Message::text(String::new()))
                }
            },
            Err(e) => {
                warn!("Websocket error: {}", e);
                Ok(Message::Text(String::new()))
            }
        }
    }
}

impl<P: Pixmap + Unpin + 'static> Actor for WsConnectionHandler<P> {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        if let Some(tcp_connection) = self.uninit_connection.take() {
            ctx.spawn(wrap_future(Self::handle_connection(
                self.pixmap_addr.clone(),
                tcp_connection,
            )));
        }
    }
}

impl<P: Pixmap + Unpin + 'static> Handler<StopActorMsg> for WsConnectionHandler<P> {
    type Result = ();

    fn handle(&mut self, _msg: StopActorMsg, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop()
    }
}
