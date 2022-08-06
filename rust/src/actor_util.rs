use actix::prelude::*;

/// A message that stops the receiving actor
#[derive(Debug, Copy, Clone, Message)]
#[rtype(result = "()")]
pub struct StopActorMsg {}
