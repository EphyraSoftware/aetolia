use crate::parser::property::{
    prop_action, prop_attach, prop_attendee, prop_description, prop_duration, prop_iana,
    prop_repeat, prop_summary, prop_trigger, prop_x,
};
use crate::parser::types::CalendarComponent;
use crate::parser::types::ComponentProperty;
use crate::parser::Error;
use nom::branch::alt;
use nom::bytes::streaming::tag;
use nom::error::ParseError;
use nom::multi::many0;
use nom::{IResult, Parser};

pub fn component_alarm<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], CalendarComponent<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, properties, _)) = (
        tag("BEGIN:VALARM\r\n"),
        many0(alt((
            alt((
                prop_action.map(ComponentProperty::Action),
                prop_trigger.map(ComponentProperty::Trigger),
                prop_duration.map(ComponentProperty::Duration),
                prop_repeat.map(ComponentProperty::RepeatCount),
                prop_attach.map(ComponentProperty::Attach),
                prop_description.map(ComponentProperty::Description),
                prop_summary.map(ComponentProperty::Summary),
                prop_attendee.map(ComponentProperty::Attendee),
            )),
            prop_x.map(ComponentProperty::XProperty),
            prop_iana.map(ComponentProperty::IanaProperty),
        ))),
        tag("END:VALARM\r\n"),
    )
        .parse(input)?;

    Ok((input, CalendarComponent::Alarm { properties }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Value;
    use crate::parser::types::{
        Action, ActionProperty, AttachProperty, AttachValue, Date, DateTime, Duration,
        DurationOrDateTime, DurationProperty, ParamValue, RepeatProperty, Time, TriggerProperty,
    };
    use crate::parser::Error;
    use crate::test_utils::check_rem;

    #[test]
    fn test_component_alarm() {
        let input = b"BEGIN:VALARM\r\n\
TRIGGER;VALUE=DATE-TIME:19970317T133000Z\r\n\
REPEAT:4\r\n\
DURATION:PT15M\r\n\
ACTION:AUDIO\r\n\
ATTACH;FMTTYPE=audio/basic:ftp://example.com/pub/sounds/bell-01.aud\r\n\
END:VALARM\r\n";

        let (rem, component) = component_alarm::<Error>(input).unwrap();
        check_rem(rem, 0);
        match component {
            CalendarComponent::Alarm { properties } => {
                assert_eq!(5, properties.len());

                assert_eq!(
                    properties[0],
                    ComponentProperty::Trigger(TriggerProperty {
                        params: vec![ParamValue::ValueType {
                            value: Value::DateTime,
                        },],
                        value: DurationOrDateTime::DateTime(DateTime {
                            date: Date {
                                year: 1997,
                                month: 3,
                                day: 17,
                            },
                            time: Time {
                                hour: 13,
                                minute: 30,
                                second: 0,
                                is_utc: true,
                            },
                        }),
                    })
                );

                assert_eq!(
                    properties[1],
                    ComponentProperty::RepeatCount(RepeatProperty {
                        other_params: vec![],
                        value: 4,
                    })
                );

                assert_eq!(
                    properties[2],
                    ComponentProperty::Duration(DurationProperty {
                        other_params: vec![],
                        value: Duration {
                            sign: 1,
                            minutes: Some(15),
                            ..Default::default()
                        },
                    })
                );

                assert_eq!(
                    properties[3],
                    ComponentProperty::Action(ActionProperty {
                        other_params: vec![],
                        value: Action::Audio,
                    })
                );

                assert_eq!(
                    properties[4],
                    ComponentProperty::Attach(AttachProperty {
                        params: vec![ParamValue::FormatType {
                            type_name: "audio".to_string(),
                            sub_type_name: "basic".to_string(),
                        },],
                        value: AttachValue::Uri(b"ftp://example.com/pub/sounds/bell-01.aud"),
                    })
                );
            }
            _ => panic!("Unexpected component type"),
        }
    }
}
