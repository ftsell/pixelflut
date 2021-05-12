use crate::net::framing::Frame;
use crate::pixmap::{Pixmap, SharedPixmap};
use crate::state_encoding::SharedMultiEncodings;
use anyhow::Error;
use bytes::buf::Take;
use bytes::{Buf, BytesMut};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

static LOG_TARGET: &str = "pixelflut.net.tcp";

pub struct TcpOptions {
    pub listen_address: SocketAddr,
}

pub async fn listen<P>(pixmap: SharedPixmap<P>, encodings: SharedMultiEncodings, options: TcpOptions)
where
    P: Pixmap + Send + Sync + 'static,
{
    let listener = TcpListener::bind(options.listen_address).await.unwrap();
    info!(
        target: LOG_TARGET,
        "Started tcp listener on {}",
        listener.local_addr().unwrap()
    );

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let pixmap = pixmap.clone();
        let encodings = encodings.clone();
        tokio::spawn(async move {
            process_connection(TcpConnection::new(socket), pixmap, encodings).await;
        });
    }
}

async fn process_connection<P>(
    mut connection: TcpConnection,
    pixmap: SharedPixmap<P>,
    encodings: SharedMultiEncodings,
) where
    P: Pixmap,
{
    debug!(
        target: LOG_TARGET,
        "Client connected {}",
        connection.stream.peer_addr().unwrap()
    );
    loop {
        // receive a frame from the client
        match connection.read_frame().await {
            Err(e) => {
                warn!(target: LOG_TARGET, "Error reading frame: {}", e);
                return;
            }
            Ok(frame) => {
                // handle the frame
                match super::handle_frame(frame, &pixmap, &encodings) {
                    None => {}
                    Some(response) => {
                        // send back a response
                        match connection.write_frame(response).await {
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

pub(crate) struct TcpConnection {
    stream: TcpStream,
    read_buffer: BytesMut,
}

impl TcpConnection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            read_buffer: BytesMut::with_capacity(256),
            stream,
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
