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
            .finish_component()
            .build();

        assert_eq!(obj.components.len(), 1);

        match &obj.components[0] {
            CalendarComponent::Event(e) => {
                assert_eq!(e.properties.len(), 1);
                match &e.properties[0] {
                    ComponentProperty::DateTimeStamp(p) => {
                        assert_eq!(p.params.len(), 1);
                    }
                    _ => panic!("Expected DateTimeStamp"),
                }
            }
            _ => panic!("Expected EventComponent"),
        }
    }
}
