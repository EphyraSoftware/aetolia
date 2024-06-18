use crate::parser::object::types::{CalendarComponent, ComponentProperty};
use crate::parser::property::{
    prop_comment, prop_date_time_start, prop_iana, prop_last_modified, prop_recurrence_date_times,
    prop_recurrence_rule, prop_time_zone_id, prop_time_zone_name, prop_time_zone_offset_from,
    prop_time_zone_offset_to, prop_time_zone_url, prop_x,
};
use crate::parser::Error;
use nom::branch::alt;
use nom::bytes::streaming::tag;
use nom::combinator::cut;
use nom::error::ParseError;
use nom::multi::many0;
use nom::sequence::tuple;
use nom::IResult;
use nom::Parser;

pub fn component_timezone<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], CalendarComponent<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, properties, _)) = tuple((
        tag("BEGIN:VTIMEZONE\r\n"),
        cut(many0(alt((
            alt((
                prop_time_zone_id.map(ComponentProperty::TimeZoneId),
                prop_last_modified.map(ComponentProperty::LastModified),
                prop_time_zone_url.map(ComponentProperty::TimeZoneUrl),
                component_standard.map(ComponentProperty::Standard),
                component_daylight.map(ComponentProperty::Daylight),
            )),
            prop_x.map(ComponentProperty::XProp),
            prop_iana.map(ComponentProperty::IanaProp),
        )))),
        tag("END:VTIMEZONE\r\n"),
    ))(input)?;

    Ok((input, CalendarComponent::TimeZone { properties }))
}

pub fn component_standard<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], CalendarComponent<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, properties, _)) =
        tuple((tag("BEGIN:STANDARD\r\n"), cut(tz_props), tag("END:STANDARD\r\n")))(input)?;

    Ok((input, CalendarComponent::Standard { properties }))
}

pub fn component_daylight<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], CalendarComponent<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, properties, _)) =
        tuple((tag("BEGIN:DAYLIGHT\r\n"), cut(tz_props), tag("END:DAYLIGHT\r\n")))(input)?;

    Ok((input, CalendarComponent::Daylight { properties }))
}

