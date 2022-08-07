//!
//! Server for handling the pixelflut protocol over TCP connections
//!

use actix::fut::wrap_future;
use actix::prelude::*;

use crate::net::ClientConnectedMsg;
use tokio::net::TcpListener;

use super::{TcpConnection, TcpOptions};
use crate::pixmap::pixmap_actor::PixmapActor;
use crate::pixmap::Pixmap;
use crate::state_encoding::MultiEncodersClient;

/// A TcpServer listens on a certain port using TCP and serves as a Pixelflut server for clients that connect
/// to it.
///
/// ## Startup
/// On Actor startup it spawns an additional future in its context which opens the TCP server socket, and
/// accepts new connections.
/// Additionally, for each new connection an additional future is spawned in the actor context to handle it.
///
/// ## Shutdown
/// On Actor shutdown, the TCP Server socket is closed so that no new connections are accepted.
/// Existing connection however remain intact for as long as the actors context is running.
///
/// This means foremost that existing connections survive an actor restart but not dropping the actor
/// completely.
#[derive(Debug, Clone)]
pub struct TcpServer<P: Pixmap + Unpin + 'static> {
    options: TcpOptions,
    pixmap_addr: Addr<PixmapActor<P>>,
    enc_client: MultiEncodersClient,
}

impl<P: Pixmap + Unpin + 'static> TcpServer<P> {
    /// Create a new TcpServer
    pub fn new(
        options: TcpOptions,
        pixmap_addr: Addr<PixmapActor<P>>,
        enc_client: MultiEncodersClient,
    ) -> Self {
        Self {
            options,
            pixmap_addr,
            enc_client,
        }
    }

    /// Listen on the tcp port defined through *options* while using the given *pixmap* and *encodings*
    /// as backing data storage
    async fn listen(
        self_addr: Addr<Self>,
        options: TcpOptions,
        pixmap_addr: Addr<PixmapActor<P>>,
        enc_client: MultiEncodersClient,
    ) {
        let listener = TcpListener::bind(options.listen_address).await.unwrap();
        log::info!("Started tcp server on {}", listener.local_addr().unwrap());

        loop {
            let (socket, _) = listener.accept().await.unwrap();
            let connection = TcpConnection::new(socket, pixmap_addr.clone(), enc_client.clone());
            self_addr.send(ClientConnectedMsg { connection }).await.unwrap();
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
            self.enc_client.clone(),
        )));
    }
}

impl<P: Pixmap + Unpin + 'static> Supervised for TcpServer<P> {}

impl<P: Pixmap + Unpin + 'static> Handler<ClientConnectedMsg<TcpConnection<P>>> for TcpServer<P> {
    type Result = ();

    fn handle(&mut self, msg: ClientConnectedMsg<TcpConnection<P>>, ctx: &mut Self::Context) -> Self::Result {
        ctx.spawn(wrap_future(msg.connection.handle_connection()));
    }
}
