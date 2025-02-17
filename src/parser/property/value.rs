use crate::parser::property::uri::param_value_uri;
use crate::parser::types::{Date, DateTime, Duration, Period, PeriodEnd, Time, Uri, UtcOffset};
use crate::parser::{read_int, Error, InnerError};
use crate::utf8_seq;
use nom::branch::alt;
use nom::bytes::complete::take_while1;
use nom::bytes::streaming::{tag, tag_no_case, take_while_m_n};
use nom::character::streaming::{char, one_of};
use nom::combinator::{opt, recognize};
use nom::error::ParseError;
use nom::multi::{fold_many0, many0};
use nom::Parser;
use nom::{AsChar, IResult};

#[inline]
const fn is_base64(c: u8) -> bool {
    matches!(c, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'+' | b'/')
}

pub fn prop_value_binary<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8], E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, content) = recognize((
        many0(take_while_m_n(4, 4, is_base64)),
        opt(alt((
            (take_while_m_n(2, 2, is_base64), tag("==")),
            (take_while_m_n(3, 3, is_base64), tag("=")),
        ))),
    ))
    .parse(input)?;

    Ok((input, content))
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
    let (input, (year, month, day)) = (
        take_while_m_n(4, 4, AsChar::is_dec_digit),
        take_while_m_n(2, 2, AsChar::is_dec_digit),
        take_while_m_n(2, 2, AsChar::is_dec_digit),
    )
        .parse(input)?;

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
    let (input, (h, m, s, is_utc)) = (
        take_while_m_n(2, 2, AsChar::is_dec_digit),
        take_while_m_n(2, 2, AsChar::is_dec_digit),
        take_while_m_n(2, 2, AsChar::is_dec_digit),
        opt(char('Z')).map(|x| x.is_some()),
    )
        .parse(input)?;

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
    let (input, (date, _, time)) = (prop_value_date, char('T'), prop_value_time).parse(input)?;

    Ok((input, DateTime { date, time }))
}

fn duration_num<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], u64, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, v) = take_while1(AsChar::is_dec_digit)(input)?;

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

type HoursMinutesSeconds = (Option<u64>, Option<u64>, Option<u64>);

fn duration_time<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], HoursMinutesSeconds, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, num) = duration_num(input)?;

    let (input, time_branch) = one_of("HMS")(input)?;

    match time_branch {
        'H' => {
            let (input, (min, sec)) = (
                opt((duration_num, char('M')).map(|(min, _)| min)),
                opt((duration_num, char('S')).map(|(sec, _)| sec)),
            )
                .parse(input)?;

            Ok((input, (Some(num), min, sec)))
        }
        'M' => {
            let (input, sec) = opt((duration_num, char('S')).map(|(sec, _)| sec)).parse(input)?;

            Ok((input, (None, Some(num), sec)))
        }
        'S' => Ok((input, (None, None, Some(num)))),
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
    let (input, (sign, _)) = (opt_sign, char('P')).parse(input)?;

    let (input, t) = opt(char('T')).parse(input)?;

    if t.is_some() {
        let (input, (hours, minutes, seconds)) = duration_time(input)?;

        return Ok((
            input,
            Duration {
                sign,
                hours,
                minutes,
                seconds,
                ..Default::default()
            },
        ));
    };

    let (input, num) = duration_num(input)?;

    let (input, date_branch) = one_of("DW")(input)?;

    match date_branch {
        'D' => {
            let (input, time) = opt((char('T'), duration_time))
                .map(|opt| opt.map(|(_, t)| t))
                .parse(input)?;

            let (hours, minutes, seconds) = time.unwrap_or((None, None, None));

            Ok((
                input,
                Duration {
                    sign,
                    days: Some(num),
                    hours,
                    minutes,
                    seconds,
                    ..Default::default()
                },
            ))
        }
        'W' => Ok((
            input,
            Duration {
                sign,
                weeks: Some(num),
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
    let (input, (sign, num)) = (
        opt_sign,
        recognize((
            take_while1(AsChar::is_dec_digit),
            opt((char('.'), take_while1(AsChar::is_dec_digit))),
        )),
    )
        .parse(input)?;

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
    let (input, (sign, num)) = (opt_sign, take_while1(AsChar::is_dec_digit)).parse(input)?;

    let num: i32 = read_int(num)?;

    Ok((input, sign as i32 * num))
}

pub fn prop_value_period<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Period, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (start, _, end)) = (
        prop_value_date_time,
        char('/'),
        alt((
            prop_value_duration.map(PeriodEnd::Duration),
            prop_value_date_time.map(PeriodEnd::DateTime),
        )),
    )
        .parse(input)?;

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
            (
                char('\\'),
                alt((tag_no_case("n").map(|_| b'\n' as char), one_of(r#"\;,"#))),
            )
                .map(|(_, c)| vec![c as u8]),
            // Allowed raw characters
            one_of(r#":""#).map(|c: char| vec![c as u8]),
            // Text split over multiple lines
            (tag("\r\n"), alt((char(' '), char('\t')))).map(|_| Vec::with_capacity(0)),
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
    )
    .parse(input)?;

    Ok((input, r))
}

pub fn prop_value_utc_offset<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], UtcOffset, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (sign, h, m, s)) = (
        one_of("+-"),
        take_while_m_n(2, 2, AsChar::is_dec_digit),
        take_while_m_n(2, 2, AsChar::is_dec_digit),
        opt(take_while_m_n(2, 2, AsChar::is_dec_digit)),
    )
        .parse(input)?;

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
                weeks: Some(7),
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
                days: Some(15),
                hours: Some(5),
                minutes: Some(0),
                seconds: Some(20),
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
                minutes: Some(10),
                seconds: Some(20),
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
                start: DateTime {
                    date: Date {
                        year: 1997,
                        month: 1,
                        day: 1
                    },
                    time: Time {
                        hour: 18,
                        minute: 0,
                        second: 0,
                        is_utc: true
                    }
                },
                end: PeriodEnd::DateTime(DateTime {
                    date: Date {
                        year: 1997,
                        month: 1,
                        day: 2
                    },
                    time: Time {
                        hour: 7,
                        minute: 0,
                        second: 0,
                        is_utc: true
                    }
                })
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
                start: DateTime {
                    date: Date {
                        year: 1997,
                        month: 1,
                        day: 1
                    },
                    time: Time {
                        hour: 18,
                        minute: 0,
                        second: 0,
                        is_utc: true
                    }
                },
                end: PeriodEnd::Duration(Duration {
                    hours: Some(5),
                    minutes: Some(30),
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
