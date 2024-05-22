#![allow(dead_code)]

use crate::parser::param::ParamValue;
use crate::{single, utf8_seq};
use nom::branch::alt;
use nom::bytes::complete::{take_while, take_while1, take_while_m_n};
use nom::bytes::streaming::{tag, tag_no_case};
use nom::character::complete::one_of;
use nom::character::streaming::{alphanumeric1, char, crlf};
use nom::combinator::{opt, recognize};
use nom::error::{ErrorKind, FromExternalError, ParseError};
use nom::multi::{fold_many0, many0, separated_list1};
use nom::sequence::{separated_pair, tuple};
use nom::{AsChar, IResult, Parser};

mod language_tag;
mod object;
mod param;
mod property;
mod component;

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
    UnknownParamName(Vec<u8>),
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

#[derive(Debug, Clone)]
struct ContentLine<'a> {
    property_name: &'a [u8],
    params: Vec<param::Param<'a>>,
    value: Vec<u8>,
}

/// All ASCII control characters except tab (%x09).
#[inline]
const fn is_control(b: u8) -> bool {
    matches!(b, b'\0'..=b'\x08' | b'\x0A'..=b'\x1F' | b'\x7F')
}

fn param_text(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    take_while(|c| c != b'\"' && c != b';' && c != b':' && c != b',' && !is_control(c))(input)
}

fn quoted_string(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    let (input, (_, content, _)) = tuple((
        char('"'),
        take_while(|c| c != b'\"' && !is_control(c)),
        char('"'),
    ))(input)?;

    Ok((input, content))
}

fn param_value(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    let (input, value) = alt((quoted_string, param_text))(input)?;

    Ok((input, value))
}

fn safe_char(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    take_while(|c| c != b'\"' && !is_control(c))(input)
}

fn iana_token(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    take_while1(|c: u8| c.is_alphanum() || c == b'-')(input)
}

fn vendor_id(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    let (rest, id) = alphanumeric1(input)?;

    if id.len() < 3 {
        return Err(nom::Err::Failure(Error::new(
            rest,
            InnerError::XNameTooShort,
        )));
    }

    Ok((rest, id))
}

fn x_name(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    let (input, x_name) = recognize(tuple((
        tag_no_case("X-"),
        opt(tuple((vendor_id, char('-')))),
        take_while1(|c: u8| c.is_alphanum() || c == b'-'),
    )))(input)?;

    Ok((input, x_name))
}

fn name(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    alt((iana_token, x_name))(input)
}

fn param_name(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    alt((iana_token, x_name))(input)
}

#[inline]
const fn is_reg_name_char(b: u8) -> bool {
    matches!(b, b'\x41'..=b'\x5A' | b'\x61'..=b'\x7A' | b'\x30'..=b'\x39' | b'\x21' | b'\x23' | b'\x24' | b'\x26' | b'\x2E' | b'\x2B' | b'\x2D' | b'\x5E' | b'\x5F')
}

// See https://www.rfc-editor.org/rfc/rfc4288 section 4.2
fn reg_name(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    take_while_m_n(1, 127, is_reg_name_char)(input)
}

fn read_string<'a>(input: &'a [u8], context: &str) -> Result<String, nom::Err<Error<'a>>> {
    Ok(std::str::from_utf8(input)
        .map_err(|e| {
            nom::Err::Failure(Error::new(
                input,
                InnerError::EncodingError(context.to_string(), e),
            ))
        })?
        .to_string())
}

fn line_value(input: &[u8]) -> IResult<&[u8], Vec<u8>, Error> {
    let (input, v) = fold_many0(
        alt((
            tuple((tag("\r\n"), one_of(" \t"))).map(|_| vec![]),
            value_char,
        )),
        Vec::new,
        |mut acc, item| {
            acc.extend_from_slice(&item);
            acc
        },
    )(input)?;

    Ok((input, v))
}

fn value_char(input: &[u8]) -> IResult<&[u8], Vec<u8>, Error> {
    alt((
        single(|b| matches!(b, b' ' | b'\t' | b'\x21'..=b'\x7E')).map(|c| vec![c]),
        utf8_seq.map(|c| c.to_vec()),
    ))(input)
}

pub fn value(input: &[u8]) -> IResult<&[u8], Vec<u8>, Error> {
    fold_many0(value_char, Vec::new, |mut acc, item| {
        acc.extend_from_slice(&item);
        acc
    })(input)
}

fn param(input: &[u8]) -> IResult<&[u8], param::Param, Error> {
    let (input, (name, values)) = separated_pair(
        param_name,
        char('='),
        separated_list1(char(','), param_value),
    )(input)?;

    Ok((
        input,
        param::Param {
            name: read_string(name, "param name")?,
            value: ParamValue::Others { values },
        },
    ))
}

fn content_line(input: &[u8]) -> IResult<&[u8], ContentLine, Error> {
    let (input, (property_name, params, _, value, _)) = tuple((
        name,
        many0(tuple((char(';'), param)).map(|(_, p)| p)),
        char(':'),
        line_value,
        crlf,
    ))(input)?;

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
        let (rem, token) = iana_token(b"DESCRIPTION").unwrap();
        check_rem(rem, 0);
        assert_eq!(b"DESCRIPTION", token);
    }

    #[test]
    fn simple_x_name() {
        let (rem, x_name) = x_name(b"X-TEST ").unwrap();
        check_rem(rem, 1);
        assert_eq!(b"X-TEST", x_name);
    }

    #[test]
    fn simple_x_name_with_vendor() {
        let (rem, x_name) = x_name(b"X-ESL-TEST ").unwrap();
        check_rem(rem, 1);
        assert_eq!(b"X-ESL-TEST", x_name);
    }

    #[test]
    fn simple_content_line() {
        let (rem, content_line) = content_line(
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
    fn content_line_multi_line() {
        let (rem, content_line) = content_line(
            b"DESCRIPTION:This is a lo\r\n ng description\r\n  that exists on a long line.\r\nnext",
        )
        .unwrap();
        check_rem(rem, 4);
        assert_eq!(b"DESCRIPTION", content_line.property_name);
        assert_eq!(
            b"This is a long description that exists on a long line.",
            content_line.value.as_slice(),
            "Got: {}",
            String::from_utf8(content_line.value.clone()).unwrap()
        );
    }

    #[test]
    fn content_line_multi_line_with_tab() {
        let (rem, content_line) =
            content_line(b"DESCRIPTION:This is a lo\r\n ng description\r\n\t that exists on a long line.\r\nnext")
                .unwrap();
        check_rem(rem, 4);
        assert_eq!(b"DESCRIPTION", content_line.property_name);
        assert_eq!(
            b"This is a long description that exists on a long line.",
            content_line.value.as_slice(),
            "Got: {}",
            String::from_utf8(content_line.value.clone()).unwrap()
        );
    }
}
