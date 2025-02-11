use crate::{single, utf8_seq};
use lazy_static::lazy_static;
use nom::branch::alt;
use nom::bytes::complete::{take_while, take_while1, take_while_m_n};
use nom::bytes::streaming::tag_no_case;
use nom::character::streaming::{char, crlf};
use nom::combinator::{cut, recognize};
use nom::error::{ErrorKind, FromExternalError, ParseError};
use nom::multi::{fold_many0, many0, separated_list1};
use nom::sequence::separated_pair;
use nom::{AsChar, IResult, Parser};
use nom_language::error::{VerboseError, VerboseErrorKind};
use std::str::FromStr;
use std::sync::Mutex;

mod component;
mod first_pass;
mod language_tag;
mod object;
mod param;
mod property;

/// Types produced by the parser.
///
/// These types represent the structure of the iCalendar format.
pub mod types;

use crate::parser::types::{ContentLine, ParamValue};
pub use first_pass::content_line_first_pass;
pub use object::{ical_object, ical_stream};
pub use param::value::*;
pub use param::{property_param, property_params};
pub use property::component::*;
pub use property::recur::prop_value_recur;
pub use property::uri::param_value_uri;
pub use property::value::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Error<'a> {
    pub input: &'a [u8],
    pub error: InnerError,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InnerError {
    Nom(ErrorKind),
    XNameTooShort,
    EncodingError(String, std::str::Utf8Error),
    InvalidDateNum,
    InvalidTimeNum,
    InvalidDurationNum,
    InvalidFloatNum,
    InvalidIntegerNum,
    InvalidRecurNum,
    InvalidRecurPart(String),
    InvalidOctet,
    InvalidIpv6,
    InvalidPort,
    MismatchedComponentEnd(Vec<u8>, Vec<u8>),
    UnknownParamName(String),
    InvalidValueParam,
    InvalidBinaryValueSpec,
}

impl<'a> Error<'a> {
    pub fn new(input: &'a [u8], error: InnerError) -> Error<'a> {
        Error { input, error }
    }
}

impl<'a> ParseError<&'a [u8]> for Error<'a> {
    fn from_error_kind(input: &'a [u8], kind: ErrorKind) -> Self {
        Error {
            input,
            error: InnerError::Nom(kind),
        }
    }

    fn append(input: &'a [u8], kind: ErrorKind, _other: Self) -> Self {
        Error {
            input,
            error: InnerError::Nom(kind),
        }
    }
}

// Enables use of `map_res` with nom::Err for the custom Error type.
impl<'a> FromExternalError<&'a [u8], nom::Err<Error<'a>>> for Error<'a> {
    fn from_external_error(input: &'a [u8], kind: ErrorKind, e: nom::Err<Error<'a>>) -> Self {
        match e {
            nom::Err::Error(e) | nom::Err::Failure(e) => Error {
                input: e.input,
                error: e.error,
            },
            nom::Err::Incomplete(_) => Error {
                input,
                error: InnerError::Nom(kind),
            },
        }
    }
}

impl<'a> From<(&'a [u8], ErrorKind)> for Error<'a> {
    fn from((input, kind): (&'a [u8], ErrorKind)) -> Self {
        Error {
            input,
            error: InnerError::Nom(kind),
        }
    }
}

lazy_static! {
    static ref ERROR_HOLD: Mutex<Vec<(usize, usize)>> = Mutex::new(Vec::new());
}

#[cfg(test)]
pub(crate) unsafe fn clear_errors() {
    for (ptr, len) in ERROR_HOLD.lock().unwrap().drain(..) {
        unsafe { String::from_raw_parts(ptr as *mut u8, len, len) };
    }
}

impl<'a> From<Error<'a>> for VerboseError<&'a [u8]> {
    fn from(value: Error<'a>) -> Self {
        let ctx = Box::leak(format!("{:?}", value.error).to_string().into_boxed_str());

        ERROR_HOLD
            .lock()
            .unwrap()
            .push((ctx.as_ptr() as usize, ctx.len()));

        VerboseError {
            errors: vec![(value.input, VerboseErrorKind::Context(ctx))],
        }
    }
}

/// All ASCII control characters except tab (%x09).
#[inline]
const fn is_control(b: u8) -> bool {
    matches!(b, b'\0'..=b'\x08' | b'\x0A'..=b'\x1F' | b'\x7F')
}

fn param_text<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8], E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    take_while(|c| c != b'\"' && c != b';' && c != b':' && c != b',' && !is_control(c))(input)
}

fn quoted_string<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8], E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (_, content, _)) = (char('"'), cut(safe_char), char('"')).parse(input)?;

    Ok((input, content))
}

fn param_value<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8], E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, value) = alt((quoted_string, param_text)).parse(input)?;

    Ok((input, value))
}

fn safe_char<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8], E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    take_while(|c| c != b'\"' && !is_control(c))(input)
}

fn iana_token<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8], E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    take_while1(|c: u8| c.is_alphanum() || c == b'-')(input)
}

