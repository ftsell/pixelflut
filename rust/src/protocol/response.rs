//!
//! Data structures for pixelflut responses received from a server
//!

use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use bytes::Buf;
use nom::Err;

use crate::i18n;
use crate::net::framing::Frame;
use crate::pixmap::Color;
use crate::protocol::{HelpTopic, StateEncodingAlgorithm};

use super::parsers;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Response {
    Help(HelpTopic),
    Size(usize, usize),
    Px(usize, usize, Color),
    State(StateEncodingAlgorithm, String),
    SubscriptionActivated,
    SubscriptionDeactivated,
}

impl FromStr for Response {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parsers::parse_response(s) {
            Ok((_remainder, response)) => Ok(response),
            Err(e) => match e {
                Err::Error(e) => Err(e.into()),
                Err::Failure(e) => Err(e.into()),
                Err::Incomplete(_) => Err(anyhow::Error::msg("too little input")),
            },
        }
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Response::Help(topic) => match topic {
                HelpTopic::General => f.write_str(i18n::HELP_GENERAL),
                HelpTopic::Size => f.write_str(i18n::HELP_SIZE),
                HelpTopic::Px => f.write_str(i18n::HELP_PX),
                HelpTopic::State => f.write_str(i18n::HELP_STATE),
            },
            Response::Size(width, height) => f.write_fmt(format_args!("SIZE {} {}", width, height)),
            Response::Px(x, y, color) => f.write_fmt(format_args!("PX {} {} #{:X}", x, y, color)),
            Response::State(alg, data) => f.write_fmt(format_args!("STATE {} {}", alg, data)),
            Response::SubscriptionActivated => f.write_str("SUBSCRIBED"),
            Response::SubscriptionDeactivated => f.write_str("UNSUBSCRIBED"),
        }
    }
}

impl<I> TryFrom<Frame<I>> for Response
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
            assert_eq!($cmd, Response::from_str($cmd).unwrap().to_string());
        };
    }

    assert_parsing!("SIZE 400 600");
    assert_parsing!("PX 42 42 #890ABC");
    assert_parsing!("STATE RGB64 d2hpdGU=");
    assert_parsing!("STATE RGBA64 d2hpdGU=");
}
