//!
//! Encoding of pixmaps with different algorithms
//!
//! A pixelflut server is able to send it's pixmap using different encoding mechanisms to a
//! requesting clients.
//! This module implements the defined encoding algorithms and also provides background threads
//! which periodically re-encode a pixmap.
//!

mod auto_encoder;
mod encodings;
mod multi_encoders_client;
mod rgb64;
mod rgba64;

pub use auto_encoder::AutoEncoder;
pub use encodings::{Encoder, GetEncodedDataMsg};
pub use multi_encoders_client::MultiEncodersClient;
pub use rgb64::Rgb64Encoder;
pub use rgba64::Rgba64Encoder;
