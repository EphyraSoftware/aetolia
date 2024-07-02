use crate::common::{OffsetWeekday, RecurFreq, Weekday};
use crate::parser::property::{prop_value_date, prop_value_time, DateTime};
use crate::parser::{DateOrDateTime, Error, InnerError};
use nom::branch::alt;
use nom::bytes::complete::{take_while1, take_while_m_n};
use nom::bytes::streaming::tag;
use nom::character::streaming::char;
use nom::character::{is_alphabetic, is_digit};
use nom::combinator::{map_res, opt};
use nom::error::ParseError;
use nom::multi::separated_list1;
use nom::sequence::tuple;
use nom::{IResult, Parser};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RecurRulePart {
    Freq(RecurFreq),
    Until(DateOrDateTime),
    Count(u64),
    Interval(u64),
    BySecList(Vec<u8>),
    ByMinute(Vec<u8>),
    ByHour(Vec<u8>),
    ByDay(Vec<OffsetWeekday>),
    ByMonthDay(Vec<i8>),
    ByYearDay(Vec<i16>),
    ByWeek(Vec<i8>),
    ByMonth(Vec<u8>),
    BySetPos(Vec<i16>),
    WeekStart(Weekday),
}

pub fn recur<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Vec<RecurRulePart>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    separated_list1(char(';'), recur_rule_part)(input)
}

fn recur_rule_part<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], RecurRulePart, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (name, _)) = tuple((take_while1(is_alphabetic), char('=')))(input)?;

    match std::str::from_utf8(name).map_err(|e| {
        nom::Err::Error(
            Error::new(
                input,
                InnerError::EncodingError("Recur part name".to_string(), e),
            )
            .into(),
        )
    })? {
        "FREQ" => recur_freq.map(RecurRulePart::Freq).parse(input),
        "UNTIL" => end_date.map(RecurRulePart::Until).parse(input),
        "COUNT" => read_num.map(RecurRulePart::Count).parse(input),
        "INTERVAL" => read_num.map(RecurRulePart::Interval).parse(input),
        "BYSECOND" => recur_by_time_list
            .map(RecurRulePart::BySecList)
            .parse(input),
        "BYMINUTE" => recur_by_time_list.map(RecurRulePart::ByMinute).parse(input),
        "BYHOUR" => recur_by_time_list.map(RecurRulePart::ByHour).parse(input),
        "BYDAY" => recur_by_weekday_list.map(RecurRulePart::ByDay).parse(input),
        "BYMONTHDAY" => recur_by_month_day_list
            .map(RecurRulePart::ByMonthDay)
            .parse(input),
        "BYYEARDAY" => recur_by_year_day_list
            .map(RecurRulePart::ByYearDay)
            .parse(input),
        "BYWEEKNO" => recur_by_week_number.map(RecurRulePart::ByWeek).parse(input),
        "BYMONTH" => recur_by_month_list.map(RecurRulePart::ByMonth).parse(input),
        "BYSETPOS" => recur_by_year_day_list
            .map(RecurRulePart::BySetPos)
            .parse(input),
        "WKST" => weekday.map(RecurRulePart::WeekStart).parse(input),
        n => Err(nom::Err::Error(
            Error::new(input, InnerError::InvalidRecurPart(n.to_string())).into(),
        )),
    }
}

fn recur_freq<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], RecurFreq, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, freq) = alt((
        tag("SECONDLY").map(|_| RecurFreq::Secondly),
        tag("MINUTELY").map(|_| RecurFreq::Minutely),
        tag("HOURLY").map(|_| RecurFreq::Hourly),
        tag("DAILY").map(|_| RecurFreq::Daily),
        tag("WEEKLY").map(|_| RecurFreq::Weekly),
        tag("MONTHLY").map(|_| RecurFreq::Monthly),
        tag("YEARLY").map(|_| RecurFreq::Yearly),
    ))(input)?;

    Ok((input, freq))
}

