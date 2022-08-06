use crate::pixmap::Color;
use actix::prelude::*;
use anyhow::Result;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use super::*;

pub trait Encoder {
    type Storage: AsRef<[u8]>;

    fn encode(pixmap_width: usize, pixmap_height: usize, pixmap_data: &[Color]) -> Self::Storage;

    fn decode(data: &Self::Storage) -> Result<Vec<Color>>;
}

#[derive(Debug, Copy, Clone, Message)]
#[rtype(result = "E::Storage")]
pub struct GetEncodedDataMsg<E: Encoder + 'static> {
    _phantom: PhantomData<E>,
}
