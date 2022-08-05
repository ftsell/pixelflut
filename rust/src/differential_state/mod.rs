//! Functionality related to tracking pixmap state as a difference to how they were before
//!
//! This is necessary to implement the `subscribe` command which allows clients to receive pixmap updates
//! as discrete events and not as retransmissions of the whole canvas.

use std::sync::{Arc, Mutex};

mod tracker;
pub use tracker::Tracker;

/// A [`Tracker`] that can be shared between threads safely
pub type SharedTracker = Arc<Mutex<Tracker>>;
