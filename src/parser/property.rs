mod recur;
mod value_types;
mod uri;
mod value;
mod types;

use nom::branch::alt;
use nom::bytes::complete::take_while1;
use nom::bytes::streaming::tag;
use nom::IResult;
use nom::sequence::tuple;
pub use value_types::*;
pub use value::*;
use crate::parser::Error;
use crate::parser::param::other_params;
use crate::parser::property::types::VersionProperty;

pub fn prop_version(input: &[u8]) -> IResult<&[u8], VersionProperty, Error> {
    let (input, (_, value, _)) = tuple((tag("VERSION"), other_params, alt((
        tag("2.0"),
        take_while1(is_)
        )), tag("\r\n")))(input)?;

    Ok((input, Property::Version(value)))
}
