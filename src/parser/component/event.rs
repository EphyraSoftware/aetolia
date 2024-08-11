use crate::parser::component::alarm::component_alarm;
use crate::parser::property::{
    prop_attach, prop_attendee, prop_categories, prop_classification, prop_comment, prop_contact,
    prop_created, prop_date_time_end, prop_date_time_stamp, prop_date_time_start, prop_description,
    prop_duration, prop_exception_date_times, prop_geographic_position, prop_iana,
    prop_last_modified, prop_location, prop_organizer, prop_priority, prop_recurrence_date_times,
    prop_recurrence_id, prop_recurrence_rule, prop_related_to, prop_request_status, prop_resources,
    prop_sequence, prop_status, prop_summary, prop_time_transparency, prop_unique_identifier,
    prop_url, prop_x,
};
use crate::parser::types::CalendarComponent;
use crate::parser::types::ComponentProperty;
use crate::parser::Error;
use nom::branch::alt;
use nom::bytes::streaming::tag;
use nom::combinator::cut;
use nom::error::ParseError;
use nom::multi::many0;
use nom::sequence::tuple;
use nom::{IResult, Parser};

pub fn component_event<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], CalendarComponent<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, properties, alarms, _)) = tuple((
        tag("BEGIN:VEVENT\r\n"),
        cut(many0(alt((
            alt((
                prop_date_time_stamp.map(ComponentProperty::DateTimeStamp),
                prop_unique_identifier.map(ComponentProperty::UniqueIdentifier),
                prop_date_time_start.map(ComponentProperty::DateTimeStart),
                prop_classification.map(ComponentProperty::Classification),
                prop_created.map(ComponentProperty::DateTimeCreated),
                prop_description.map(ComponentProperty::Description),
                prop_geographic_position.map(ComponentProperty::GeographicPosition),
                prop_last_modified.map(ComponentProperty::LastModified),
                prop_location.map(ComponentProperty::Location),
                prop_organizer.map(ComponentProperty::Organizer),
                prop_priority.map(ComponentProperty::Priority),
                prop_sequence.map(ComponentProperty::Sequence),
                prop_status.map(ComponentProperty::Status),
                prop_summary.map(ComponentProperty::Summary),
                prop_time_transparency.map(ComponentProperty::TimeTransparency),
            )),
            alt((
                prop_url.map(ComponentProperty::Url),
                prop_recurrence_id.map(ComponentProperty::RecurrenceId),
                prop_recurrence_rule.map(ComponentProperty::RecurrenceRule),
                prop_date_time_end.map(ComponentProperty::DateTimeEnd),
                prop_duration.map(ComponentProperty::Duration),
                prop_attach.map(ComponentProperty::Attach),
                prop_attendee.map(ComponentProperty::Attendee),
                prop_categories.map(ComponentProperty::Categories),
                prop_comment.map(ComponentProperty::Comment),
                prop_contact.map(ComponentProperty::Contact),
                prop_exception_date_times.map(ComponentProperty::ExceptionDateTimes),
                prop_request_status.map(ComponentProperty::RequestStatus),
                prop_related_to.map(ComponentProperty::RelatedTo),
                prop_resources.map(ComponentProperty::Resources),
                prop_recurrence_date_times.map(ComponentProperty::RecurrenceDateTimes),
            )),
            prop_x.map(ComponentProperty::XProperty),
            prop_iana.map(ComponentProperty::IanaProperty),
        )))),
        many0(component_alarm),
        tag("END:VEVENT\r\n"),
    ))(input)?;

    Ok((input, CalendarComponent::Event { properties, alarms }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::types::{
        CategoriesProperty, Classification, ClassificationProperty, Date, DateOrDateTime, DateTime,
        DateTimeEndProperty, DateTimeStampProperty, DateTimeStartProperty, SummaryProperty, Time,
        UniqueIdentifierProperty,
    };
    use crate::parser::Error;
    use crate::test_utils::check_rem;

    #[test]
    fn test_component_event() {
        let input = b"BEGIN:VEVENT\r\n\
UID:19970901T130000Z-123401@example.com\r\n\
DTSTAMP:19970901T130000Z\r\n\
DTSTART:19970903T163000Z\r\n\
DTEND:19970903T190000Z\r\n\
SUMMARY:Annual Employee Review\r\n\
CLASS:PRIVATE\r\n\
CATEGORIES:BUSINESS,HUMAN RESOURCES\r\n\
END:VEVENT\r\n";

        let (rem, component) = component_event::<Error>(input).unwrap();
        check_rem(rem, 0);

        match component {
            CalendarComponent::Event { properties, .. } => {
                assert_eq!(properties.len(), 7);

                assert_eq!(
                    properties[0],
                    ComponentProperty::UniqueIdentifier(UniqueIdentifierProperty {
                        other_params: vec![],
                        value: b"19970901T130000Z-123401@example.com".to_vec(),
                    })
                );

                assert_eq!(
                    properties[1],
                    ComponentProperty::DateTimeStamp(DateTimeStampProperty {
                        other_params: vec![],
                        value: DateTime {
                            date: Date {
                                year: 1997,
                                month: 9,
                                day: 1,
                            },
                            time: Time {
                                hour: 13,
                                minute: 0,
                                second: 0,
                                is_utc: true,
                            },
                        },
                    })
                );

                assert_eq!(
                    properties[2],
                    ComponentProperty::DateTimeStart(DateTimeStartProperty {
                        params: vec![],
                        value: DateOrDateTime::DateTime(DateTime {
                            date: Date {
                                year: 1997,
                                month: 9,
                                day: 3,
                            },
                            time: Time {
                                hour: 16,
                                minute: 30,
                                second: 0,
                                is_utc: true,
                            },
                        }),
                    })
                );

                assert_eq!(
                    properties[3],
                    ComponentProperty::DateTimeEnd(DateTimeEndProperty {
                        params: vec![],
                        value: DateOrDateTime::DateTime(DateTime {
                            date: Date {
                                year: 1997,
                                month: 9,
                                day: 3,
                            },
                            time: Time {
                                hour: 19,
                                minute: 0,
                                second: 0,
                                is_utc: true,
                            },
                        }),
                    })
                );

                assert_eq!(
                    properties[4],
                    ComponentProperty::Summary(SummaryProperty {
                        params: vec![],
                        value: b"Annual Employee Review".to_vec(),
                    })
                );

                assert_eq!(
                    properties[5],
                    ComponentProperty::Classification(ClassificationProperty {
                        other_params: vec![],
                        value: Classification::Private,
                    })
                );

                assert_eq!(
                    properties[6],
                    ComponentProperty::Categories(CategoriesProperty {
                        params: vec![],
                        value: vec![b"BUSINESS".to_vec(), b"HUMAN RESOURCES".to_vec()],
                    })
                );
            }
            _ => panic!("Unexpected component type"),
        }
    }
}
