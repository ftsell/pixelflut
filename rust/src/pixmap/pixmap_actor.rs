use crate::pixmap::{Color, Pixmap};
use actix::prelude::*;
use anyhow::Result;

/// An actor that manages a [`Pixmap`] and synchronizes access to it
pub struct PixmapActor<P: Pixmap> {
    pixmap: P,
}

impl<P: Pixmap + Default> Default for PixmapActor<P> {
    fn default() -> Self {
        Self { pixmap: P::default() }
    }
}

impl<P: Pixmap + Unpin + 'static> Actor for PixmapActor<P> {
    type Context = Context<Self>;
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

#[derive(Debug, Copy, Clone, Message)]
#[rtype(result = "Result<Color>")]
pub struct GetPixelMsg {
    x: usize,
    y: usize,
}

#[derive(Debug, Copy, Clone, Message)]
#[rtype(result = "Result<()>")]
pub struct SetPixelMsg {
    x: usize,
    y: usize,
    color: Color,
}

#[derive(Debug, Copy, Clone, Message)]
#[rtype(result = "Result<(usize, usize)>")]
pub struct GetSizeMsg {}

#[derive(Debug, Copy, Clone, Message)]
#[rtype(result = "Result<Vec<Color>>")]
pub struct GetRawDataMsg {}

#[derive(Debug, Clone, Message)]
#[rtype(result = "Result<()>")]
pub struct PutRawDataMsg {
    data: Vec<Color>,
}
