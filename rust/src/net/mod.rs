//!
//! Networking layer for pixelflut servers and clients as well as on-the-wire protocol handling
//!

use actix::prelude::*;
use std::convert::TryFrom;

use anyhow::Result;
use bytes::{Buf, Bytes};

use crate::net::framing::Frame;
use crate::pixmap::pixmap_actor::{GetPixelMsg, GetSizeMsg, PixmapActor, SetPixelMsg};
use crate::pixmap::Pixmap;
use crate::protocol::{Request, Response};

pub mod framing;
// pub mod udp_server;
pub mod tcp;
pub mod udp;
pub mod ws;

/// handle a request frame and return a response frame
async fn handle_frame<P, B>(input: Frame<B>, pixmap_addr: &Addr<PixmapActor<P>>) -> Option<Frame<Bytes>>
where
    P: Pixmap + Unpin + 'static,
    B: Buf,
{
    // try parse the received frame as request
    match Request::try_from(input) {
        Err(e) => Some(Frame::new_from_string(e.to_string())),
        Ok(request) => match handle_request(request, pixmap_addr).await {
            Err(e) => Some(Frame::new_from_string(e.to_string())),
            Ok(response) => response.map(|r| r.into()),
        },
    }
}

/// handle a request and return a response
async fn handle_request<P>(request: Request, pixmap_addr: &Addr<PixmapActor<P>>) -> Result<Option<Response>>
where
    P: Pixmap + Unpin + 'static,
{
    let pixmap_size = pixmap_addr.send(GetSizeMsg {}).await??;

    match request {
        Request::Size => Ok(Some(Response::Size(pixmap_size.0, pixmap_size.1))),
        Request::Help(topic) => Ok(Some(Response::Help(topic))),
        Request::PxGet(x, y) => Ok(Some(Response::Px(
            x,
            y,
            pixmap_addr.send(GetPixelMsg { x: x, y: y }).await??,
        ))),
        Request::PxSet(x, y, color) => {
            pixmap_addr
                .send(SetPixelMsg {
                    x: x,
                    y: y,
                    color: color,
                })
                .await??;
            Ok(None)
        }
        // Request::State(algorithm) => match algorithm {
        //     StateEncodingAlgorithm::Rgb64 => Ok(Some(Response::State(
        //         algorithm,
        //         encodings.rgb64.lock().unwrap().clone(),
        //     ))),
        //     StateEncodingAlgorithm::Rgba64 => Ok(Some(Response::State(
        //         algorithm,
        //         encodings.rgba64.lock().unwrap().clone(),
        //     ))),
        // },
        _ => todo!("Re-implement encodings"),
    }
}

#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
struct ClientConnectedMsg<C> {
    connection: C,
}