fn end_date<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], DateOrDateTime, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (date, opt_time)) =
        tuple((prop_value_date, opt(tuple((char('T'), prop_value_time)))))(input)?;

    let time = opt_time.map(|(_, time)| time);

    Ok((
        input,
        match time {
            Some(time) => DateOrDateTime::DateTime(DateTime { date, time }),
            None => DateOrDateTime::Date(date),
        },
    ))
}

fn read_num<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], u64, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, c) = take_while1(is_digit)(input)?;

    let v = std::str::from_utf8(c).map_err(|e| {
        nom::Err::Error(
            Error::new(input, InnerError::EncodingError("Recur num".to_string(), e)).into(),
        )
    })?;

    Ok((
        input,
        v.parse()
            .map_err(|_| nom::Err::Error(Error::new(input, InnerError::InvalidRecurNum).into()))?,
    ))
}

fn recur_by_time_list<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Vec<u8>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    separated_list1(
        char(','),
        map_res(take_while_m_n(1, 2, is_digit), |s| {
            std::str::from_utf8(s)
                .map_err(|e| {
                    nom::Err::Error(
                        Error::new(
                            input,
                            InnerError::EncodingError("Recur time list".to_string(), e),
                        )
                        .into(),
                    )
                })?
                .parse()
                .map_err(|_| nom::Err::Error(Error::new(input, InnerError::InvalidRecurNum).into()))
        }),
    )(input)
}

fn weekday<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Weekday, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    alt((
        tag("MO").map(|_| Weekday::Monday),
        tag("TU").map(|_| Weekday::Tuesday),
        tag("WE").map(|_| Weekday::Wednesday),
        tag("TH").map(|_| Weekday::Thursday),
        tag("FR").map(|_| Weekday::Friday),
        tag("SA").map(|_| Weekday::Saturday),
        tag("SU").map(|_| Weekday::Sunday),
    ))(input)
}

fn recur_by_weekday_list<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Vec<OffsetWeekday>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    separated_list1(
        char(','),
        tuple((
            opt(map_res(
                tuple((
                    opt(alt((char('+'), char('-')))).map(|opt_sign| {
                        if let Some('-') = opt_sign {
                            -1i8
                        } else {
                            1
                        }
                    }),
                    take_while_m_n(1, 2, is_digit),
                )),
                |(sign, num)| {
                    std::str::from_utf8(num)
                        .map_err(|e| {
                            nom::Err::Error(
                                Error::new(
                                    input,
                                    InnerError::EncodingError("Recur weekday list".to_string(), e),
                                )
                                .into(),
                            )
                        })?
                        .parse::<i8>()
                        .map_err(|_| {
                            nom::Err::Error(Error::new(input, InnerError::InvalidRecurNum).into())
                        })
                        .map(|num| sign * num)
                },
            )),
            weekday,
        )),
    )
    .map(|values| {
        values
            .into_iter()
            .map(|(offset_weeks, weekday)| OffsetWeekday {
                offset_weeks,
                weekday,
            })
            .collect()
    })
    .parse(input)
}

fn recur_by_month_day_list<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Vec<i8>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    separated_list1(
        char(','),
        map_res(
            tuple((
                opt(alt((char('+'), char('-'))))
                    .map(|sign| if let Some('-') = sign { -1i8 } else { 1 }),
                take_while_m_n(1, 2, is_digit),
            )),
            |(sign, num)| {
                std::str::from_utf8(num)
                    .map_err(|e| {
                        nom::Err::Error(
                            Error::new(
                                input,
                                InnerError::EncodingError("Recur month day list".to_string(), e),
                            )
                            .into(),
                        )
                    })?
                    .parse::<i8>()
                    .map_err(|_| {
                        nom::Err::Error(Error::new(input, InnerError::InvalidRecurNum).into())
                    })
                    .map(|num| sign * num)
            },
        ),
    )(input)
}

