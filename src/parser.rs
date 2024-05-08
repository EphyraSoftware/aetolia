use nom::{AsChar, InputTakeAtPosition, IResult};
use nom::branch::alt;
use nom::bytes::complete::take_until;
use nom::character::streaming::{char, crlf};
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

struct ContentLine<'a> {
    property_name: &'a [u8],
    value: Vec<u8>,
}

pub fn upper_alpha1<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar,
{
    input.split_at_position(|item| {
        let c = item.as_char();
        !(c >= 'A' && c <= 'Z')
    })
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
        upper_alpha1,
        nom::character::streaming::char(':'),
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
    fn simple_content_line() {
        let (rem, content_line) =
            parse_line(b"DESCRIPTION:This is a long description that exists on a long line.\r\nnext")
                .unwrap();
        check_rem(rem, 4);
        assert_eq!(b"DESCRIPTION", content_line.property_name);
        assert_eq!(b"This is a long description that exists on a long line.", content_line.value.as_slice());
    }

    #[test]
    fn content_line_multi_line() {
        let (rem, content_line) =
            parse_line(b"DESCRIPTION:This is a lo\r\n ng description\r\n  that exists on a long line.\r\nnext")
                .unwrap();
        check_rem(rem, 4);
        assert_eq!(b"DESCRIPTION", content_line.property_name);
        assert_eq!(b"This is a long description that exists on a long line.", content_line.value.as_slice(), "Got: {}", String::from_utf8(content_line.value.clone()).unwrap());
    }

    #[test]
    fn content_line_multi_line_with_tab() {
        let (rem, content_line) =
            parse_line(b"DESCRIPTION:This is a lo\r\n ng description\r\n\t that exists on a long line.\r\nnext")
                .unwrap();
        check_rem(rem, 4);
        assert_eq!(b"DESCRIPTION", content_line.property_name);
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
