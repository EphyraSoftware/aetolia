use crate::parser::Error;
use nom::branch::alt;
use nom::bytes::streaming::{tag, tag_no_case, take_while_m_n};
use nom::combinator::{opt, recognize};
use nom::multi::many0;
use nom::sequence::tuple;
use nom::IResult;
use nom::Parser;

#[inline]
const fn is_base64(c: u8) -> bool {
    matches!(c, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'+' | b'/')
}

pub fn prop_value_base64(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    let (input, content) = recognize(tuple((
        many0(take_while_m_n(4, 4, is_base64)),
        opt(alt((tag("=="), tag("=")))),
    )))(input)?;

    Ok((input, content))
}

pub fn prop_value_boolean(input: &[u8]) -> IResult<&[u8], bool, Error> {
    let (input, value) = alt((
        tag_no_case("TRUE").map(|_| true),
        tag_no_case("FALSE").map(|_| false),
    ))(input)?;

    Ok((input, value))
}

#[cfg(test)]
mod tests {
    use base64::Engine;
    use super::*;
    use crate::test_utils::check_rem;

    #[test]
    fn base64() {
        let (rem, value) = prop_value_base64(b"VGhpcyBpcyBhIGJhc2U2NCBlbmNvZGluZyB0ZXh0;").unwrap();
        check_rem(rem, 1);
        let r = base64::prelude::BASE64_STANDARD.decode(value).unwrap();
        assert_eq!(b"This is a base64 encoding text", r.as_slice());
    }

    #[test]
    fn boolean() {
        let (rem, value) = prop_value_boolean(b"TRUE;").unwrap();
        check_rem(rem, 1);
        assert!(value);
    }

    #[test]
    fn boolean_lower() {
        let (rem, value) = prop_value_boolean(b"true;").unwrap();
        check_rem(rem, 1);
        assert!(value);
    }
}
