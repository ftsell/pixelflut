use crate::state_encoding::{GetEncodedDataMsg, Rgb64Encoder, Rgba64Encoder};
use actix::prelude::*;

/// A client that automatically dispatches to the desired actor depending on which encoding is queried
#[derive(Debug, Clone)]
pub struct MultiEncodersClient {
    rgb64_addr: Recipient<GetEncodedDataMsg<Rgb64Encoder>>,
    rgba64_addr: Recipient<GetEncodedDataMsg<Rgba64Encoder>>,
}

impl MultiEncodersClient {
    /// Create a new MultiEncodersClient which is backed by the given encoder
    pub fn new(
        rgb64_addr: Recipient<GetEncodedDataMsg<Rgb64Encoder>>,
        rgba64_addr: Recipient<GetEncodedDataMsg<Rgba64Encoder>>,
    ) -> Self {
        Self {
            rgb64_addr,
            rgba64_addr,
        }
    }

    /// Retrieve *rgb64* encoded data
    pub async fn get_rgb64_data(&self) -> String {
        self.rgb64_addr.send(GetEncodedDataMsg::new()).await.unwrap()
    }

    /// Retrieve *rgba64* encoded data
    pub async fn get_rgba64_data(&self) -> String {
        self.rgba64_addr.send(GetEncodedDataMsg::new()).await.unwrap()
    }
}
