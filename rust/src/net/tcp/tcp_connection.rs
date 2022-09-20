use crate::differential_state::{SubscribeMsg, TrackerActor};
use crate::net::framing::Frame;
use crate::net::ConnectionPreferences;
use crate::pixmap::pixmap_actor::{PixmapActor, SetPixelMsg};
use crate::pixmap::Pixmap;
use crate::protocol::Response;
use crate::state_encoding::MultiEncodersClient;
use actix::prelude::*;
use anyhow::Error;
use bytes::buf::Take;
use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::watch;
use tokio::sync::watch::Ref;

pub(crate) struct TcpConnection<P: Pixmap + Unpin + 'static> {
    pixmap_addr: Addr<PixmapActor<P>>,
    tracker_addr: Addr<TrackerActor>,
    enc_client: MultiEncodersClient,
    stream: TcpStream,
    read_buffer: BytesMut,
    client_preferences: ConnectionPreferences,
    data_subscription: Option<watch::Receiver<Vec<SetPixelMsg>>>,
}

impl<P: Pixmap + Unpin + 'static> TcpConnection<P> {
    pub fn new(
        stream: TcpStream,
        pixmap_addr: Addr<PixmapActor<P>>,
        enc_client: MultiEncodersClient,
        tracker_addr: Addr<TrackerActor>,
    ) -> Self {
        Self {
            stream,
            pixmap_addr,
            tracker_addr,
            enc_client,
            read_buffer: BytesMut::with_capacity(256),
            client_preferences: ConnectionPreferences::default(),
            data_subscription: None,
        }
    }

    /// Handle this connection by waiting for requests, processing them and sending back responses
    pub(crate) async fn handle_connection(mut self) {
        let peer_addr = self.stream.peer_addr().unwrap();
        debug!("Client connected {}", peer_addr);

        loop {
            if !match self.data_subscription.take() {
                None => {
                    let frame = self.read_frame().await;
                    self.handle_request_frame(frame).await
                }
                Some(mut data_subscription) => {
                    let result = tokio::select! {
                        // receive a frame from the client
                        frame = self.read_frame() => self.handle_request_frame(frame).await,
                        _ = data_subscription.changed() => self.handle_new_subscription_data(data_subscription.borrow()).await
                    };
                    self.data_subscription = Some(data_subscription);
                    result
                }
            } {
                log::debug!("Closing client connection {}", peer_addr);
                break;
            }
        }
    }

    /// Handle a received frame by processing it and sending back an appropriate response
    ///
    /// Returns true if more requests should be processed and false when the connection is considered closed
    async fn handle_request_frame(&mut self, frame: std::io::Result<Frame<impl Buf>>) -> bool {
        // handle the request and send back a response
        match frame {
            Err(e) => {
                warn!("Error reading frame: {}", e);
                return false;
            }
            Ok(frame) => {
                // handle the frame
                match crate::net::handle_frame(
                    frame,
                    &self.pixmap_addr,
                    &self.enc_client,
                    &mut self.client_preferences,
                )
                .await
                {
                    None => {}
                    Some(response) => {
                        // send back a response
                        match self.write_frame(response).await {
                            Ok(_) => {}
                            Err(e) => {
                                warn!("Error writing frame: {}", e);
                                return false;
                            }
                        }
                    }
                }
            }
        }

        // subscribe to updates if necessary
        if self.client_preferences.subscribed && self.data_subscription.is_none() {
            let subscription = self.tracker_addr.send(SubscribeMsg {}).await.unwrap();
            self.data_subscription = Some(subscription);
            self.write_frame(Response::SubscriptionActivated.into())
                .await
                .unwrap();
        }

        // unsubscribe from updates if necessary
        if !self.client_preferences.subscribed && self.data_subscription.is_some() {
            self.data_subscription = None;
            self.write_frame(Response::SubscriptionDeactivated.into())
                .await
                .unwrap();
        }

        true
    }

    /// Handle new subscription data being available by sending them over to the client
    ///
    /// Returns true if more requests should be processed and false when the connection is considered closed
    async fn handle_new_subscription_data(&mut self, data: Ref<'_, Vec<SetPixelMsg>>) -> bool {
        for i_update in data.iter() {
            self.write_frame(Response::Px(i_update.x, i_update.y, i_update.color).into())
                .await
                .unwrap();
        }

        true
    }

    /// Read a single frame from the TCP stream and advance the internal buffer past that frame
    async fn read_frame(&mut self) -> std::io::Result<Frame<Take<BytesMut>>> {
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

    /// Write a single frame to the TCP stream
    async fn write_frame(&mut self, frame: Frame<impl Buf>) -> std::io::Result<()> {
        self.stream.write_buf(&mut frame.encode()).await?;
        Ok(())
    }
}
