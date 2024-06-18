use crate::parser::component::alarm::component_alarm;
use crate::parser::object::types::{CalendarComponent, ComponentProperty};
use crate::parser::property::{
    prop_attach, prop_attendee, prop_categories, prop_classification, prop_comment, prop_contact,
    prop_created, prop_date_time_completed, prop_date_time_due, prop_date_time_stamp,
    prop_date_time_start, prop_description, prop_duration, prop_exception_date_times,
    prop_geographic_position, prop_iana, prop_last_modified, prop_location, prop_organizer,
    prop_percent_complete, prop_priority, prop_recurrence_date_times, prop_recurrence_id,
    prop_recurrence_rule, prop_related_to, prop_request_status, prop_resources, prop_sequence,
    prop_status, prop_summary, prop_unique_identifier, prop_url, prop_x,
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

pub fn component_todo<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], CalendarComponent<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, properties, alarms, _)) = tuple((
        tag("BEGIN:VTODO\r\n"),
        cut(many0(alt((
            alt((
                prop_date_time_stamp.map(ComponentProperty::DateTimeStamp),
                prop_unique_identifier.map(ComponentProperty::UniqueIdentifier),
                prop_classification.map(ComponentProperty::Classification),
                prop_date_time_completed.map(ComponentProperty::DateTimeCompleted),
                prop_created.map(ComponentProperty::DateTimeCreated),
                prop_description.map(ComponentProperty::Description),
                prop_date_time_start.map(ComponentProperty::DateTimeStart),
                prop_geographic_position.map(ComponentProperty::GeographicPosition),
                prop_last_modified.map(ComponentProperty::LastModified),
                prop_location.map(ComponentProperty::Location),
                prop_organizer.map(ComponentProperty::Organizer),
                prop_percent_complete.map(ComponentProperty::PercentComplete),
                prop_priority.map(ComponentProperty::Priority),
                prop_recurrence_id.map(ComponentProperty::RecurrenceId),
                prop_sequence.map(ComponentProperty::Sequence),
            )),
            alt((
                prop_status.map(ComponentProperty::Status),
                prop_summary.map(ComponentProperty::Summary),
                prop_url.map(ComponentProperty::Url),
                prop_recurrence_rule.map(ComponentProperty::RecurrenceRule),
                prop_date_time_due.map(ComponentProperty::DateTimeDue),
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
            prop_x.map(ComponentProperty::XProp),
            prop_iana.map(ComponentProperty::IanaProp),
        )))),
        many0(component_alarm),
        tag("END:VTODO\r\n"),
    ))(input)?;

    Ok((input, CalendarComponent::ToDo { properties, alarms }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::param::{Param, ParamValue, Value};
    use crate::parser::property::{
        CategoriesProperty, Classification, ClassificationProperty, Date, DateOrDateTime, DateTime,
        DateTimeDueProperty, DateTimeStampProperty, Status, StatusProperty, SummaryProperty, Time,
        UniqueIdentifierProperty,
    };
    use crate::parser::Error;
    use crate::test_utils::check_rem;

    #[test]
    fn test_component_todo() {
        let input = b"BEGIN:VTODO\r\nUID:20070313T123432Z-456553@example.com\r\nDTSTAMP:20070313T123432Z\r\nDUE;VALUE=DATE:20070501\r\nSUMMARY:Submit Quebec Income Tax Return for 2006\r\nCLASS:CONFIDENTIAL\r\nCATEGORIES:FAMILY,FINANCE\r\nSTATUS:NEEDS-ACTION\r\nEND:VTODO\r\n";
        let (rem, component) = component_todo::<Error>(input).unwrap();
        check_rem(rem, 0);

        match component {
            CalendarComponent::ToDo { properties, .. } => {
                assert_eq!(properties.len(), 7);

                assert_eq!(
                    properties[0],
                    ComponentProperty::UniqueIdentifier(UniqueIdentifierProperty {
                        other_params: vec![],
                        value: b"20070313T123432Z-456553@example.com".to_vec(),
                    })
                );

                assert_eq!(
                    properties[1],
                    ComponentProperty::DateTimeStamp(DateTimeStampProperty {
                        other_params: vec![],
                        value: DateTime {
                            date: Date {
                                year: 2007,
                                month: 3,
                                day: 13,
                            },
                            time: Time {
                                hour: 12,
                                minute: 34,
                                second: 32,
                                is_utc: true,
                            },
                        },
                    })
                );

                assert_eq!(
                    properties[2],
                    ComponentProperty::DateTimeDue(DateTimeDueProperty {
                        params: vec![Param {
                            name: "VALUE".to_string(),
                            value: ParamValue::Value { value: Value::Date },
                        }],
                        value: DateOrDateTime::Date(Date {
                            year: 2007,
                            month: 5,
                            day: 1,
                        }),
                    })
                );

                assert_eq!(
                    properties[3],
                    ComponentProperty::Summary(SummaryProperty {
                        params: vec![],
                        value: b"Submit Quebec Income Tax Return for 2006".to_vec(),
                    })
                );

                assert_eq!(
                    properties[4],
                    ComponentProperty::Classification(ClassificationProperty {
                        other_params: vec![],
                        value: Classification::Confidential,
                    })
                );

                assert_eq!(
                    properties[5],
                    ComponentProperty::Categories(CategoriesProperty {
                        params: vec![],
                        value: vec![b"FAMILY".to_vec(), b"FINANCE".to_vec()],
                    })
                );

                assert_eq!(
                    properties[6],
                    ComponentProperty::Status(StatusProperty {
                        other_params: vec![],
                        value: Status::NeedsAction,
                    })
                );
            }
            _ => panic!("Wrong component type"),
        }
    }
}
