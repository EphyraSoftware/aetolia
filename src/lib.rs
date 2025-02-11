#![doc = include_str!("../README.md")]

use crate::parser::Error;
use nom::branch::alt;
use nom::combinator::recognize;
use nom::error::ParseError;
use nom::{IResult, Input, Parser};
use std::num::NonZeroUsize;

/// Common types.
pub mod common;

/// Conversion from the parser model to the core representation.
pub mod convert;

/// The core representation that is used for everything except the parser.
pub mod model;

/// Common operations.
pub mod ops;

/// The iCalendar parser.
pub mod parser;

/// The serializer for the core representation back to the iCalendar text format.
pub mod serialize;

/// Validation of iCalendar rules against the core representation.
pub mod validate;

#[cfg(test)]
mod test_utils;

/// Prelude which contains everything that's needed for most use-cases to consume this library.
pub mod prelude {
    pub use crate::common::PropertyKind;
    pub use crate::common::*;
    pub use crate::model::access::*;
    pub use crate::model::component::*;
    pub use crate::model::object::*;
    pub use crate::model::param::*;
    pub use crate::model::property::*;
    pub use crate::ops::{load_ical, read_ical};
    pub use crate::parser::{content_line_first_pass, ical_object, ical_stream};
    pub use crate::serialize::WriteModel;
    pub use crate::validate::{validate_model, ICalendarErrorSeverity};
}

/// Streaming, single character matching the predicate
pub(crate) fn single<F, In, Output, Error: ParseError<In>>(
    cond: F,
) -> impl Fn(In) -> IResult<In, Output, Error>
where
    In: Input<Item = Output>,
    F: Fn(<In as Input>::Item) -> bool,
    Output: Copy,
{
    move |i: In| {
        match i.iter_elements().next() {
            Some(c) if cond(c) => {
                let (input, v) = i.take_split(1);
                Ok((input, v.iter_elements().next().unwrap()))
            }
            // Closest error I can get, can't add to the Nom enum!
            Some(_) => Err(nom::Err::Error(Error::from_error_kind(
                i,
                nom::error::ErrorKind::OneOf,
            ))),
            None => Err(nom::Err::Incomplete(nom::Needed::Size(
                NonZeroUsize::new(1).unwrap(),
            ))),
        }
    }
}

fn utf8_seq<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8], E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, seq) = alt((
        // Utf-8 2-byte sequence
        recognize((
            single(|b| matches!(b, b'\xC2'..=b'\xDF')),
            single(|b| matches!(b, b'\x80'..=b'\xBF')),
        )),
        // Utf-8 3-byte sequence
        alt((
            recognize((
                single(|b| b == b'\xE0'),
                single(|b| matches!(b, b'\xA0'..=b'\xBF')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
            )),
            recognize((
                single(|b| matches!(b, b'\xE1'..=b'\xEC')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
            )),
            recognize((
                single(|b| b == b'\xED'),
                single(|b| matches!(b, b'\x80'..=b'\x9F')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
            )),
            recognize((
                single(|b| matches!(b, b'\xEE'..=b'\xEF')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
            )),
        )),
        // Utf-8 4-byte sequence
        alt((
            recognize((
                single(|b| b == b'\xF0'),
                single(|b| matches!(b, b'\x90'..=b'\xBF')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
            )),
            recognize((
                single(|b| matches!(b, b'\xF1'..=b'\xF3')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
            )),
            recognize((
                single(|b| b == b'\xF4'),
                single(|b| matches!(b, b'\x80'..=b'\x8F')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
            )),
        )),
    ))
    .parse(input)?;

    Ok((input, seq))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::multi::many1;

    #[test]
    fn valid_utf8() {
        let (rem, seq) = utf8_seq::<Error>("👍".as_bytes()).unwrap();
        test_utils::check_rem(rem, 0);
        assert_eq!(seq, "👍".as_bytes());
    }

    #[test]
    fn invalid_utf8() {
        let mut input = "👍👌".as_bytes().to_vec();
        input.extend_from_slice(&[1, 3, 4, 5, 2, 1]);
        let (rem, seq) = many1(utf8_seq::<Error>).parse(input.as_slice()).unwrap();
        test_utils::check_rem(rem, 6);
        assert_eq!(
            seq.into_iter().flatten().cloned().collect::<Vec<_>>(),
            "👍👌".as_bytes().to_vec()
        );
    }
}
