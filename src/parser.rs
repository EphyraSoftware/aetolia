use nom::branch::alt;
use nom::bytes::complete::{take_until, take_while1};
use nom::bytes::streaming::tag;
use nom::character::streaming::{alphanumeric1, char, crlf};
use nom::combinator::{opt, recognize};
use nom::error::{ErrorKind, ParseError};
use nom::multi::separated_list1;
use nom::sequence::{separated_pair, tuple};
use nom::{AsChar, IResult, Parser};

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

#[derive(Debug, Clone)]
struct ContentLine<'a> {
    property_name: &'a [u8],
    value: Vec<u8>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Param {
    name: String,
    value: ParamValue,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum ParamValue {
    AltRep { uri: String },
}

/// All ASCII control characters except tab (%x09).
#[inline]
pub const fn is_control(b: u8) -> bool {
    matches!(b, b'\0'..=b'\x08' | b'\x0A'..=b'\x1F' | b'\x7F')
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

fn param(input: &[u8]) -> IResult<&[u8], Option<Param>, Error> {
    let (input, (name, _)) = tuple((param_name, char('=')))(input)?;

    let name_s = read_string(name, "param_name")?;
    let (input, maybe_param_value) = match name_s.as_str() {
        "ALTREP" => {
            let (input, (_, uri, _)) = tuple((char('"'), take_until("\""), char('"')))(input)?;

            (
                input,
                Some(ParamValue::AltRep {
                    uri: read_string(&uri, "uri")?,
                }),
            )
        }
        _ => {
            // TODO not robust! Check 3
            let (input, _) = take_until(";")(input)?;

            (input, None)
        }
    };

    Ok((
        input,
        if let Some(param_value) = maybe_param_value {
            Some(Param {
                name: name_s,
                value: param_value,
            })
        } else {
            None
        },
    ))
}

fn read_string<'a>(input: &'a[u8], context: &str) -> Result<String, nom::Err<Error<'a>>> {
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
        parts.iter().map(|c| c).fold(vec![], |mut acc, x| {
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
    fn param_altrep() {
        let (rem, param) = param(b"ALTREP=\"http://example.com/calendar\";").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
        assert_eq!("ALTREP", param.name);
        assert_eq!(
            ParamValue::AltRep {
                uri: "http://example.com/calendar".to_string()
            },
            param.value
        );
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

    fn check_rem(rem: &[u8], expected_len: usize) {
        if rem.len() != expected_len {
            let str = String::from_utf8(rem.to_vec()).unwrap();
            println!("rem: {str}");
        }
        assert_eq!(
            expected_len,
            rem.len(),
            "Remainder length should be {expected_len} but was {}",
            rem.len()
        );
    }
}
