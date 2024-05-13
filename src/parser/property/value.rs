use crate::parser::property::types::Date;
use crate::parser::property::Duration;
use crate::parser::{Error, InnerError};
use nom::branch::alt;
use nom::bytes::complete::take_while1;
use nom::bytes::streaming::{tag, tag_no_case, take_while_m_n};
use nom::character::complete::one_of;
use nom::character::is_digit;
use nom::character::streaming::char;
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

pub fn prop_value_date(input: &[u8]) -> IResult<&[u8], Date, Error> {
    let (input, (year, month, day)) = tuple((
        take_while_m_n(4, 4, is_digit),
        take_while_m_n(2, 2, is_digit),
        take_while_m_n(2, 2, is_digit),
    ))(input)?;

    let year = std::str::from_utf8(year).map_err(|e| {
        nom::Err::Error(Error::new(
            input,
            InnerError::EncodingError("Invalid date year text".to_string(), e),
        ))
    })?.parse().map_err(|_| nom::Err::Error(Error::new(input, InnerError::InvalidDateNum)))?;


    let month = std::str::from_utf8(month).map_err(|e| {
        nom::Err::Error(Error::new(
            input,
            InnerError::EncodingError("Invalid date month text".to_string(), e),
        ))
    })?.parse().map_err(|_| nom::Err::Error(Error::new(input, InnerError::InvalidDateNum)))?;

    let day = std::str::from_utf8(day).map_err(|e| {
        nom::Err::Error(Error::new(
            input,
            InnerError::EncodingError("Invalid date day text".to_string(), e),
        ))
    })?.parse().map_err(|_| nom::Err::Error(Error::new(input, InnerError::InvalidDateNum)))?;

    Ok((
        input,
        Date {
            year,
            month,
            day,
        },
    ))
}

pub fn duration_num(input: &[u8]) -> IResult<&[u8], u64, Error> {
    let (input, v) = take_while1(is_digit)(input)?;

    let s = std::str::from_utf8(v).map_err(|e| {
        nom::Err::Error(Error::new(
            input,
            InnerError::EncodingError("Invalid duration number text".to_string(), e),
        ))
    })?;

    Ok((
        input,
        s.parse()
            .map_err(|_| nom::Err::Error(Error::new(input, InnerError::InvalidDurationNum)))?,
    ))
}

fn duration_time(input: &[u8]) -> IResult<&[u8], u64, Error> {
    let (input, num) = duration_num(input)?;

    let (input, time_branch) = one_of("HMS")(input)?;

    match time_branch {
        'H' => {
            let (input, (min, sec)) = tuple((
                opt(tuple((duration_num, char('M'))))
                    .map(|min| min.map(|(min, _)| min).unwrap_or(0)),
                opt(tuple((duration_num, char('S'))))
                    .map(|min| min.map(|(min, _)| min).unwrap_or(0)),
            ))(input)?;

            Ok((input, num * 60 * 60 + min * 60 + sec))
        }
        'M' => {
            let (input, sec) = opt(tuple((duration_num, char('S'))))(input)?;

            let sec = if let Some((sec, _)) = sec { sec } else { 0 };

            Ok((input, num * 60 + sec))
        }
        'S' => Ok((input, num)),
        // This is unreachable because of the one_of combinator
        _ => unreachable!(),
    }
}

pub fn prop_value_duration(input: &[u8]) -> IResult<&[u8], Duration, Error> {
    let (input, (sign, _)) = tuple((
        opt(alt((char('+'), char('-')))).map(|x| {
            match x {
                Some('-') => -1,
                None | Some('+') => 1,
                // This is unreachable because of the alt combinator
                _ => unreachable!(),
            }
        }),
        char('P'),
    ))(input)?;

    let (input, t) = opt(char('T'))(input)?;

    if let Some(_) = t {
        let (input, t) = duration_time(input)?;

        return Ok((input, Duration {
            sign,
            seconds: t,
            ..Default::default()
        }));
    };

    let (input, num) = duration_num(input)?;

    let (input, date_branch) = one_of("DW")(input)?;

    match date_branch {
        'D' => {
            let (input, seconds) = opt(tuple((char('T'), duration_time)))(input)?;

            let seconds = if let Some((_, seconds)) = seconds {
                seconds
            } else {
                0
            };

            Ok((
                input,
                Duration {
                    sign,
                    days: num,
                    seconds,
                    ..Default::default()
                },
            ))
        }
        'W' => Ok((
            input,
            Duration {
                sign,
                weeks: num,
                ..Default::default()
            },
        )),
        // This is unreachable because of the one_of combinator
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::check_rem;
    use base64::Engine;

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

    #[test]
    fn date() {
        let (rem, value) = prop_value_date(b"19970714;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            Date {
                year: 1997,
                month: 7,
                day: 14
            },
            value
        );
    }

    #[test]
    fn duration_seven_weeks() {
        let (rem, value) = prop_value_duration(b"P7W;").unwrap();
        check_rem(rem, 1);
        assert_eq!(Duration {
            weeks: 7,
            ..Default::default()
        }, value);
    }

    #[test]
    fn duration_date_and_time() {
        let (rem, value) = prop_value_duration(b"P15DT5H0M20S;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            Duration {
                days: 15,
                seconds: 5 * 60 * 60 + 20,
                ..Default::default()
            },
            value
        );
    }

    #[test]
    fn duration_signed_time() {
        let (rem, value) = prop_value_duration(b"-PT10M20S;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            Duration {
                sign: -1,
                seconds: 10 * 60 + 20,
                ..Default::default()
            },
            value
        );
    }
}
