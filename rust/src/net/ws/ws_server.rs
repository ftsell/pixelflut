//!
//! Server for handling the pixelflut protocol over websocket connections
//!
//! This implementation is currently fairly basic and only really only intended to be used by [pixelflut-js](https://github.com/ftsell/pixelflut-js)
//!

use actix::fut::wrap_future;
use actix::prelude::*;

use crate::net::ClientConnectedMsg;
use tokio::net::TcpListener;

use super::{WsConnection, WsOptions};
use crate::pixmap::pixmap_actor::PixmapActor;
use crate::pixmap::Pixmap;
use crate::state_encoding::MultiEncodersClient;

/// A WebSocket server accepts incoming connections, upgrades the protocol to WebSocket and then handles
/// Pixelflut messages that are transmitted via the WebSocket connection
#[derive(Debug, Clone)]
pub struct WsServer<P: Pixmap + Unpin + 'static> {
    options: WsOptions,
    pixmap_addr: Addr<PixmapActor<P>>,
    enc_client: MultiEncodersClient,
}

impl<P: Pixmap + Unpin + 'static> WsServer<P> {
    /// Create a new WebSocket server with the given parameters
    pub fn new(
        options: WsOptions,
        pixmap_addr: Addr<PixmapActor<P>>,
        enc_client: MultiEncodersClient,
    ) -> Self {
        Self {
            options,
            pixmap_addr,
            enc_client,
        }
    }

    /// Listen on the tpc port defined through *options* while using the given *pixmap* and *encodings*
    /// as backing data storage
    pub async fn listen(
        self_addr: Addr<Self>,
        options: WsOptions,
        pixmap_addr: Addr<PixmapActor<P>>,
        enc_client: MultiEncodersClient,
    ) {
        let listener = TcpListener::bind(options.listen_address).await.unwrap();
        info!("Started websocket listener on {}", listener.local_addr().unwrap());

        loop {
            let res = listener.accept().await;
            let (socket, _) = res.unwrap();

            let connection = WsConnection::new(socket, pixmap_addr.clone(), enc_client.clone());
            self_addr.send(ClientConnectedMsg { connection }).await.unwrap();
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
            self.enc_client.clone(),
        )));
    }
}

impl<P: Pixmap + Unpin + 'static> Supervised for WsServer<P> {}

impl<P: Pixmap + Unpin + 'static> Handler<ClientConnectedMsg<WsConnection<P>>> for WsServer<P> {
    type Result = ();

    fn handle(&mut self, msg: ClientConnectedMsg<WsConnection<P>>, ctx: &mut Self::Context) -> Self::Result {
        ctx.spawn(wrap_future(msg.connection.handle_connection()));
    }
}
