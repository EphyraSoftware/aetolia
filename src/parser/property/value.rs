use crate::parser::property::uri::{param_value_uri, Uri};
use crate::parser::property::value_types::Date;
use crate::parser::property::{DateTime, Duration, Period, PeriodEnd, Time, UtcOffset};
use crate::parser::{Error, InnerError};
use crate::utf8_seq;
use nom::branch::alt;
use nom::bytes::complete::take_while1;
use nom::bytes::streaming::{tag, tag_no_case, take_while_m_n};
use nom::character::is_digit;
use nom::character::streaming::{char, one_of};
use nom::combinator::{opt, recognize};
use nom::error::ParseError;
use nom::multi::{fold_many0, many0};
use nom::sequence::tuple;
use nom::IResult;
use nom::Parser;

#[inline]
const fn is_base64(c: u8) -> bool {
    matches!(c, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'+' | b'/')
}

pub fn prop_value_binary<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8], E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, content) = recognize(tuple((
        many0(take_while_m_n(4, 4, is_base64)),
        opt(alt((
            tuple((take_while_m_n(2, 2, is_base64), tag("=="))),
            tuple((take_while_m_n(3, 3, is_base64), tag("="))),
        ))),
    )))(input)?;

    Ok((input, content))
}

pub fn prop_value_boolean<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], bool, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, value) = alt((
        tag_no_case("TRUE").map(|_| true),
        tag_no_case("FALSE").map(|_| false),
    ))(input)?;

    Ok((input, value))
}

pub fn prop_value_calendar_user_address<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Uri<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, uri) = param_value_uri(input)?;

    Ok((input, uri))
}

pub fn prop_value_date<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Date, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (year, month, day)) = tuple((
        take_while_m_n(4, 4, is_digit),
        take_while_m_n(2, 2, is_digit),
        take_while_m_n(2, 2, is_digit),
    ))(input)?;

    let year = std::str::from_utf8(year)
        .map_err(|e| {
            nom::Err::Error(
                Error::new(
                    input,
                    InnerError::EncodingError("Invalid date year text".to_string(), e),
                )
                .into(),
            )
        })?
        .parse()
        .map_err(|_| nom::Err::Error(Error::new(input, InnerError::InvalidDateNum).into()))?;

    let month = std::str::from_utf8(month)
        .map_err(|e| {
            nom::Err::Error(
                Error::new(
                    input,
                    InnerError::EncodingError("Invalid date month text".to_string(), e),
                )
                .into(),
            )
        })?
        .parse()
        .map_err(|_| nom::Err::Error(Error::new(input, InnerError::InvalidDateNum).into()))?;

    let day = std::str::from_utf8(day)
        .map_err(|e| {
            nom::Err::Error(
                Error::new(
                    input,
                    InnerError::EncodingError("Invalid date day text".to_string(), e),
                )
                .into(),
            )
        })?
        .parse()
        .map_err(|_| nom::Err::Error(Error::new(input, InnerError::InvalidDateNum).into()))?;

    Ok((input, Date { year, month, day }))
}

pub fn prop_value_time<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Time, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (h, m, s, is_utc)) = tuple((
        take_while_m_n(2, 2, is_digit),
        take_while_m_n(2, 2, is_digit),
        take_while_m_n(2, 2, is_digit),
        opt(char('Z')).map(|x| x.is_some()),
    ))(input)?;

    let read_time = |s: &[u8]| -> Result<u8, Error> {
        std::str::from_utf8(s)
            .map_err(|e| {
                Error::new(
                    input,
                    InnerError::EncodingError("Invalid time text".to_string(), e),
                )
            })?
            .parse()
            .map_err(|_| Error::new(input, InnerError::InvalidTimeNum))
    };

    Ok((
        input,
        Time {
            hour: read_time(h).map_err(|e| nom::Err::Error(e.into()))?,
            minute: read_time(m).map_err(|e| nom::Err::Error(e.into()))?,
            second: read_time(s).map_err(|e| nom::Err::Error(e.into()))?,
            is_utc,
        },
    ))
}

pub fn prop_value_date_time<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], DateTime, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (date, _, time)) = tuple((prop_value_date, char('T'), prop_value_time))(input)?;

    Ok((input, DateTime { date, time }))
}

pub fn duration_num<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], u64, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, v) = take_while1(is_digit)(input)?;

    let s = std::str::from_utf8(v).map_err(|e| {
        nom::Err::Error(
            Error::new(
                input,
                InnerError::EncodingError("Invalid duration number text".to_string(), e),
            )
            .into(),
        )
    })?;

    Ok((
        input,
        s.parse().map_err(|_| {
            nom::Err::Error(Error::new(input, InnerError::InvalidDurationNum).into())
        })?,
    ))
}

fn duration_time<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], u64, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
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

fn opt_sign<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], i8, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    opt(alt((char('+'), char('-'))))
        .map(|x| {
            match x {
                Some('-') => -1,
                None | Some('+') => 1,
                // This is unreachable because of the alt combinator
                _ => unreachable!(),
            }
        })
        .parse(input)
}

