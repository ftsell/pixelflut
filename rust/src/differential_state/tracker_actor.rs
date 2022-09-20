use super::Tracker;
use crate::pixmap::pixmap_actor::SetPixelMsg;
use actix::dev::{MessageResponse, OneshotSender};
use actix::fut::wrap_future;
use actix::prelude::*;
use anyhow::Result;
use std::time::Duration;
use tokio::sync::watch;

/// Actor wrapper for [`Tracker`]s
#[derive(Debug)]
pub struct TrackerActor {
    tracker: Tracker,
    trigger_period: Duration,
    trigger_task: Option<SpawnHandle>,
    publisher: watch::Sender<Vec<SetPixelMsg>>,
}

impl TrackerActor {
    /// Crate a new TrackerActor that can track changes of pixmaps of the given dimensions
    pub fn new(pixmap_width: usize, pixmap_height: usize) -> Self {
        Self {
            tracker: Tracker::new(pixmap_width, pixmap_height),
            trigger_task: None,
            publisher: watch::channel(Vec::new()).0,
            trigger_period: Duration::from_millis(100),
        }
    }
}

impl Actor for TrackerActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::debug!("Starting TrackerActor");

        if self.trigger_task.is_some() {
            panic!("TrackerActor is just starting up but already has a handle to a running trigger_task");
        }

        let self_addr = ctx.address();
        let mut interval = actix::clock::interval(self.trigger_period);
        let task_handle = ctx.spawn(wrap_future(async move {
            loop {
                interval.tick().await;
                self_addr.send(TriggerUpdatesMsg {}).await.unwrap();
            }
        }));
        self.trigger_task = Some(task_handle);
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        match self.trigger_task {
            None => panic!("TrackerActor is stopping but has no running trigger_task"),
            Some(task_handle) => ctx.cancel_future(task_handle),
        };
    }
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

impl Handler<SubscribeMsg> for TrackerActor {
    type Result = watch::Receiver<Vec<SetPixelMsg>>;

    fn handle(&mut self, _msg: SubscribeMsg, _ctx: &mut Self::Context) -> Self::Result {
        self.publisher.subscribe()
    }
}

impl Handler<TriggerUpdatesMsg> for TrackerActor {
    type Result = ();

    fn handle(&mut self, _msg: TriggerUpdatesMsg, _ctx: &mut Self::Context) -> Self::Result {
        if !self.publisher.is_closed() {
            self.publisher
                .send(
                    self.tracker
                        .get_changes()
                        .map(|change| SetPixelMsg {
                            x: change.coordinates.0,
                            y: change.coordinates.1,
                            color: change.color,
                        })
                        .collect(),
                )
                .unwrap();
        }
    }
}

/// A message which requests a subscription from the TrackerActor.
/// It is responded with a channel on which changes are regularly broadcast.
#[derive(Debug, Copy, Clone, Message)]
#[rtype(result = "watch::Receiver<Vec<SetPixelMsg>>")]
pub struct SubscribeMsg {}

impl MessageResponse<TrackerActor, SubscribeMsg> for watch::Receiver<Vec<SetPixelMsg>> {
    fn handle(
        self,
        _ctx: &mut Context<TrackerActor>,
        tx: Option<OneshotSender<watch::Receiver<Vec<SetPixelMsg>>>>,
    ) {
        if let Some(tx) = tx {
            tx.send(self).unwrap();
        }
    }
}

/// A message that triggers an [`TrackerActor`] to send out all tracked changes and broadcast them to the
/// rest of the system.
#[derive(Debug, Copy, Clone, Message)]
#[rtype(result = "()")]
pub struct TriggerUpdatesMsg {}