fn x_name<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8], E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, x_name) = recognize((
        tag_no_case("X-"),
        cut(take_while1(|c: u8| c.is_alphanum() || c == b'-')),
    ))
    .parse(input)?;

    Ok((input, x_name))
}

fn name<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8], E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    alt((iana_token, x_name)).parse(input)
}

fn param_name<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8], E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    alt((iana_token, x_name)).parse(input)
}

#[inline]
const fn is_reg_name_char(b: u8) -> bool {
    matches!(b, b'\x41'..=b'\x5A' | b'\x61'..=b'\x7A' | b'\x30'..=b'\x39' | b'\x21' | b'\x23' | b'\x24' | b'\x26' | b'\x2E' | b'\x2B' | b'\x2D' | b'\x5E' | b'\x5F')
}

// See https://www.rfc-editor.org/rfc/rfc4288 section 4.2
fn reg_name<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8], E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    take_while_m_n(1, 127, is_reg_name_char)(input)
}

fn read_string<'a, E>(input: &'a [u8], context: &str) -> Result<String, nom::Err<E>>
where
    E: ParseError<&'a [u8]>,
    E: From<Error<'a>>,
{
    Ok(std::str::from_utf8(input)
        .map_err(|e| {
            nom::Err::Failure(
                Error::new(input, InnerError::EncodingError(context.to_string(), e)).into(),
            )
        })?
        .to_string())
}

fn read_int<'a, E, N>(input: &'a [u8]) -> Result<N, nom::Err<E>>
where
    E: ParseError<&'a [u8]>,
    E: From<Error<'a>>,
    N: FromStr,
{
    std::str::from_utf8(input)
        .map_err(|e| {
            nom::Err::Error(
                Error::new(
                    input,
                    InnerError::EncodingError("Invalid integer number text".to_string(), e),
                )
                .into(),
            )
        })?
        .parse()
        .map_err(|_| nom::Err::Error(Error::new(input, InnerError::InvalidIntegerNum).into()))
}

fn line_value<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Vec<u8>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, v) = fold_many0(value_char, Vec::new, |mut acc, item| {
        acc.extend_from_slice(&item);
        acc
    })
    .parse(input)?;

    Ok((input, v))
}

fn value_char<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Vec<u8>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    alt((
        single(|b| matches!(b, b' ' | b'\t' | b'\x21'..=b'\x7E')).map(|c| vec![c]),
        utf8_seq.map(|c| c.to_vec()),
    ))
    .parse(input)
}

fn value<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Vec<u8>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    fold_many0(value_char, Vec::new, |mut acc, item| {
        acc.extend_from_slice(&item);
        acc
    })
    .parse(input)
}

fn param<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (name, values)) = separated_pair(
        param_name,
        char('='),
        cut(separated_list1(char(','), param_value)),
    )
    .parse(input)?;

    Ok((
        input,
        if values.len() == 1 {
            ParamValue::Other {
                name,
                value: values[0],
            }
        } else {
            ParamValue::Others { name, values }
        },
    ))
}

fn content_line<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ContentLine<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (property_name, params, _, value, _)) = (
        name,
        many0((char(';'), cut(param))).map(|v| v.into_iter().map(|(_, p)| p).collect()),
        char(':'),
        cut(line_value),
        crlf,
    )
        .parse(input)?;

    Ok((
        input,
        ContentLine {
            property_name,
            params,
            value,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::check_rem;

    #[test]
    fn iana_token_desc() {
        let (rem, token) = iana_token::<Error>(b"DESCRIPTION").unwrap();
        check_rem(rem, 0);
        assert_eq!(b"DESCRIPTION", token);
    }

    #[test]
    fn simple_x_name() {
        let (rem, x_name) = x_name::<Error>(b"X-TEST ").unwrap();
        check_rem(rem, 1);
        assert_eq!(b"X-TEST", x_name);
    }

    #[test]
    fn simple_x_name_with_vendor() {
        let (rem, x_name) = x_name::<Error>(b"X-ESL-TEST ").unwrap();
        check_rem(rem, 1);
        assert_eq!(b"X-ESL-TEST", x_name);
    }

    #[test]
    fn simple_content_line() {
        let (rem, content_line) = content_line::<Error>(
            b"DESCRIPTION:This is a long description that exists on a long line.\r\nnext",
        )
        .unwrap();
        check_rem(rem, 4);
        assert_eq!(b"DESCRIPTION", content_line.property_name);
        assert_eq!(
            b"This is a long description that exists on a long line.",
            content_line.value.as_slice()
        );
    }

    #[test]
    fn simple_content_line_utf8() {
        let (rem, content_line) = content_line::<Error>(
            "DESCRIPTION:This is a long description of a happy face - 😁.\r\n;".as_bytes(),
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(b"DESCRIPTION", content_line.property_name);
        assert_eq!(
            "This is a long description of a happy face - 😁.".as_bytes(),
            content_line.value.as_slice()
        );
    }
}