pub fn prop_value_duration<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Duration, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (sign, _)) = tuple((opt_sign, char('P')))(input)?;

    let (input, t) = opt(char('T'))(input)?;

    if t.is_some() {
        let (input, t) = duration_time(input)?;

        return Ok((
            input,
            Duration {
                sign,
                seconds: t,
                ..Default::default()
            },
        ));
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

pub fn prop_value_float<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], f64, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (sign, num)) = tuple((
        opt_sign,
        recognize(tuple((
            take_while1(is_digit),
            opt(tuple((char('.'), take_while1(is_digit)))),
        ))),
    ))(input)?;

    let num: f64 = std::str::from_utf8(num)
        .map_err(|e| {
            nom::Err::Error(
                Error::new(
                    input,
                    InnerError::EncodingError("Invalid float number text".to_string(), e),
                )
                .into(),
            )
        })?
        .parse()
        .map_err(|_| nom::Err::Error(Error::new(input, InnerError::InvalidFloatNum).into()))?;

    Ok((input, sign as f64 * num))
}

pub fn prop_value_integer<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], i32, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (sign, num)) = tuple((opt_sign, take_while1(is_digit)))(input)?;

    let num: i32 = std::str::from_utf8(num)
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
        .map_err(|_| nom::Err::Error(Error::new(input, InnerError::InvalidIntegerNum).into()))?;

    Ok((input, sign as i32 * num))
}

#[inline]
const fn is_iso_8601_basic(c: u8) -> bool {
    matches!(c, b'0'..=b'9' | b'T' | b'Z' | b'-' | b'+' | b':')
}

fn iso_8601_basic<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8], E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    take_while_m_n(1, 21, is_iso_8601_basic)(input)
}

pub fn prop_value_period<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Period<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (start, _, end)) = tuple((
        iso_8601_basic,
        char('/'),
        alt((
            prop_value_duration.map(PeriodEnd::Duration),
            iso_8601_basic.map(PeriodEnd::DateTime),
        )),
    ))(input)?;

    Ok((input, Period { start, end }))
}

#[inline]
const fn is_text_safe_char(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\x21' | b'\x23'..=b'\x2B' | b'\x2D'..=b'\x39' | b'\x3C'..=b'\x5B' | b'\x5D'..=b'\x7E')
}

pub fn prop_value_text<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Vec<u8>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, r) = fold_many0(
        alt((
            // Escaped characters
            tuple((
                char('\\'),
                alt((tag_no_case("n").map(|_| b'\n' as char), one_of(r#"\;,"#))),
            ))
            .map(|(_, c)| vec![c as u8]),
            // Allowed raw characters
            one_of(r#":""#).map(|c: char| vec![c as u8]),
            // Text split over multiple lines
            tuple((tag("\r\n"), alt((char(' '), char('\t'))))).map(|_| Vec::with_capacity(0)),
            // UTF-8 sequence
            utf8_seq.map(|seq| seq.to_vec()),
            // Other text safe characters
            take_while1(is_text_safe_char).map(|section: &[u8]| section.to_vec()),
        )),
        Vec::new,
        |mut acc, item| {
            acc.extend_from_slice(&item);
            acc
        },
    )(input)?;

    Ok((input, r))
}

fn prop_value_uri<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Uri<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, uri, _)) = tuple((char('"'), param_value_uri, char('"')))(input)?;

    Ok((input, uri))
}

