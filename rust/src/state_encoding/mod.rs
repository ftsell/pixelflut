//!
//! Encoding of pixmaps with different algorithms
//!
//! A pixelflut server is able to send it's pixmap using different encoding mechanisms to a
//! requesting clients.
//! This module implements the defined encoding algorithms and also provides background threads
//! which periodically re-encode a pixmap.
//!

mod encodings;
mod multi_encoders_client;
mod rgb64;
mod rgba64;
mod auto_encoder;

pub use encodings::*;
pub use rgb64::Rgb64Encoder;
pub use rgba64::Rgba64Encoder;
pub use auto_encoder::AutoEncoder;
