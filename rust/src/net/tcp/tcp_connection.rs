use crate::net::framing::Frame;
use crate::pixmap::pixmap_actor::PixmapActor;
use crate::pixmap::Pixmap;
use actix::prelude::*;
use anyhow::Error;
use bytes::buf::Take;
use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub(crate) struct TcpConnection<P: Pixmap + Unpin + 'static> {
    stream: TcpStream,
    pixmap_addr: Addr<PixmapActor<P>>,
    read_buffer: BytesMut,
}

impl<P: Pixmap + Unpin + 'static> TcpConnection<P> {
    pub fn new(stream: TcpStream, pixmap_addr: Addr<PixmapActor<P>>) -> Self {
        Self {
            stream,
            pixmap_addr,
            read_buffer: BytesMut::with_capacity(256),
        }
    }

    /// Handle this connection by waiting for requests, processing them and sending back responses
    pub(crate) async fn handle_connection(mut self) {
        debug!("Client connected {}", self.stream.peer_addr().unwrap());

        loop {
            // receive a frame from the client
            let frame = self.read_frame().await;
            match frame {
                Err(e) => {
                    warn!("Error reading frame: {}", e);
                    return;
                }
                Ok(frame) => {
                    // handle the frame
                    match crate::net::handle_frame(frame, &self.pixmap_addr).await {
                        None => {}
                        Some(response) => {
                            // send back a response
                            match self.write_frame(response).await {
                                Ok(_) => {}
                                Err(e) => {
                                    warn!("Error writing frame: {}", e)
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Read a single frame from the TCP stream and advance the internal buffer past that frame
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

    /// Write a single frame to the TCP stream
    pub(self) async fn write_frame(&mut self, frame: Frame<impl Buf>) -> std::io::Result<()> {
        self.stream.write_buf(&mut frame.encode()).await?;
        Ok(())
    }
}
