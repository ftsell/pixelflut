use nom::branch::{alt, permutation};
use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace1;
use nom::combinator::{cut, eof, flat_map, map, value};
use nom::IResult;

use crate::protocol::request::Request::PxGet;
use crate::protocol::{HelpTopic, Request};

use super::combinators::*;
use super::{coordinate, encoding_algorithm, help_topic, hex_color, Error};

pub fn parse(input: &str) -> IResult<&str, Request, Error> {
    Ok(alt((
        // SIZE
        value(Request::Size, tag_no_case("size")),
        // HELP
        cond_parse(
            tag_no_case("help"),
            cut(alt((
                // HELP TOPIC
                cond_parse(
                    multispace1,
                    cut(map(help_topic::parse, |topic| Request::Help(topic))),
                ),
                // HELP (no topic)
                value(Request::Help(HelpTopic::General), eof),
            ))),
        ),
        // STATE
        cond_parse(
            tag_no_case("state"),
            cut(cond_parse(
                multispace1,
                map(encoding_algorithm::parse, |alg| Request::State(alg)),
            )),
        ),
        // PX
        cond_parse(
            tag_no_case("px"),
            cut(flat_map(
                permutation((multispace1, coordinate::parse, multispace1, coordinate::parse)),
                |(_, x, _, y)| {
                    alt((
                        // PX X Y
                        value(PxGet(x, y), eof),
                        // PX X Y COLOR
                        cond_parse(
                            multispace1,
                            cut(map(hex_color::parse, move |color| Request::PxSet(x, y, color))),
                        ),
                    ))
                },
            )),
        ),
        // SUBSCRIBE
        value(Request::Subscribe, tag_no_case("subscribe")),
        // UNSUBSCRIBE
        value(Request::Unsubscribe, tag_no_case("unsubscribe")),
    ))(input)?)
}
