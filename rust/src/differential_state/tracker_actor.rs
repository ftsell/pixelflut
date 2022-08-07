use super::Tracker;
use actix::prelude::*;
use anyhow::Result;
use pixelflut::pixmap::pixmap_actor::SetPixelMsg;

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

impl Handler<SetPixelMsg> for TrackerActor {
    type Result = Result<()>;

    fn handle(&mut self, msg: SetPixelMsg, _ctx: &mut Self::Context) -> Self::Result {
        self.tracker.add(msg.x, msg.y, msg.color);
        Ok(())
    }
}
