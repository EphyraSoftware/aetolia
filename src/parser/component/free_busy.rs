use crate::parser::object::types::{CalendarComponent, ComponentProperty};
use crate::parser::property::{
    prop_attendee, prop_comment, prop_contact, prop_date_time_end, prop_date_time_stamp,
    prop_date_time_start, prop_free_busy_time, prop_iana, prop_organizer, prop_request_status,
    prop_unique_identifier, prop_url, prop_x,
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

pub fn component_free_busy<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], CalendarComponent<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, properties, _)) = tuple((
        tag("BEGIN:VFREEBUSY\r\n"),
        cut(many0(alt((
            alt((
                prop_date_time_stamp.map(ComponentProperty::DateTimeStamp),
                prop_unique_identifier.map(ComponentProperty::UniqueIdentifier),
                prop_contact.map(ComponentProperty::Contact),
                prop_date_time_start.map(ComponentProperty::DateTimeStart),
                prop_date_time_end.map(ComponentProperty::DateTimeEnd),
                prop_organizer.map(ComponentProperty::Organizer),
                prop_url.map(ComponentProperty::Url),
                prop_attendee.map(ComponentProperty::Attendee),
                prop_comment.map(ComponentProperty::Comment),
                prop_free_busy_time.map(ComponentProperty::FreeBusyTime),
                prop_request_status.map(ComponentProperty::RequestStatus),
            )),
            prop_x.map(ComponentProperty::XProperty),
            prop_iana.map(ComponentProperty::IanaProperty),
        )))),
        tag("END:VFREEBUSY\r\n"),
    ))(input)?;

    Ok((input, CalendarComponent::FreeBusy { properties }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::property::{
        AttendeeProperty, Date, DateOrDateTime, DateTime, DateTimeEndProperty,
        DateTimeStampProperty, DateTimeStartProperty, OrganizerProperty, Time,
        UniqueIdentifierProperty,
    };
    use crate::parser::Error;
    use crate::test_utils::check_rem;

    #[test]
    fn test_component_free_busy() {
        let input = b"BEGIN:VFREEBUSY\r\n\
UID:19970901T082949Z-FA43EF@example.com\r\n\
ORGANIZER:mailto:jane_doe@example.com\r\n\
ATTENDEE:mailto:john_public@example.com\r\n\
DTSTART:19971015T050000Z\r\n\
DTEND:19971016T050000Z\r\n\
DTSTAMP:19970901T083000Z\r\n\
END:VFREEBUSY\r\n";

        let (rem, component) = component_free_busy::<Error>(input).unwrap();
        check_rem(rem, 0);
        match component {
            CalendarComponent::FreeBusy { properties } => {
                assert_eq!(properties.len(), 6);

                assert_eq!(
                    properties[0],
                    ComponentProperty::UniqueIdentifier(UniqueIdentifierProperty {
                        other_params: vec![],
                        value: b"19970901T082949Z-FA43EF@example.com".to_vec(),
                    })
                );

                assert_eq!(
                    properties[1],
                    ComponentProperty::Organizer(OrganizerProperty {
                        params: vec![],
                        value: b"mailto:jane_doe@example.com",
                    })
                );

                assert_eq!(
                    properties[2],
                    ComponentProperty::Attendee(AttendeeProperty {
                        params: vec![],
                        value: b"mailto:john_public@example.com",
                    })
                );

                assert_eq!(
                    properties[3],
                    ComponentProperty::DateTimeStart(DateTimeStartProperty {
                        params: vec![],
                        value: DateOrDateTime::DateTime(DateTime {
                            date: Date {
                                year: 1997,
                                month: 10,
                                day: 15,
                            },
                            time: Time {
                                hour: 5,
                                minute: 0,
                                second: 0,
                                is_utc: true,
                            },
                        }),
                    })
                );

                assert_eq!(
                    properties[4],
                    ComponentProperty::DateTimeEnd(DateTimeEndProperty {
                        params: vec![],
                        value: DateOrDateTime::DateTime(DateTime {
                            date: Date {
                                year: 1997,
                                month: 10,
                                day: 16,
                            },
                            time: Time {
                                hour: 5,
                                minute: 0,
                                second: 0,
                                is_utc: true,
                            },
                        }),
                    })
                );

                assert_eq!(
                    properties[5],
                    ComponentProperty::DateTimeStamp(DateTimeStampProperty {
                        other_params: vec![],
                        value: DateTime {
                            date: Date {
                                year: 1997,
                                month: 9,
                                day: 1,
                            },
                            time: Time {
                                hour: 8,
                                minute: 30,
                                second: 0,
                                is_utc: true,
                            },
                        },
                    })
                );
            }
            _ => panic!("Unexpected component type"),
        }
    }
}
