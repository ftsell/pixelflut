use crate::pixmap::pixmap_actor::{GetRawDataMsg, GetSizeMsg, PixmapActor};
use crate::pixmap::Pixmap;
use crate::state_encoding::{Encoder, GetEncodedDataMsg};
use actix::dev::MessageResponse;
use actix::fut::wrap_future;
use actix::prelude::*;
use std::time::Duration;

/// An AutoEncoder periodically encodes the pixmap content and caches the result.
/// Manual encoding can be triggered by sending a [`TriggerEncodingMsg`] to the running actor.
///
/// ## Startup
/// On Actor startup, an additional future is spawned into the actors context that periodically triggers
///
/// ## Shutdown
/// On Actor shutdown, automatic encoding is stopped
#[derive(Debug)]
pub struct AutoEncoder<P, E>
where
    P: Pixmap + Unpin + 'static,
    E: Encoder,
{
    interval_period: Duration,
    interval_handle: Option<SpawnHandle>,
    pixmap_addr: Addr<PixmapActor<P>>,
    cache: E::Storage,
}

impl<P, E> AutoEncoder<P, E>
where
    P: Pixmap + Unpin + 'static,
    E: Encoder,
{
    pub fn new(interval_period: Duration, pixmap_addr: Addr<PixmapActor<P>>) -> Self {
        Self {
            interval_handle: None,
            cache: E::Storage::default(),
            interval_period,
            pixmap_addr,
        }
    }
}

impl<P, E> Actor for AutoEncoder<P, E>
where
    P: Pixmap + Unpin + 'static,
    E: Encoder + 'static,
    <E as Encoder>::Storage: Unpin + 'static,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        if self.interval_handle.is_some() {
            panic!("AutoEncoder actor was trying to be started but already has an interval handle");
        }

        let mut interval = actix::clock::interval(self.interval_period);
        let self_addr = ctx.address();
        let handle = ctx.spawn(wrap_future(async move {
            interval.tick().await;
            self_addr.send(TriggerEncodingMsg {}).await.unwrap();
            ()
        }));

        self.interval_handle = Some(handle);
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        match self.interval_handle {
            None => panic!("AutoEncoder was trying to be stopped but the interval handle was lost"),
            Some(handle) => ctx.cancel_future(handle),
        };
    }
}

impl<P, E> Handler<TriggerEncodingMsg> for AutoEncoder<P, E>
where
    P: Pixmap + Unpin + 'static,
    E: Encoder + 'static,
    <E as Encoder>::Storage: Unpin + 'static,
{
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, _msg: TriggerEncodingMsg, _ctx: &mut Self::Context) -> Self::Result {
        let pixmap_addr = self.pixmap_addr.clone();
        Box::pin(
            // retrieve pixmap size and data from it
            async move {
                let (pixmap_size, pixmap_data) = tokio::join!(
                    pixmap_addr.send(GetSizeMsg {}),
                    pixmap_addr.send(GetRawDataMsg {})
                );
                (pixmap_size.unwrap(), pixmap_data.unwrap())
            }
            // turn future into an actor-future so the own state is accessible
            .into_actor(self)
            // encode the returned data and save it in cache
            .map(|(pixmap_size, pixmap_data), selff, _ctx| {
                let (pixmap_width, pixmap_height) = pixmap_size.unwrap();
                let encoding_result = E::encode(pixmap_width, pixmap_height, &pixmap_data.unwrap());
                selff.cache = encoding_result;
            }),
        )
    }
}

impl<P, E> Handler<GetEncodedDataMsg<E>> for AutoEncoder<P, E>
where
    P: Pixmap + Unpin + 'static,
    E: Encoder + 'static,
    <E as Encoder>::Storage: MessageResponse<Self, GetEncodedDataMsg<E>> + Unpin + 'static,
{
    type Result = E::Storage;

    fn handle(&mut self, _msg: GetEncodedDataMsg<E>, _ctx: &mut Self::Context) -> Self::Result {
        self.cache.clone()
    }
}

/// A message that triggers an [`AutoEncoder`] to re-encode the pixmap content
#[derive(Debug, Copy, Clone, Message)]
#[rtype(result = "()")]
pub struct TriggerEncodingMsg {}
