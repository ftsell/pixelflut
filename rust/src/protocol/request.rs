//!
//! Data structures for pixelflut requests sent to a server
//!

use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use bytes::Buf;

use crate::net::framing::Frame;
use crate::pixmap::Color;
use crate::protocol::{HelpTopic, StateEncodingAlgorithm};

use super::parsers;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Request {
    Help(HelpTopic),
    Size,
    PxGet(usize, usize),
    PxSet(usize, usize, Color),
    State(StateEncodingAlgorithm),
    Subscribe,
    Unsubscribe,
}

impl FromStr for Request {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parsers::parse_request(s) {
            Ok((_remainder, request)) => Ok(request),
            Err(e) => match e {
                nom::Err::Error(e) => Err(e.into()),
                nom::Err::Failure(e) => Err(e.into()),
                nom::Err::Incomplete(_) => Err(anyhow::Error::msg("too little input")),
            },
        }
    }
}

impl Display for Request {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Request::Help(topic) => match topic {
                HelpTopic::General => f.write_str("HELP"),
                HelpTopic::State => f.write_str("HELP STATE"),
                HelpTopic::Size => f.write_str("HELP SIZE"),
                HelpTopic::Px => f.write_str("HELP PX"),
            },
            Request::Size => f.write_str("SIZE"),
            Request::PxGet(x, y) => f.write_fmt(format_args!("PX {} {}", x, y)),
            Request::PxSet(x, y, color) => f.write_fmt(format_args!("PX {} {} #{:X}", x, y, color)),
            Request::State(alg) => f.write_fmt(format_args!("STATE {}", alg)),
            Request::Subscribe => f.write_str("SUBSCRIBE"),
            Request::Unsubscribe => f.write_str("UNSUBSCRIBE"),
        }
    }
}

impl<I> TryFrom<Frame<I>> for Request
where
    I: Buf,
{
    type Error = anyhow::Error;

    fn try_from(value: Frame<I>) -> Result<Self, Self::Error> {
        let string: String = value.try_into()?;
        Ok(Self::from_str(&string)?)
    }
}

#[cfg(test)]
#[test]
fn test_from_string_to_string() {
    macro_rules! assert_parsing {
        ($cmd:literal) => {
            assert_eq!($cmd, Request::from_str($cmd).unwrap().to_string());
        };
    }

    assert_parsing!("HELP");
    assert_parsing!("HELP STATE");
    assert_parsing!("HELP SIZE");
    assert_parsing!("HELP PX");
    assert_parsing!("SIZE");
    assert_parsing!("PX 42 42");
    assert_parsing!("PX 42 42 #890ABC");
    assert_parsing!("STATE RGB64");
    assert_parsing!("STATE RGBA64");
}
