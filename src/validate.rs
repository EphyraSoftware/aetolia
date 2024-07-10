mod error;

use crate::model::{CalendarComponent, CalendarProperty, ComponentProperty, ICalObject, Param};
use crate::validate::error::{CalendarPropertyError, ICalendarError, ParamError};

pub use error::*;

pub fn validate_model(ical_object: ICalObject) -> Vec<ICalendarError> {
    let mut errors = Vec::new();

    errors.extend_from_slice(
        ICalendarError::many_from_calendar_property_errors(validate_calendar_properties(
            &ical_object,
        ))
        .as_slice(),
    );

    for (index, component) in ical_object.components.iter().enumerate() {
        match component {
            CalendarComponent::Event(event) => {
                errors.extend_from_slice(
                    ICalendarError::many_from_component_property_errors(
                        validate_component_properties(&event.properties),
                        index,
                        component_name(component).to_string(),
                    )
                    .as_slice(),
                );
            }
            CalendarComponent::TimeZone(time_zone) => {
                errors.extend_from_slice(
                    ICalendarError::many_from_component_property_errors(
                        validate_component_properties(&time_zone.properties),
                        index,
                        component_name(component).to_string(),
                    )
                    .as_slice(),
                );
            }
            CalendarComponent::XComponent(x_component) => {
                errors.extend_from_slice(
                    ICalendarError::many_from_component_property_errors(
                        validate_component_properties(&x_component.properties),
                        index,
                        component_name(component).to_string(),
                    )
                    .as_slice(),
                );
            }
            _ => {
                unimplemented!()
            }
        }
    }

    errors
}

fn validate_component_properties(properties: &[ComponentProperty]) -> Vec<ComponentPropertyError> {
    let mut errors = Vec::new();

    for (index, property) in properties.iter().enumerate() {
        match property {
            ComponentProperty::Description(description) => {
                let property_info = PropertyInfo::new(ValueType::Text);
                errors.extend_from_slice(
                    ComponentPropertyError::many_from_param_errors(
                        validate_params(&description.params, property_info),
                        index,
                        component_property_name(property).to_string(),
                    )
                    .as_slice(),
                );
            }
            ComponentProperty::IanaProperty(_) => {
                // Nothing to validate
            }
            _ => {
                unimplemented!()
            }
        }
    }

    errors
}

#[derive(Debug)]
struct PropertyInfo {
    value_type: ValueType,
    /// This is an xProperty or ianaProperty
    is_other: bool,
}

impl PropertyInfo {
    fn new(value_type: ValueType) -> Self {
        PropertyInfo {
            value_type,
            is_other: false,
        }
    }

    fn other(mut self) -> Self {
        self.is_other = true;
        self
    }
}

#[derive(Eq, PartialEq, Debug)]
enum ValueType {
    VersionValue,
    CalendarAddress,
    Text,
}

fn validate_calendar_properties(ical_object: &ICalObject) -> Vec<CalendarPropertyError> {
    let mut errors = Vec::new();

    for (index, property) in ical_object.properties.iter().enumerate() {
        match property {
            CalendarProperty::Version(version) => {
                let property_info = PropertyInfo::new(ValueType::VersionValue);
                errors.extend_from_slice(
                    CalendarPropertyError::many_from_param_errors(
                        validate_params(&version.params, property_info),
                        index,
                        calendar_property_name(property).to_string(),
                    )
                    .as_slice(),
                );
            }
            _ => {
                unimplemented!()
            }
        }
    }

    errors
}

fn validate_params(params: &[Param], property_info: PropertyInfo) -> Vec<ParamError> {
    let mut errors = Vec::new();

    for (index, param) in params.iter().enumerate() {
        println!("{:?}", param);
        match param {
            Param::CommonName { name } => {
                validate_cn_param(&mut errors, param, index, &property_info);
            }
            Param::Others { name, .. } | Param::Other { name, .. } if name == "CN" => {
                validate_cn_param(&mut errors, param, index, &property_info);
            }
            _ => {
                unimplemented!()
            }
        }
    }

    errors
}

fn validate_cn_param(
    errors: &mut Vec<ParamError>,
    param: &Param,
    index: usize,
    property_info: &PropertyInfo,
) {
    println!(
        "Validating param {:?}, with info {:?}",
        param, property_info
    );
    if !property_info.is_other && property_info.value_type != ValueType::CalendarAddress {
        errors.push(ParamError {
            index,
            name: param_name(param).to_string(),
            message: "Common name is not allowed for this property type".to_string(),
        });
    }
}

fn calendar_property_name(property: &CalendarProperty) -> &str {
    match property {
        CalendarProperty::Version { .. } => "VERSION",
        _ => unimplemented!(),
    }
}

fn component_property_name(property: &ComponentProperty) -> &str {
    match property {
        ComponentProperty::Description { .. } => "DESCRIPTION",
        _ => unimplemented!(),
    }
}

fn component_name(component: &CalendarComponent) -> &str {
    match component {
        CalendarComponent::Event { .. } => "VEVENT",
        CalendarComponent::TimeZone(_) => "VTIMEZONE",
        CalendarComponent::XComponent(component) => &component.name,
        _ => unimplemented!(),
    }
}

fn param_name(param: &Param) -> &str {
    match param {
        Param::CommonName { .. } => "CN",
        Param::Other { name, .. } => name,
        _ => unimplemented!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convert::ToModel;
    use crate::model::ICalObjectBuilder;
    use crate::parser::Error;
    use crate::test_utils::check_rem;

    #[ignore = "Not enough structure implemented for this test yet"]
    #[test]
    fn sample_passes_validation() {
        let content = "BEGIN:VCALENDAR\r\n\
BEGIN:VTIMEZONE\r\n\
TZID:Fictitious\r\n\
LAST-MODIFIED:19870101T000000Z\r\n\
BEGIN:STANDARD\r\n\
DTSTART:19671029T020000\r\n\
RRULE:FREQ=YEARLY;BYDAY=-1SU;BYMONTH=10\r\n\
TZOFFSETFROM:-0400\r\n\
TZOFFSETTO:-0500\r\n\
TZNAME:EST\r\n\
END:STANDARD\r\n\
BEGIN:DAYLIGHT\r\n\
DTSTART:19870405T020000\r\n\
RRULE:FREQ=YEARLY;BYDAY=1SU;BYMONTH=4;UNTIL=19980404T070000Z\r\n\
TZOFFSETFROM:-0500\r\n\
TZOFFSETTO:-0400\r\n\
TZNAME:EDT\r\n\
END:DAYLIGHT\r\n\
BEGIN:DAYLIGHT\r\n\
DTSTART:19990424T020000\r\n\
RRULE:FREQ=YEARLY;BYDAY=-1SU;BYMONTH=4\r\n\
TZOFFSETFROM:-0500\r\n\
TZOFFSETTO:-0400\r\n\
TZNAME:EDT\r\n\
END:DAYLIGHT\r\n\
END:VTIMEZONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn common_name_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
VERSION;CN=hello:2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In calendar property \"VERSION\" at index 0: Common name is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn common_name_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
DESCRIPTION;CN=hello:some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 0: Common name is not allowed for this property type", errors.first().unwrap().to_string());
    }

    fn validate_content(content: &str) -> Vec<ICalendarError> {
        let (rem, object) = crate::parser::ical_object::<Error>(content.as_bytes()).unwrap();
        check_rem(rem, 0);

        validate_model(object.to_model().unwrap())
    }
}
