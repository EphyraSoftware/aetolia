#![allow(unused)]

mod component;
mod object;
mod param;
mod property;

pub use component::*;
pub use object::*;
pub use param::*;
pub use property::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::object::ICalObject;
    use crate::model::param::OtherParamsBuilder;
    use time::Date;

    #[test]
    fn all_cal_props_cal_object() {
        let obj = ICalObject::builder()
            .add_product_id("-//ABC Corporation//NONSGML My Product//EN")
            .add_x_param("x-special-param", "my-value")
            .finish_property()
            .add_max_version("2.0")
            .add_x_param_values(
                "x-special-param",
                vec!["one-value".to_string(), "another-value".to_string()],
            )
            .finish_property()
            .add_calendar_scale("GREGORIAN")
            .finish_property()
            .add_method("REQUEST")
            .finish_property()
            .add_x_property("X-PROP", "X-VALUE")
            .add_iana_param("special-param", "my-value")
            .finish_property()
            .add_iana_property("IANA-PARAM", "IANA-VALUE")
            .add_iana_param_values(
                "iana-special-param",
                vec!["one-value".to_string(), "another-value".to_string()],
            )
            .finish_property()
            .build();

        assert_eq!(obj.properties.len(), 6);
    }

    #[test]
    fn x_component_cal_object() {
        let obj = ICalObject::builder()
            .add_product_id("x_component_cal_object")
            .finish_property()
            .add_x_component("X-SOME-COMPONENT", |b| {
                b.add_x_property("X-SOME-PROP", "X-SOME-VALUE")
                    .add_x_param("x-special-param", "my-value")
                    .add_iana_param("special-param", "my-value")
                    .finish_property()
                    .finish_component()
            })
            .add_iana_component("IANA-SOME-COMPONENT", |b| {
                b.add_iana_property("IANA-SOME-PROP", "IANA-SOME-VALUE")
                    .add_iana_param("special-param", "my-value")
                    .add_x_param("x-special-param", "my-value")
                    .finish_property()
                    .finish_component()
            })
            .build();

        assert_eq!(obj.components.len(), 2);

        match &obj.components[0] {
            CalendarComponent::XComponent(x) => {
                assert_eq!(x.properties.len(), 1);
                match &x.properties[0] {
                    ComponentProperty::XProperty(p) => {
                        assert_eq!(p.params.len(), 2);
                    }
                    _ => panic!("Expected XProperty"),
                }
            }
            _ => panic!("Expected XComponent"),
        }

        match &obj.components[1] {
            CalendarComponent::IanaComponent(x) => {
                assert_eq!(x.properties.len(), 1);
                match &x.properties[0] {
                    ComponentProperty::IanaProperty(p) => {
                        assert_eq!(p.params.len(), 2);
                    }
                    _ => panic!("Expected IanaProperty"),
                }
            }
            _ => panic!("Expected IanaComponent"),
        }
    }

    #[test]
    fn event_component_cal_object() {
        let obj = ICalObject::builder()
            .add_product_id("event_component")
            .finish_property()
            .add_event_component()
            .add_date_time_stamp(
                time::Date::from_calendar_date(1997, time::Month::September, 1).unwrap(),
                time::Time::from_hms(13, 0, 0).unwrap(),
            )
            .add_x_param("X-SOME-PROP", "X-SOME-VALUE")
            .finish_property()
            .add_uid("some-uid")
            .add_x_param("x-special-param", "my-value")
            .finish_property()
            .add_date_time_start(
                time::Date::from_calendar_date(1997, time::Month::September, 1).unwrap(),
                Some(time::Time::from_hms(14, 30, 0).unwrap()),
            )
            .add_tz_id("America/New_York", true)
            .finish_property()
            .add_class(Classification::Private)
            .add_x_param("x-special-param", "my-value")
            .finish_property()
            .add_created(
                time::Date::from_calendar_date(1997, time::Month::September, 1).unwrap(),
                time::Time::from_hms(13, 0, 0).unwrap(),
            )
            .add_x_param("x-special-param", "my-value")
            .finish_property()
            .add_description("Event description")
            .add_alternate_representation("CID:evt.desc".to_string())
            .add_language("en-US".to_string())
            .add_x_param("x-special-param", "my-value")
            .finish_property()
            .add_geographic_position(37.386013, -122.082932)
            .add_x_param("x-special-param", "my-value")
            .finish_property()
            .add_organizer("mailto:john@local.net".to_string())
            .add_common_name("John")
            .add_directory_entry_reference("ldap://local.net/john".to_string())
            .add_sent_by("mailto:lilith@local.net".to_string())
            .add_language("en-US".to_string())
            .add_x_param("x-special-param", "my-value")
            .finish_property()
            .add_priority(4)
            .add_x_param("x-special-param", "my-value")
            .finish_property()
            .add_sequence(10)
            .add_x_param("x-special-param", "my-value")
            .finish_property()
            .add_request_status(&[200, 4], "Success".to_string(), None)
            .add_x_param("x-special-param", "my-value")
            .finish_property()
            .add_time_transparency(TimeTransparency::Transparent)
            .add_x_param("x-special-param", "my-value")
            .finish_property()
            .add_url("http://local.net/john".to_string())
            .add_x_param("x-special-param", "my-value")
            .finish_property()
            .add_recurrence_id(
                Date::from_calendar_date(1997, time::Month::September, 1).unwrap(),
                None,
            )
            .add_tz_id("America/New_York", true)
            .add_range(Range::ThisAndFuture)
            .add_x_param("x-special-param", "my-value")
            .finish_property()
            .finish_component()
            .build();

        assert_eq!(obj.components.len(), 1);

        match &obj.components[0] {
            CalendarComponent::Event(e) => {
                assert_eq!(e.properties.len(), 14);
                match &e.properties[0] {
                    ComponentProperty::DateTimeStamp(p) => {
                        assert_eq!(p.params.len(), 1);
                    }
                    _ => panic!("Expected DateTimeStamp"),
                }
                match &e.properties[1] {
                    ComponentProperty::UniqueIdentifier(p) => {
                        assert_eq!(p.params.len(), 1);
                    }
                    _ => panic!("Expected UniqueIdentifier"),
                }
                match &e.properties[2] {
                    ComponentProperty::DateTimeStart(p) => {
                        assert_eq!(p.params.len(), 1);
                    }
                    _ => panic!("Expected DateTimeStart"),
                }
                match &e.properties[3] {
                    ComponentProperty::Class(p) => {
                        assert_eq!(p.params.len(), 1);
                    }
                    _ => panic!("Expected Class"),
                }
                match &e.properties[4] {
                    ComponentProperty::Created(p) => {
                        assert_eq!(p.params.len(), 1);
                    }
                    _ => panic!("Expected Created"),
                }
                match &e.properties[5] {
                    ComponentProperty::Description(p) => {
                        assert_eq!(p.params.len(), 3);
                    }
                    _ => panic!("Expected Description"),
                }
                match &e.properties[6] {
                    ComponentProperty::GeographicPosition(p) => {
                        assert_eq!(p.params.len(), 1);
                    }
                    _ => panic!("Expected GeographicPosition"),
                }
                match &e.properties[7] {
                    ComponentProperty::Organizer(p) => {
                        assert_eq!(p.params.len(), 5);
                    }
                    _ => panic!("Expected Organizer"),
                }
                match &e.properties[8] {
                    ComponentProperty::Priority(p) => {
                        assert_eq!(p.params.len(), 1);
                    }
                    _ => panic!("Expected Priority"),
                }
                match &e.properties[9] {
                    ComponentProperty::Sequence(p) => {
                        assert_eq!(p.params.len(), 1);
                    }
                    _ => panic!("Expected Sequence"),
                }
                match &e.properties[10] {
                    ComponentProperty::RequestStatus(p) => {
                        assert_eq!(p.params.len(), 1);
                    }
                    _ => panic!("Expected RequestStatus"),
                }
                match &e.properties[11] {
                    ComponentProperty::TimeTransparency(p) => {
                        assert_eq!(p.params.len(), 1);
                    }
                    _ => panic!("Expected TimeTransparency"),
                }
                match &e.properties[12] {
                    ComponentProperty::Url(p) => {
                        assert_eq!(p.params.len(), 1);
                    }
                    _ => panic!("Expected Url"),
                }
                match &e.properties[13] {
                    ComponentProperty::RecurrenceId(p) => {
                        assert_eq!(p.params.len(), 4);
                    }
                    _ => panic!("Expected RecurrenceId"),
                }
            }
            _ => panic!("Expected EventComponent"),
        }
    }
}
