//! Implementation of an actor that hosts a pixmap as well as Message definitions

use crate::pixmap::{Color, Pixmap};
use actix::prelude::*;
use anyhow::Result;
use std::any::type_name;

/// An actor that manages a [`Pixmap`] and synchronizes access to it
#[derive(Debug, Clone)]
pub struct PixmapActor<P: Pixmap> {
    pixmap: P,
}

impl<P: Pixmap> PixmapActor<P> {
    /// Create a new PixmapActor that is backed by the given pixmap
    pub fn new(pixmap: P) -> Self {
        Self { pixmap }
    }
}

impl<P: Pixmap + Default> Default for PixmapActor<P> {
    fn default() -> Self {
        Self { pixmap: P::default() }
    }
}

impl<P: Pixmap + Unpin + 'static> Actor for PixmapActor<P> {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        log::debug!("Started PixmapActor<{}>", type_name::<P>())
    }
}

impl<P: Pixmap + Unpin + 'static> Supervised for PixmapActor<P> {}

impl<P: Pixmap + Unpin + 'static> Handler<GetPixelMsg> for PixmapActor<P> {
    type Result = Result<Color>;

    fn handle(&mut self, msg: GetPixelMsg, _ctx: &mut Self::Context) -> Self::Result {
        self.pixmap.get_pixel(msg.x, msg.y)
    }
}

impl<P: Pixmap + Unpin + 'static> Handler<SetPixelMsg> for PixmapActor<P> {
    type Result = Result<()>;

    fn handle(&mut self, msg: SetPixelMsg, _ctx: &mut Self::Context) -> Self::Result {
        self.pixmap.set_pixel(msg.x, msg.y, msg.color)
    }
}

impl<P: Pixmap + Unpin + 'static> Handler<GetSizeMsg> for PixmapActor<P> {
    type Result = Result<(usize, usize)>;

    fn handle(&mut self, _msg: GetSizeMsg, _ctx: &mut Self::Context) -> Self::Result {
        self.pixmap.get_size()
    }
}

impl<P: Pixmap + Unpin + 'static> Handler<GetRawDataMsg> for PixmapActor<P> {
    type Result = Result<Vec<Color>>;

    fn handle(&mut self, _msg: GetRawDataMsg, _ctx: &mut Self::Context) -> Self::Result {
        self.pixmap.get_raw_data()
    }
}

impl<P: Pixmap + Unpin + 'static> Handler<PutRawDataMsg> for PixmapActor<P> {
    type Result = Result<()>;

    fn handle(&mut self, msg: PutRawDataMsg, _ctx: &mut Self::Context) -> Self::Result {
        self.pixmap.put_raw_data(&msg.data)
    }
}

/// A message to query a certain pixel from the pixmap
#[derive(Debug, Copy, Clone, Message)]
#[rtype(result = "Result<Color>")]
pub struct GetPixelMsg {
    /// X coordinate of the queried pixel
    pub x: usize,
    /// Y coordinate of the queried pixel
    pub y: usize,
}

/// A message to set a certain pixel to a certain color
#[derive(Debug, Copy, Clone, Message)]
#[rtype(result = "Result<()>")]
pub struct SetPixelMsg {
    /// X coordinate of the target pixel
    pub x: usize,
    /// Y coordinate of the target pixel
    pub y: usize,
    /// Color which the target pixel should be set to
    pub color: Color,
}

/// A message to query the size of the pixmap as a  *width, height* tuple
#[derive(Debug, Copy, Clone, Message)]
#[rtype(result = "Result<(usize, usize)>")]
pub struct GetSizeMsg {}

/// A message to query the completely dumped color data of a pixmap
#[derive(Debug, Copy, Clone, Message)]
#[rtype(result = "Result<Vec<Color>>")]
pub struct GetRawDataMsg {}

/// A message to overwrite the complete color data of a pixmap
#[derive(Debug, Clone, Message)]
#[rtype(result = "Result<()>")]
pub struct PutRawDataMsg {
    /// The color data with which the pixmap should be overwritten
    pub data: Vec<Color>,
}
