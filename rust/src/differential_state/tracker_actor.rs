use crate::differential_state::Tracker;
use actix::prelude::*;
use pixelflut::pixmap::Color;

pub struct TrackerActor {
    tracker: Tracker,
}

impl TrackerActor {
    pub fn new(pixmap_width: usize, pixmap_height: usize) -> Self {
        Self {
            tracker: Tracker::new(pixmap_width, pixmap_height),
        }
    }
}

impl Actor for TrackerActor {
    type Context = Context<Self>;
}

impl Supervised for TrackerActor {
    fn restarting(&mut self, _ctx: &mut <Self as Actor>::Context) {
        self.tracker.clear();
    }
}

impl Handler<PixelChanged> for TrackerActor {
    type Result = ();

    fn handle(&mut self, msg: PixelChanged, _ctx: &mut Self::Context) -> Self::Result {
        self.tracker.add(msg.x, msg.y, msg.color);
        ()
    }
}

#[derive(Debug, Copy, Clone, Message)]
#[rtype(result = "()")]
struct PixelChanged {
    x: usize,
    y: usize,
    color: Color,
}
