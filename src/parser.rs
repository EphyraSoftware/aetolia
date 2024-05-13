#![allow(dead_code)]

use nom::branch::alt;
use nom::bytes::complete::{take_until, take_while, take_while1, take_while_m_n};
use nom::bytes::streaming::tag;
use nom::character::streaming::{alphanumeric1, char, crlf};
use nom::combinator::{opt, recognize};
use nom::error::{ErrorKind, FromExternalError, ParseError};
use nom::multi::separated_list1;
use nom::sequence::{separated_pair, tuple};
use nom::{AsChar, IResult};

mod language_tag;
mod param;
mod property;

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
    InvalidDurationNum,
    InvalidFloatNum,
    InvalidIntegerNum,
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
        tag("X-"),
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

fn parse_line_content(input: &[u8]) -> IResult<&[u8], Vec<u8>, Error> {
    let (input, (parts, _)) = tuple((
        separated_list1(
            tuple((crlf, alt((char(' '), char('\t'))))),
            take_until("\r\n"),
        ),
        crlf,
    ))(input)?;

    Ok((
        input,
        parts.iter().fold(vec![], |mut acc, x| {
            acc.extend_from_slice(x);
            acc
        }),
    ))
}

fn parse_line(input: &[u8]) -> IResult<&[u8], ContentLine, Error> {
    let (input, (property_name, value)) =
        separated_pair(name, char(':'), parse_line_content)(input)?;

    Ok((
        input,
        ContentLine {
            property_name,
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
        let (rem, content_line) = parse_line(
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
        let (rem, content_line) = parse_line(
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
            parse_line(b"DESCRIPTION:This is a lo\r\n ng description\r\n\t that exists on a long line.\r\nnext")
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
