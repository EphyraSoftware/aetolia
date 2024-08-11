use crate::common::PropertyKind;
use crate::model::object::ICalObject;
use crate::model::property::CalendarProperty;
use crate::validate::error::CalendarPropertyError;
use crate::validate::params::validate_params;
use crate::validate::{
    calendar_property_name, check_occurrence, CalendarInfo, CalendarPropertyLocation,
    ICalendarErrorSeverity, OccurrenceExpectation, PropertyInfo, PropertyLocation, ValueType,
};
use std::collections::HashMap;

pub(super) fn validate_calendar_properties(
    ical_object: &ICalObject,
    calendar_info: &mut CalendarInfo,
) -> Vec<CalendarPropertyError> {
    let mut errors = Vec::new();

    let mut seen = HashMap::<String, u32>::new();
    let add_count = |seen: &mut HashMap<String, u32>, key: &str| {
        *seen
            .entry(key.to_string())
            .and_modify(|count| *count += 1)
            .or_insert(1)
    };
    for (index, property) in ical_object.properties.iter().enumerate() {
        match property {
            CalendarProperty::ProductId(_) => {
                let name = calendar_property_name(property);
                add_count(&mut seen, name);

                if let Some(message) = check_occurrence(&seen, name, OccurrenceExpectation::Once) {
                    errors.push(CalendarPropertyError {
                        message,
                        severity: ICalendarErrorSeverity::Error,
                        location: Some(CalendarPropertyLocation {
                            index,
                            name: name.to_string(),
                            property_location: None,
                        }),
                    })
                }
            }
            CalendarProperty::Version(version) => {
                let name = calendar_property_name(property);
                add_count(&mut seen, name);

                if let Some(message) = check_occurrence(&seen, name, OccurrenceExpectation::Once) {
                    errors.push(CalendarPropertyError {
                        message,
                        severity: ICalendarErrorSeverity::Error,
                        location: Some(CalendarPropertyLocation {
                            index,
                            name: name.to_string(),
                            property_location: None,
                        }),
                    })
                }

                let property_info = PropertyInfo::new(
                    calendar_info,
                    PropertyLocation::Calendar,
                    PropertyKind::Version,
                    ValueType::VersionValue,
                );
                errors.extend_from_slice(
                    CalendarPropertyError::many_from_param_errors(
                        validate_params(&version.params, property_info),
                        index,
                        calendar_property_name(property).to_string(),
                    )
                    .as_slice(),
                );
            }
            CalendarProperty::CalendarScale(_) => {
                let name = calendar_property_name(property);
                add_count(&mut seen, name);

                if let Some(message) =
                    check_occurrence(&seen, name, OccurrenceExpectation::OptionalOnce)
                {
                    errors.push(CalendarPropertyError {
                        message,
                        severity: ICalendarErrorSeverity::Error,
                        location: Some(CalendarPropertyLocation {
                            index,
                            name: name.to_string(),
                            property_location: None,
                        }),
                    })
                }
            }
            CalendarProperty::Method(method) => {
                if calendar_info.method.is_none() {
                    calendar_info.method = Some(method.value.clone());
                }

                let name = calendar_property_name(property);
                add_count(&mut seen, name);

                if let Some(message) =
                    check_occurrence(&seen, name, OccurrenceExpectation::OptionalOnce)
                {
                    errors.push(CalendarPropertyError {
                        message,
                        severity: ICalendarErrorSeverity::Error,
                        location: Some(CalendarPropertyLocation {
                            index,
                            name: name.to_string(),
                            property_location: None,
                        }),
                    })
                }
            }
            _ => {
                // Nothing further to validate
            }
        }
    }

    // Check required properties, in case they were missing. If they are specified more than once
    // then it will produce duplicate errors.
    if let Some(message) = check_occurrence(&seen, "PRODID", OccurrenceExpectation::Once) {
        errors.push(CalendarPropertyError {
            message,
            severity: ICalendarErrorSeverity::Error,
            location: None,
        })
    }
    if let Some(message) = check_occurrence(&seen, "VERSION", OccurrenceExpectation::Once) {
        errors.push(CalendarPropertyError {
            message,
            severity: ICalendarErrorSeverity::Error,
            location: None,
        })
    }

    errors
}
