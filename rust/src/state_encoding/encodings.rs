use crate::pixmap::Color;
use actix::prelude::*;
use anyhow::Result;
use std::marker::PhantomData;

/// An encoder can [`encode`](Encoder::encode()) and [`decode`](Encoder::decode()) pixmap data in a certain
/// format
pub trait Encoder {
    /// The type which the encoder outputs after it encodes the pixmap and which it can decode again
    type ResultFormat: AsRef<[u8]> + Default + Clone;

    /// Encode the given *pixmap_data* in this encoders format
    fn encode(pixmap_width: usize, pixmap_height: usize, pixmap_data: &[Color]) -> Self::ResultFormat;

    /// Decode the given *data* back into colors
    fn decode(data: &Self::ResultFormat) -> Result<Vec<Color>>;
}

/// A message which queries encoded data from something
#[derive(Debug, Copy, Clone, Message)]
#[rtype(result = "E::ResultFormat")]
pub struct GetEncodedDataMsg<E: Encoder + 'static> {
    _phantom: PhantomData<E>,
}

impl<E: Encoder + 'static> GetEncodedDataMsg<E> {
    /// Create a new GetEncodedDataMsg
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData::default(),
        }
    }
}
