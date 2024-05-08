use nom::{AsChar, IResult, Parser};
use nom::branch::alt;
use nom::bytes::complete::{take_until, take_while1};
use nom::bytes::streaming::tag;
use nom::character::streaming::{alpha1, alphanumeric1, char, crlf};
use nom::combinator::opt;
use nom::error::{ErrorKind, ParseError};
use nom::multi::separated_list1;
use nom::sequence::{separated_pair, tuple};

#[derive(Clone, Debug, PartialEq)]
pub struct Error<'a> {
    pub input: &'a [u8],
    pub error: InnerError,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InnerError {
    Nom(ErrorKind),
    XNameTooShort,
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

impl<'a> From<(&'a [u8], ErrorKind)> for Error<'a> {
    fn from((input, kind): (&'a [u8], ErrorKind)) -> Self {
        Error {
            input,
            error: InnerError::Nom(kind),
        }
    }
}

struct ContentLine {
    property_name: Vec<u8>,
    value: Vec<u8>,
}

fn iana_token(input: &[u8]) -> IResult<&[u8], &[u8], Error>
{
    take_while1(|c: u8| c.is_alphanum() || c == b'-')(input)
}

fn vendor_id(input: &[u8]) -> IResult<&[u8], &[u8], Error>
{
    let (rest, id) = alphanumeric1(input)?;

    if id.len() < 3 {
        return Err(nom::Err::Failure(Error::new(rest, InnerError::XNameTooShort)))
    }

    Ok((rest, id))
}

fn x_name(input: &[u8]) -> IResult<&[u8], Vec<u8>, Error>
{
    let (input, (prefix, maybe_vendor_id, name)) = tuple((tag("X-"), opt(tuple((vendor_id, char('-')))), take_while1(|c: u8| c.is_alphanum() || c == b'-')))(input)?;

    let mut n = prefix.to_vec();
    if let Some((vendor_id, _)) = maybe_vendor_id {
        n.extend_from_slice(vendor_id);
        n.push(b'-');
    }
    n.extend_from_slice(name);

    Ok((input, n))
}

fn name(input: &[u8]) -> IResult<&[u8], Vec<u8>, Error>
{
    alt((iana_token.map(|t| t.to_vec()), x_name))(input)
}

fn param_name(input: &[u8]) -> IResult<&[u8], Vec<u8>, Error>
{
    alt((iana_token.map(|t| t.to_vec()), x_name))(input)
}

fn param(input: &[u8]) -> IResult<&[u8], (Vec<u8>, Vec<u8>), Error>
{
    let (input, (name, _, value)) = tuple((param_name, char('='), take_until(";")))(input)?;

    Ok((input, (name, value.to_vec())))
}

fn parse_line_content(input: &[u8]) -> IResult<&[u8], Vec<u8>, Error> {
    let (input, (parts, _)) = tuple((separated_list1(
        tuple((crlf, alt((char(' '), char('\t'))))),
        take_until("\r\n"),
    ), crlf))(input)?;

    Ok((
        input,
        parts.iter().map(|c| c).fold(vec![], |mut acc, x| {
            acc.extend_from_slice(x);
            acc
        }),
    ))
}

fn parse_line(input: &[u8]) -> IResult<&[u8], ContentLine, Error> {
    let (input, (property_name, value)) = separated_pair(
        name,
        char(':'),
        parse_line_content,
    )(input)?;

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
        assert_eq!(b"X-TEST", x_name.as_slice());
    }

    #[test]
    fn simple_x_name_with_vendor() {
        let (rem, x_name) = x_name(b"X-ESL-TEST ").unwrap();
        check_rem(rem, 1);
        assert_eq!(b"X-ESL-TEST", x_name.as_slice());
    }

    #[test]
    fn simple_content_line() {
        let (rem, content_line) =
            parse_line(b"DESCRIPTION:This is a long description that exists on a long line.\r\nnext")
                .unwrap();
        check_rem(rem, 4);
        assert_eq!(b"DESCRIPTION", content_line.property_name.as_slice());
        assert_eq!(b"This is a long description that exists on a long line.", content_line.value.as_slice());
    }

    #[test]
    fn content_line_multi_line() {
        let (rem, content_line) =
            parse_line(b"DESCRIPTION:This is a lo\r\n ng description\r\n  that exists on a long line.\r\nnext")
                .unwrap();
        check_rem(rem, 4);
        assert_eq!(b"DESCRIPTION", content_line.property_name.as_slice());
        assert_eq!(b"This is a long description that exists on a long line.", content_line.value.as_slice(), "Got: {}", String::from_utf8(content_line.value.clone()).unwrap());
    }

    #[test]
    fn content_line_multi_line_with_tab() {
        let (rem, content_line) =
            parse_line(b"DESCRIPTION:This is a lo\r\n ng description\r\n\t that exists on a long line.\r\nnext")
                .unwrap();
        check_rem(rem, 4);
        assert_eq!(b"DESCRIPTION", content_line.property_name.as_slice());
        assert_eq!(b"This is a long description that exists on a long line.", content_line.value.as_slice(), "Got: {}", String::from_utf8(content_line.value.clone()).unwrap());
    }

    fn check_rem(rem: &[u8], expected_len: usize) {
        if rem.len() != expected_len {
            let str = String::from_utf8(rem.to_vec()).unwrap();
            println!("rem: {str}");
        }
        assert_eq!(expected_len, rem.len(), "Remainder length should be {expected_len} but was {}", rem.len());
    }
}
