use crate::parser::object::types::{CalendarComponent, ComponentProperty};
use crate::parser::property::{
    prop_attach, prop_attendee, prop_categories, prop_classification, prop_comment, prop_contact,
    prop_created, prop_date_time_stamp, prop_date_time_start, prop_description,
    prop_exception_date_times, prop_iana, prop_last_modified, prop_organizer,
    prop_recurrence_date_times, prop_recurrence_id, prop_recurrence_rule, prop_related_to,
    prop_request_status, prop_sequence, prop_status, prop_summary, prop_unique_identifier,
    prop_url, prop_x,
};
use crate::parser::Error;
use nom::branch::alt;
use nom::bytes::streaming::tag;
use nom::combinator::cut;
use nom::error::ParseError;
use nom::multi::many0;
use nom::sequence::tuple;
use nom::{IResult, Parser};

pub fn component_journal<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], CalendarComponent<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, properties, _)) = tuple((
        tag("BEGIN:VJOURNAL\r\n"),
        cut(many0(alt((
            alt((
                prop_date_time_stamp.map(ComponentProperty::DateTimeStamp),
                prop_unique_identifier.map(ComponentProperty::UniqueIdentifier),
                prop_classification.map(ComponentProperty::Classification),
                prop_created.map(ComponentProperty::DateTimeCreated),
                prop_date_time_start.map(ComponentProperty::DateTimeStart),
                prop_last_modified.map(ComponentProperty::LastModified),
                prop_organizer.map(ComponentProperty::Organizer),
                prop_recurrence_id.map(ComponentProperty::RecurrenceId),
                prop_sequence.map(ComponentProperty::Sequence),
                prop_status.map(ComponentProperty::Status),
                prop_summary.map(ComponentProperty::Summary),
                prop_url.map(ComponentProperty::Url),
                prop_recurrence_rule.map(ComponentProperty::RecurrenceRule),
                prop_attach.map(ComponentProperty::Attach),
                prop_attendee.map(ComponentProperty::Attendee),
            )),
            alt((
                prop_categories.map(ComponentProperty::Categories),
                prop_comment.map(ComponentProperty::Comment),
                prop_contact.map(ComponentProperty::Contact),
                prop_description.map(ComponentProperty::Description),
                prop_exception_date_times.map(ComponentProperty::ExceptionDateTimes),
                prop_related_to.map(ComponentProperty::RelatedTo),
                prop_recurrence_date_times.map(ComponentProperty::RecurrenceDateTimes),
                prop_request_status.map(ComponentProperty::RequestStatus),
            )),
            prop_x.map(ComponentProperty::XProp),
            prop_iana.map(ComponentProperty::IanaProp),
        )))),
        tag("END:VJOURNAL\r\n"),
    ))(input)?;

    Ok((input, CalendarComponent::Journal { properties }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::param::{Param, ParamValue, Value};
    use crate::parser::property::{
        Date, DateOrDateTime, DateTime, DateTimeStampProperty, DateTimeStartProperty,
        DescriptionProperty, SummaryProperty, Time, UniqueIdentifierProperty,
    };
    use crate::parser::Error;
    use crate::test_utils::check_rem;

    #[test]
    fn test_component_journal() {
        let input = b"BEGIN:VJOURNAL\r\nUID:19970901T130000Z-123405@example.com\r\nDTSTAMP:19970901T130000Z\r\nDTSTART;VALUE=DATE:19970317\r\nSUMMARY:Staff meeting minutes\r\nDESCRIPTION:1. Staff meeting: Participants include Joe\\,\r\n  Lisa\\, and Bob. Aurora project plans were reviewed.\r\n  There is currently no budget reserves for this project.\r\n  Lisa will escalate to management. Next meeting on Tuesday.\\n\r\n 2. Telephone Conference: ABC Corp. sales representative\r\n  called to discuss new printer. Promised to get us a demo by\r\n  Friday.\\n3. Henry Miller (Handsoff Insurance): Car was\r\n  totaled by tree. Is looking into a loaner car. 555-2323\r\n  (tel).\r\nEND:VJOURNAL\r\n";
        let (rem, component) = component_journal::<Error>(input).unwrap();
        check_rem(rem, 0);

        match component {
            CalendarComponent::Journal { properties } => {
                assert_eq!(properties.len(), 5);

                assert_eq!(
                    properties[0],
                    ComponentProperty::UniqueIdentifier(UniqueIdentifierProperty {
                        other_params: vec![],
                        value: b"19970901T130000Z-123405@example.com".to_vec(),
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
                        }
                    })
                );

                assert_eq!(
                    properties[2],
                    ComponentProperty::DateTimeStart(DateTimeStartProperty {
                        params: vec![Param {
                            name: "VALUE".to_string(),
                            value: ParamValue::Value { value: Value::Date }
                        }],
                        value: DateOrDateTime::Date(Date {
                            year: 1997,
                            month: 3,
                            day: 17,
                        })
                    })
                );

                assert_eq!(
                    properties[3],
                    ComponentProperty::Summary(SummaryProperty {
                        params: vec![],
                        value: b"Staff meeting minutes".to_vec(),
                    })
                );

                assert_eq!(
                    properties[4],
                    ComponentProperty::Description(DescriptionProperty {
                        params: vec![],
                        value: br#"1. Staff meeting: Participants include Joe, Lisa, and Bob. Aurora project plans were reviewed. There is currently no budget reserves for this project. Lisa will escalate to management. Next meeting on Tuesday.
2. Telephone Conference: ABC Corp. sales representative called to discuss new printer. Promised to get us a demo by Friday.
3. Henry Miller (Handsoff Insurance): Car was totaled by tree. Is looking into a loaner car. 555-2323 (tel)."#.to_vec(),
                    })
                );
            }
            _ => panic!("Wrong component type"),
        }
    }
}
