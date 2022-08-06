//! Functionality related to tracking pixmap state as a difference to how they were before
//!
//! This is necessary to implement the `subscribe` command which allows clients to receive pixmap updates
//! as discrete events and not as retransmissions of the whole canvas.

use std::sync::{Arc, Mutex};

mod tracker;
mod tracker_actor;

pub use tracker::Tracker;