fn tz_props<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Vec<ComponentProperty<'a>>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    many0(alt((
        alt((
            prop_date_time_start.map(ComponentProperty::DateTimeStart),
            prop_time_zone_offset_to.map(ComponentProperty::TimeZoneOffsetTo),
            prop_time_zone_offset_from.map(ComponentProperty::TimeZoneOffsetFrom),
            prop_recurrence_rule.map(ComponentProperty::RecurrenceRule),
            prop_comment.map(ComponentProperty::Comment),
            prop_recurrence_date_times.map(ComponentProperty::RecurrenceDateTimes),
            prop_time_zone_name.map(ComponentProperty::TimeZoneName),
        )),
        prop_x.map(ComponentProperty::XProp),
        prop_iana.map(ComponentProperty::IanaProp),
    )))(input)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::parser::property::{
        Date, DateOrDateTime, DateTime, DateTimeStartProperty, LastModifiedProperty, Time,
        TimeZoneIdProperty, TimeZoneNameProperty, TimeZoneOffsetProperty, UtcOffset,
    };
    use crate::parser::Error;
    use crate::test_utils::check_rem;

    #[test]
    fn test_component_timezone() {
        let input = b"BEGIN:VTIMEZONE\r\nTZID:America/New_York\r\nLAST-MODIFIED:20050809T050000Z\r\nBEGIN:STANDARD\r\nDTSTART:20071104T020000\r\nTZOFFSETFROM:-0400\r\nTZOFFSETTO:-0500\r\nTZNAME:EST\r\nEND:STANDARD\r\nBEGIN:DAYLIGHT\r\nDTSTART:20070311T020000\r\nTZOFFSETFROM:-0500\r\nTZOFFSETTO:-0400\r\nTZNAME:EDT\r\nEND:DAYLIGHT\r\nEND:VTIMEZONE\r\n";
        let (rem, component) = component_timezone::<Error>(input).unwrap();
        check_rem(rem, 0);

        match component {
            CalendarComponent::TimeZone { properties } => {
                assert_eq!(properties.len(), 4);

                assert_eq!(
                    properties[0],
                    ComponentProperty::TimeZoneId(TimeZoneIdProperty {
                        other_params: vec![],
                        value: b"America/New_York".to_vec(),
                    })
                );

                assert_eq!(
                    properties[1],
                    ComponentProperty::LastModified(LastModifiedProperty {
                        other_params: vec![],
                        value: DateTime {
                            date: Date {
                                year: 2005,
                                month: 8,
                                day: 9,
                            },
                            time: Time {
                                hour: 5,
                                minute: 0,
                                second: 0,
                                is_utc: true,
                            },
                        }
                    })
                );

                match &properties[2] {
                    ComponentProperty::Standard(CalendarComponent::Standard { properties }) => {
                        assert_eq!(properties.len(), 4);

                        assert_eq!(
                            properties[0],
                            ComponentProperty::DateTimeStart(DateTimeStartProperty {
                                params: vec![],
                                value: DateOrDateTime::DateTime(DateTime {
                                    date: Date {
                                        year: 2007,
                                        month: 11,
                                        day: 4,
                                    },
                                    time: Time {
                                        hour: 2,
                                        minute: 0,
                                        second: 0,
                                        is_utc: false,
                                    },
                                })
                            })
                        );

                        assert_eq!(
                            properties[1],
                            ComponentProperty::TimeZoneOffsetFrom(TimeZoneOffsetProperty {
                                other_params: vec![],
                                value: UtcOffset {
                                    sign: -1,
                                    hours: 4,
                                    minutes: 0,
                                    seconds: None,
                                }
                            })
                        );

                        assert_eq!(
                            properties[2],
                            ComponentProperty::TimeZoneOffsetTo(TimeZoneOffsetProperty {
                                other_params: vec![],
                                value: UtcOffset {
                                    sign: -1,
                                    hours: 5,
                                    minutes: 0,
                                    seconds: None,
                                }
                            })
                        );

                        assert_eq!(
                            properties[3],
                            ComponentProperty::TimeZoneName(TimeZoneNameProperty {
                                params: vec![],
                                value: b"EST".to_vec(),
                            })
                        );
                    }
                    _ => panic!("Unexpected component type"),
                }

                match &properties[3] {
                    ComponentProperty::Daylight(CalendarComponent::Daylight { properties }) => {
                        assert_eq!(properties.len(), 4);

                        assert_eq!(
                            properties[0],
                            ComponentProperty::DateTimeStart(DateTimeStartProperty {
                                params: vec![],
                                value: DateOrDateTime::DateTime(DateTime {
                                    date: Date {
                                        year: 2007,
                                        month: 3,
                                        day: 11,
                                    },
                                    time: Time {
                                        hour: 2,
                                        minute: 0,
                                        second: 0,
                                        is_utc: false,
                                    },
                                })
                            })
                        );

                        assert_eq!(
                            properties[1],
                            ComponentProperty::TimeZoneOffsetFrom(TimeZoneOffsetProperty {
                                other_params: vec![],
                                value: UtcOffset {
                                    sign: -1,
                                    hours: 5,
                                    minutes: 0,
                                    seconds: None,
                                }
                            })
                        );

                        assert_eq!(
                            properties[2],
                            ComponentProperty::TimeZoneOffsetTo(TimeZoneOffsetProperty {
                                other_params: vec![],
                                value: UtcOffset {
                                    sign: -1,
                                    hours: 4,
                                    minutes: 0,
                                    seconds: None,
                                }
                            })
                        );

                        assert_eq!(
                            properties[3],
                            ComponentProperty::TimeZoneName(TimeZoneNameProperty {
                                params: vec![],
                                value: b"EDT".to_vec(),
                            })
                        );
                    }
                    _ => panic!("Unexpected component type"),
                }
            }
            _ => panic!("Unexpected component type"),
        }
    }
}
