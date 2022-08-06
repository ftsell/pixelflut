use crate::state_encoding::{AutoEncoder, GetEncodedDataMsg, Rgb64Encoder, Rgba64Encoder};
use actix::dev::ToEnvelope;
use actix::prelude::*;

pub type DefaultMultiEncodersClient<P> =
    MultiEncodersClient<AutoEncoder<P, Rgb64Encoder>, AutoEncoder<P, Rgba64Encoder>>;

/// A client that automatically dispatches to the desired actor depending on which encoding is queried
#[derive(Debug, Clone)]
pub struct MultiEncodersClient<E1, E2>
where
    E1: Handler<GetEncodedDataMsg<Rgb64Encoder>>,
    E2: Handler<GetEncodedDataMsg<Rgba64Encoder>>,
{
    rgb64_addr: Addr<E1>,
    rgba64_addr: Addr<E2>,
}

impl<E1, E2> MultiEncodersClient<E1, E2>
where
    E1: Handler<GetEncodedDataMsg<Rgb64Encoder>> + Actor,
    <E1 as Actor>::Context: ToEnvelope<E1, GetEncodedDataMsg<Rgb64Encoder>>,
    E2: Handler<GetEncodedDataMsg<Rgba64Encoder>> + Actor,
    <E2 as Actor>::Context: ToEnvelope<E2, GetEncodedDataMsg<Rgba64Encoder>>,
{
    pub fn new(rgb64_addr: Addr<E1>, rgba64_addr: Addr<E2>) -> Self {
        Self {
            rgb64_addr,
            rgba64_addr,
        }
    }

    pub async fn get_rgb64_data(&self) -> String {
        self.rgb64_addr.send(GetEncodedDataMsg::new()).await.unwrap()
    }

    pub async fn get_rgba64_data(&self) -> String {
        self.rgba64_addr.send(GetEncodedDataMsg::new()).await.unwrap()
    }
}