pub fn prop_value_utc_offset<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], UtcOffset, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (sign, h, m, s)) = tuple((
        one_of("+-"),
        take_while_m_n(2, 2, is_digit),
        take_while_m_n(2, 2, is_digit),
        opt(take_while_m_n(2, 2, is_digit)),
    ))(input)?;

    Ok((
        input,
        UtcOffset {
            sign: if sign == '+' { 1 } else { -1 },
            hours: std::str::from_utf8(h).unwrap().parse().unwrap(),
            minutes: std::str::from_utf8(m).unwrap().parse().unwrap(),
            seconds: s.map(|s| std::str::from_utf8(s).unwrap().parse().unwrap()),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::property::uri::Host;
    use crate::test_utils::check_rem;
    use base64::Engine;

    #[test]
    fn base64() {
        let (rem, value) =
            prop_value_binary::<Error>(b"VGhpcyBpcyBhIGJhc2U2NCBlbmNvZGluZyB0ZXh0;").unwrap();
        check_rem(rem, 1);
        let r = base64::prelude::BASE64_STANDARD.decode(value).unwrap();
        assert_eq!(b"This is a base64 encoding text", r.as_slice());
    }

    #[test]
    fn boolean() {
        let (rem, value) = prop_value_boolean::<Error>(b"TRUE;").unwrap();
        check_rem(rem, 1);
        assert!(value);
    }

    #[test]
    fn boolean_lower() {
        let (rem, value) = prop_value_boolean::<Error>(b"true;").unwrap();
        check_rem(rem, 1);
        assert!(value);
    }

    #[test]
    fn calendar_user_address() {
        let (rem, value) =
            prop_value_calendar_user_address::<Error>(b"mailto:jane_doe@example.com`").unwrap();
        check_rem(rem, 1);
        assert_eq!(value.scheme, b"mailto");
        assert_eq!(value.path, b"jane_doe@example.com")
    }

    #[test]
    fn date() {
        let (rem, value) = prop_value_date::<Error>(b"19970714;").unwrap();
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
    fn time() {
        let (rem, value) = prop_value_time::<Error>(b"230000;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            Time {
                hour: 23,
                minute: 0,
                second: 0,
                is_utc: false
            },
            value
        );
    }

    #[test]
    fn time_utc() {
        let (rem, value) = prop_value_time::<Error>(b"133000Z;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            Time {
                hour: 13,
                minute: 30,
                second: 0,
                is_utc: true
            },
            value
        );
    }

    #[test]
    fn date_time() {
        let (rem, value) = prop_value_date_time::<Error>(b"19980118T230000;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            DateTime {
                date: Date {
                    year: 1998,
                    month: 1,
                    day: 18
                },
                time: Time {
                    hour: 23,
                    minute: 0,
                    second: 0,
                    is_utc: false
                }
            },
            value
        );
    }

    #[test]
    fn duration_seven_weeks() {
        let (rem, value) = prop_value_duration::<Error>(b"P7W;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            Duration {
                weeks: 7,
                ..Default::default()
            },
            value
        );
    }

    #[test]
    fn duration_date_and_time() {
        let (rem, value) = prop_value_duration::<Error>(b"P15DT5H0M20S;").unwrap();
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
        let (rem, value) = prop_value_duration::<Error>(b"-PT10M20S;").unwrap();
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

    #[test]
    fn float() {
        let (rem, value) = prop_value_float::<Error>(b"1000000.0000001;").unwrap();
        check_rem(rem, 1);
        assert_eq!(1000000.0000001f64, value);
    }

    #[test]
    fn float_negative() {
        let (rem, value) = prop_value_float::<Error>(b"-1.333;").unwrap();
        check_rem(rem, 1);
        assert_eq!(-1.333, value);
    }

    #[test]
    fn integer() {
        let (rem, value) = prop_value_integer::<Error>(b"1234567890;").unwrap();
        check_rem(rem, 1);
        assert_eq!(1234567890, value);
    }

    #[test]
    fn integer_negative() {
        let (rem, value) = prop_value_integer::<Error>(b"-1234567890;").unwrap();
        check_rem(rem, 1);
        assert_eq!(-1234567890, value);
    }

    #[test]
    fn period() {
        let (rem, value) =
            prop_value_period::<Error>(b"19970101T180000Z/19970102T070000Z;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            Period {
                start: b"19970101T180000Z",
                end: PeriodEnd::DateTime(b"19970102T070000Z")
            },
            value
        );
    }

    #[test]
    fn period_duration() {
        let (rem, value) = prop_value_period::<Error>(b"19970101T180000Z/PT5H30M;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            Period {
                start: b"19970101T180000Z",
                end: PeriodEnd::Duration(Duration {
                    seconds: 5 * 60 * 60 + 30 * 60,
                    ..Default::default()
                })
            },
            value
        );
    }

    #[test]
    fn text() {
        let (rem, value) = prop_value_text::<Error>(
            br#"Project XYZ Final Review\nConference Room - 3B\nCome Prepared.;"#,
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            br#"Project XYZ Final Review
Conference Room - 3B
Come Prepared."#,
            value.as_slice()
        );
    }

    #[test]
    fn text_with_quoted_value() {
        let (rem, value) = prop_value_text::<Error>(br#"Hello\, "World";"#).unwrap();
        println!("{:?}", String::from_utf8(value.clone()).unwrap());
        check_rem(rem, 1);
        assert_eq!(br#"Hello, "World""#, value.as_slice());
    }

    #[test]
    fn uri() {
        let (rem, value) =
            prop_value_uri::<Error>(b"\"http://example.com/my-report.txt\";").unwrap();
        check_rem(rem, 1);
        assert_eq!(value.scheme, b"http");
        assert_eq!(
            value.authority.unwrap().host,
            Host::RegName(b"example.com".to_vec())
        );
        assert_eq!(value.path, b"/my-report.txt");
    }

    #[test]
    fn utc_offset_negative() {
        let (rem, value) = prop_value_utc_offset::<Error>(b"-0500;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            UtcOffset {
                sign: -1,
                hours: 5,
                minutes: 0,
                seconds: None
            },
            value
        );
    }

    #[test]
    fn utc_offset_positive() {
        let (rem, value) = prop_value_utc_offset::<Error>(b"+0130;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            UtcOffset {
                sign: 1,
                hours: 1,
                minutes: 30,
                seconds: None
            },
            value
        );
    }
}
