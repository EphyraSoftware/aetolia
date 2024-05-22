use nom::branch::alt;
use nom::bytes::streaming::tag;
use nom::character::streaming::char;
use nom::{IResult, Parser};
use nom::sequence::tuple;
use crate::parser::Error;
use crate::parser::param::{other_params, Param, params};
use crate::parser::property::{DateOrDateTime, DateTime, prop_value_date, prop_value_date_time};

#[derive(Debug, Eq, PartialEq)]
pub struct DateTimeStartProperty<'a> {
    pub params: Vec<Param<'a>>,
    pub value: DateOrDateTime,
}

pub fn prop_date_time_start(input: &[u8]) -> IResult<&[u8], DateTimeStartProperty, Error> {
    let (input, (_, params, _, value, _)) = tuple((
        tag("DTSTART"),
        params,
        char(':'),
        alt((
            prop_value_date_time.map(DateOrDateTime::DateTime),
            prop_value_date.map(DateOrDateTime::Date)
            )),
        tag("\r\n"),
    ))(input)?;

    Ok((
        input,
        DateTimeStartProperty {
            params,
            value,
        },
    ))
}

#[derive(Debug, Eq, PartialEq)]
pub struct DateTimeStamp<'a> {
    pub other_params: Vec<Param<'a>>,
    pub value: DateTime,
}

pub fn prop_date_time_stamp(input: &[u8]) -> IResult<&[u8], DateTimeStamp, Error> {
    let (input, (_, other_params, _, value, _)) = tuple((
        tag("DTSTAMP"),
        other_params,
        char(':'),
        prop_value_date_time,
        tag("\r\n"),
    ))(input)?;

    Ok((
        input,
        DateTimeStamp {
            other_params,
            value,
        },
    ))
}

#[cfg(test)]
mod tests {
    use crate::parser::property::{Date, Time};
    use crate::test_utils::check_rem;
    use super::*;

    #[test]
    fn date_time_stamp() {
        let (rem, prop) = prop_date_time_stamp(b"DTSTAMP:19971210T080000Z\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(prop, DateTimeStamp {
            other_params: vec![],
            value: DateTime {
                date: Date {
                    year: 1997,
                    month: 12,
                    day: 10,
                },
                time: Time {
                    hour: 8,
                    minute: 0,
                    second: 0,
                    is_utc: true,
                },
            }
        });
    }
}