fn recur_by_year_day_list<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Vec<i16>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    separated_list1(
        char(','),
        map_res(
            tuple((
                opt(alt((char('+'), char('-'))))
                    .map(|sign| if let Some('-') = sign { -1i16 } else { 1 }),
                take_while_m_n(1, 3, is_digit),
            )),
            |(sign, num)| {
                std::str::from_utf8(num)
                    .map_err(|e| {
                        nom::Err::Error(
                            Error::new(
                                input,
                                InnerError::EncodingError("Recur year day list".to_string(), e),
                            )
                            .into(),
                        )
                    })?
                    .parse::<i16>()
                    .map_err(|_| {
                        nom::Err::Error(Error::new(input, InnerError::InvalidRecurNum).into())
                    })
                    .map(|num| sign * num)
            },
        ),
    )(input)
}

fn recur_by_week_number<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Vec<i8>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    separated_list1(
        char(','),
        map_res(
            tuple((
                opt(alt((char('+'), char('-'))))
                    .map(|sign| if let Some('-') = sign { -1i8 } else { 1 }),
                take_while_m_n(1, 2, is_digit),
            )),
            |(sign, num)| {
                std::str::from_utf8(num)
                    .map_err(|e| {
                        nom::Err::Error(
                            Error::new(
                                input,
                                InnerError::EncodingError("Recur week number list".to_string(), e),
                            )
                            .into(),
                        )
                    })?
                    .parse::<i8>()
                    .map_err(|_| {
                        nom::Err::Error(Error::new(input, InnerError::InvalidRecurNum).into())
                    })
                    .map(|num| sign * num)
            },
        ),
    )(input)
}

fn recur_by_month_list<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Vec<u8>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    separated_list1(
        char(','),
        map_res(take_while_m_n(1, 2, is_digit), |num| {
            std::str::from_utf8(num)
                .map_err(|e| {
                    nom::Err::Error(
                        Error::new(
                            input,
                            InnerError::EncodingError("Recur month list".to_string(), e),
                        )
                        .into(),
                    )
                })?
                .parse::<u8>()
                .map_err(|_| nom::Err::Error(Error::new(input, InnerError::InvalidRecurNum).into()))
        }),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::check_rem;

    #[test]
    fn daily_rule() {
        let (rem, rule) = recur::<Error>(b"FREQ=DAILY;COUNT=10;INTERVAL=2;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            rule,
            vec![
                RecurRulePart::Freq(RecurFreq::Daily),
                RecurRulePart::Count(10),
                RecurRulePart::Interval(2)
            ]
        );
    }

    #[test]
    fn monthly_rule() {
        let (rem, rule) =
            recur::<Error>(b"FREQ=MONTHLY;BYDAY=MO,TU,WE,TH,FR;BYSETPOS=-1;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            rule,
            vec![
                RecurRulePart::Freq(RecurFreq::Monthly),
                RecurRulePart::ByDay(vec![
                    OffsetWeekday {
                        offset_weeks: None,
                        weekday: Weekday::Monday
                    },
                    OffsetWeekday {
                        offset_weeks: None,
                        weekday: Weekday::Tuesday
                    },
                    OffsetWeekday {
                        offset_weeks: None,
                        weekday: Weekday::Wednesday
                    },
                    OffsetWeekday {
                        offset_weeks: None,
                        weekday: Weekday::Thursday
                    },
                    OffsetWeekday {
                        offset_weeks: None,
                        weekday: Weekday::Friday
                    }
                ]),
                RecurRulePart::BySetPos(vec![-1])
            ]
        );
    }

    #[test]
    fn yearly_rule() {
        let (rem, rule) =
            recur::<Error>(b"FREQ=YEARLY;INTERVAL=2;BYMONTH=1;BYDAY=SU;BYHOUR=8,9;BYMINUTE=30;")
                .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            rule,
            vec![
                RecurRulePart::Freq(RecurFreq::Yearly),
                RecurRulePart::Interval(2),
                RecurRulePart::ByMonth(vec![1]),
                RecurRulePart::ByDay(vec![OffsetWeekday {
                    offset_weeks: None,
                    weekday: Weekday::Sunday
                }]),
                RecurRulePart::ByHour(vec![8, 9]),
                RecurRulePart::ByMinute(vec![30]),
            ]
        );
    }
}
