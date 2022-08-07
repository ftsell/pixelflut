use crate::net::framing::Frame;
use crate::pixmap::pixmap_actor::PixmapActor;
use crate::pixmap::Pixmap;
use crate::state_encoding::MultiEncodersClient;
use actix::prelude::*;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};

pub(crate) struct WsConnection<P: Pixmap + Unpin + 'static> {
    uninit_connection: Option<TcpStream>,
    pixmap_addr: Addr<PixmapActor<P>>,
    enc_client: MultiEncodersClient,
}

impl<P: Pixmap + Unpin + 'static> WsConnection<P> {
    pub fn new(
        connection: TcpStream,
        pixmap_addr: Addr<PixmapActor<P>>,
        enc_client: MultiEncodersClient,
    ) -> Self {
        Self {
            uninit_connection: Some(connection),
            pixmap_addr,
            enc_client,
        }
    }

    pub async fn handle_connection(mut self) {
        let websocket = match self.uninit_connection.take() {
            None => panic!("Websocket Connection could not be set up because there is no underlying tcp connection present"),
            Some(tcp_connection) => {
                debug!("Client connected {}", tcp_connection.peer_addr().unwrap());
                tokio_tungstenite::accept_async(tcp_connection).await.unwrap()
            }
        };

        let (mut write, mut read) = websocket.split();

        while let Some(msg) = read.next().await {
            let response = self.process_received(msg).await.unwrap();
            write.send(response).await.unwrap();
        }
    }

    async fn process_received(&self, msg: Result<Message, WsError>) -> Result<Message, WsError>
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
                    match crate::net::handle_frame(frame, &self.pixmap_addr, &self.enc_client).await {
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
