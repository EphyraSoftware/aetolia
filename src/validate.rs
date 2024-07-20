mod error;

use crate::common::{Encoding, ParticipationStatusUnknown, RecurFreq, Status, Value};
use crate::convert::ToModel;
use crate::model::{
    CalendarComponent, CalendarProperty, ComponentProperty, ICalObject, Param, RecurRulePart,
    RecurrenceRule, XProperty,
};
use crate::parser::recur::recur;
use crate::parser::uri::param_value_uri;
use crate::parser::{
    prop_value_date, prop_value_date_time, prop_value_duration, prop_value_float,
    prop_value_integer, prop_value_period, prop_value_text, prop_value_time, prop_value_utc_offset,
    Error,
};
use crate::serialize::WriteModel;
use crate::validate::error::{CalendarPropertyError, ICalendarError, ParamError};
use anyhow::Context;
pub use error::*;
use nom::character::streaming::char;
use nom::multi::separated_list1;
use nom::{AsBytes, Parser};
use std::collections::{HashMap, HashSet};

pub fn validate_model(ical_object: ICalObject) -> anyhow::Result<Vec<ICalendarError>> {
    let mut errors = Vec::new();

    let time_zone_ids = ical_object
        .components
        .iter()
        .filter_map(|component| {
            if let CalendarComponent::TimeZone(time_zone) = component {
                for property in &time_zone.properties {
                    if let ComponentProperty::TimeZoneId(tz_id) = property {
                        return Some(tz_id.value.clone());
                    }
                }
            }

            None
        })
        .collect::<HashSet<_>>();

    let mut calendar_info = CalendarInfo::new(time_zone_ids);

    errors.extend_from_slice(
        ICalendarError::many_from_calendar_property_errors(validate_calendar_properties(
            &ical_object,
            &mut calendar_info,
        ))
        .as_slice(),
    );

    if ical_object.components.is_empty() {
        errors.push(ICalendarError {
            message: "No components found in calendar object, required at least one".to_string(),
            location: None,
        });
    }

    for (index, component) in ical_object.components.iter().enumerate() {
        match component {
            CalendarComponent::Event(event) => {
                errors.extend_from_slice(
                    ICalendarError::many_from_component_property_errors(
                        validate_component_properties(
                            &calendar_info,
                            PropertyLocation::Event,
                            &event.properties,
                        )?,
                        index,
                        component_name(component).to_string(),
                    )
                    .as_slice(),
                );
            }
            CalendarComponent::ToDo(to_do) => {
                errors.extend_from_slice(
                    ICalendarError::many_from_component_property_errors(
                        validate_component_properties(
                            &calendar_info,
                            PropertyLocation::ToDo,
                            &to_do.properties,
                        )?,
                        index,
                        component_name(component).to_string(),
                    )
                    .as_slice(),
                );
            }
            CalendarComponent::Journal(journal) => {
                errors.extend_from_slice(
                    ICalendarError::many_from_component_property_errors(
                        validate_component_properties(
                            &calendar_info,
                            PropertyLocation::Journal,
                            &journal.properties,
                        )?,
                        index,
                        component_name(component).to_string(),
                    )
                    .as_slice(),
                );
            }
            CalendarComponent::FreeBusy(free_busy) => {
                errors.extend_from_slice(
                    ICalendarError::many_from_component_property_errors(
                        validate_component_properties(
                            &calendar_info,
                            PropertyLocation::FreeBusy,
                            &free_busy.properties,
                        )?,
                        index,
                        component_name(component).to_string(),
                    )
                    .as_slice(),
                );
            }
            CalendarComponent::TimeZone(time_zone) => {
                errors.extend_from_slice(
                    ICalendarError::many_from_component_property_errors(
                        validate_component_properties(
                            &calendar_info,
                            PropertyLocation::TimeZone,
                            &time_zone.properties,
                        )?,
                        index,
                        component_name(component).to_string(),
                    )
                    .as_slice(),
                );
            }
            CalendarComponent::XComponent(x_component) => {
                errors.extend_from_slice(
                    ICalendarError::many_from_component_property_errors(
                        validate_component_properties(
                            &calendar_info,
                            PropertyLocation::Other,
                            &x_component.properties,
                        )?,
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

    Ok(errors)
}

macro_rules! check_component_property_occurrence {
    ($errors:ident, $seen:ident, $property:ident, $index:ident, $occur:expr) => {
        let name = component_property_name($property);
        let count = add_to_seen(&mut $seen, name);
        if let Some(message) = check_occurrence(&$seen, name, $occur.clone()) {
            $errors.push(ComponentPropertyError {
                message,
                location: Some(ComponentPropertyLocation {
                    index: $index,
                    name: name.to_string(),
                    property_location: None,
                }),
            });
        }

        // If the property shouldn't appear then don't validate it further.
        if $occur == OccurrenceExpectation::Never {
            continue;
        }
    };
}

fn validate_component_properties(
    calendar_info: &CalendarInfo,
    property_location: PropertyLocation,
    properties: &[ComponentProperty],
) -> anyhow::Result<Vec<ComponentPropertyError>> {
    let mut errors = Vec::new();

    if properties.is_empty() {
        errors.push(ComponentPropertyError {
            message: "No properties found in component, required at least one".to_string(),
            location: None,
        });
    }

    let dt_stamp_occurrence_expectation = match property_location {
        PropertyLocation::Event
        | PropertyLocation::ToDo
        | PropertyLocation::Journal
        | PropertyLocation::FreeBusy => OccurrenceExpectation::Once,
        PropertyLocation::TimeZone | PropertyLocation::TimeZoneComponent => {
            OccurrenceExpectation::Never
        }
        PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
        PropertyLocation::Calendar => {
            unreachable!()
        }
    };

    let uid_occurrence_expectation = match property_location {
        PropertyLocation::Event
        | PropertyLocation::ToDo
        | PropertyLocation::Journal
        | PropertyLocation::FreeBusy => OccurrenceExpectation::Once,
        PropertyLocation::TimeZone | PropertyLocation::TimeZoneComponent => {
            OccurrenceExpectation::Never
        }
        PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
        PropertyLocation::Calendar => {
            unreachable!()
        }
    };

    let dt_start_expectation = match property_location {
        PropertyLocation::Event => {
            if calendar_info.method.is_none() {
                OccurrenceExpectation::Once
            } else {
                OccurrenceExpectation::OptionalOnce
            }
        }
        PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Once,
        PropertyLocation::ToDo | PropertyLocation::Journal | PropertyLocation::FreeBusy => {
            OccurrenceExpectation::OptionalOnce
        }
        PropertyLocation::TimeZone => OccurrenceExpectation::Never,
        PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
        PropertyLocation::Calendar => {
            unreachable!()
        }
    };

    let tz_id_occurrence_expectation = match property_location {
        PropertyLocation::Event
        | PropertyLocation::ToDo
        | PropertyLocation::Journal
        | PropertyLocation::FreeBusy
        | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
        PropertyLocation::TimeZone => OccurrenceExpectation::Once,
        PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
        PropertyLocation::Calendar => {
            unreachable!()
        }
    };

    let tz_offset_to_occurrence_expectation = match property_location {
        PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Once,
        PropertyLocation::Event
        | PropertyLocation::ToDo
        | PropertyLocation::Journal
        | PropertyLocation::FreeBusy
        | PropertyLocation::TimeZone => OccurrenceExpectation::Never,
        PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
        _ => {
            unimplemented!()
        }
    };

    let tz_offset_from_occurrence_expectation = match property_location {
        PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Once,
        PropertyLocation::Event
        | PropertyLocation::ToDo
        | PropertyLocation::Journal
        | PropertyLocation::FreeBusy
        | PropertyLocation::TimeZone => OccurrenceExpectation::Never,
        PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
        _ => {
            unimplemented!()
        }
    };

    let mut has_dt_start = false;
    let mut has_dt_end = false;
    let mut has_duration = false;
    let mut has_due = false;

    let mut seen = HashMap::<String, u32>::new();
    for (index, property) in properties.iter().enumerate() {
        check_encoding_for_binary_values(&mut errors, property, index)?;

        match property {
            ComponentProperty::DateTimeStamp(_) => {
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    dt_stamp_occurrence_expectation.clone()
                );
            }
            ComponentProperty::UniqueIdentifier(_) => {
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    uid_occurrence_expectation.clone()
                );
            }
            ComponentProperty::DateTimeStart(date_time_start) => {
                has_dt_start = true;

                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    dt_start_expectation.clone()
                );

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    if date_time_start.time.is_some() {
                        ValueType::DateTime
                    } else {
                        ValueType::Date
                    },
                )
                .utc(date_time_start.is_utc);
                errors.extend_from_slice(
                    ComponentPropertyError::many_from_param_errors(
                        validate_params(&date_time_start.params, property_info),
                        index,
                        component_property_name(property).to_string(),
                    )
                    .as_slice(),
                );
            }
            ComponentProperty::Classification(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal => OccurrenceExpectation::OptionalOnce,
                    PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::DateTimeCreated(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal => OccurrenceExpectation::OptionalOnce,
                    PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::Description(description) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event | PropertyLocation::ToDo => {
                        OccurrenceExpectation::OptionalOnce
                    }
                    PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::Journal | PropertyLocation::Other => {
                        OccurrenceExpectation::OptionalMany
                    }
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );

                let property_info =
                    PropertyInfo::new(calendar_info, property_location.clone(), ValueType::Text);
                errors.extend_from_slice(
                    ComponentPropertyError::many_from_param_errors(
                        validate_params(&description.params, property_info),
                        index,
                        component_property_name(property).to_string(),
                    )
                    .as_slice(),
                );
            }
            ComponentProperty::GeographicPosition(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event | PropertyLocation::ToDo => {
                        OccurrenceExpectation::OptionalOnce
                    }
                    PropertyLocation::Journal
                    | PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::LastModified(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal
                    | PropertyLocation::TimeZone => OccurrenceExpectation::OptionalOnce,
                    PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::Location(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event | PropertyLocation::ToDo => {
                        OccurrenceExpectation::OptionalOnce
                    }
                    PropertyLocation::Journal
                    | PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::Organizer(organizer) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal
                    | PropertyLocation::FreeBusy => OccurrenceExpectation::OptionalOnce,
                    PropertyLocation::TimeZone | PropertyLocation::TimeZoneComponent => {
                        OccurrenceExpectation::Never
                    }
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    ValueType::CalendarAddress,
                );
                errors.extend_from_slice(
                    ComponentPropertyError::many_from_param_errors(
                        validate_params(&organizer.params, property_info),
                        index,
                        component_property_name(property).to_string(),
                    )
                    .as_slice(),
                );
            }
            ComponentProperty::Priority(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event | PropertyLocation::ToDo => {
                        OccurrenceExpectation::OptionalOnce
                    }
                    PropertyLocation::Journal
                    | PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::Sequence(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal => OccurrenceExpectation::OptionalOnce,
                    PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::Status(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal => OccurrenceExpectation::OptionalOnce,
                    PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::Summary(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal => OccurrenceExpectation::OptionalOnce,
                    PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::TimeTransparency(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event => OccurrenceExpectation::OptionalOnce,
                    PropertyLocation::ToDo
                    | PropertyLocation::Journal
                    | PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::Url(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal
                    | PropertyLocation::FreeBusy => OccurrenceExpectation::OptionalOnce,
                    PropertyLocation::TimeZone | PropertyLocation::TimeZoneComponent => {
                        OccurrenceExpectation::Never
                    }
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::RecurrenceId(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal => OccurrenceExpectation::OptionalOnce,
                    PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::RecurrenceRule(_) => {
                // An RRULE can appear more than once, it just SHOULD NOT.
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::OptionalMany,
                    PropertyLocation::FreeBusy | PropertyLocation::TimeZone => {
                        OccurrenceExpectation::Never
                    }
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::DateTimeEnd(_) => {
                has_dt_end = true;

                let occurrence_expectation = match property_location {
                    PropertyLocation::Event | PropertyLocation::FreeBusy => {
                        OccurrenceExpectation::OptionalOnce
                    }
                    PropertyLocation::ToDo
                    | PropertyLocation::Journal
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::Duration(_) => {
                has_duration = true;

                let occurrence_expectation = match property_location {
                    PropertyLocation::Event | PropertyLocation::ToDo => {
                        OccurrenceExpectation::OptionalOnce
                    }
                    PropertyLocation::Journal
                    | PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::Attach(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal => OccurrenceExpectation::OptionalMany,
                    PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::Attendee(attendee) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal
                    | PropertyLocation::FreeBusy => OccurrenceExpectation::OptionalMany,
                    PropertyLocation::TimeZone | PropertyLocation::TimeZoneComponent => {
                        OccurrenceExpectation::Never
                    }
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    ValueType::CalendarAddress,
                );
                errors.extend_from_slice(
                    ComponentPropertyError::many_from_param_errors(
                        validate_params(&attendee.params, property_info),
                        index,
                        component_property_name(property).to_string(),
                    )
                    .as_slice(),
                );
            }
            ComponentProperty::Categories(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal => OccurrenceExpectation::OptionalMany,
                    PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::Comment(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal
                    | PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::OptionalMany,
                    PropertyLocation::TimeZone => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::Contact(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::FreeBusy => OccurrenceExpectation::OptionalOnce,
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal => OccurrenceExpectation::OptionalMany,
                    PropertyLocation::TimeZone => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::ExceptionDateTimes(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal => OccurrenceExpectation::OptionalMany,
                    PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::RequestStatus(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal
                    | PropertyLocation::FreeBusy => OccurrenceExpectation::OptionalMany,
                    PropertyLocation::TimeZone | PropertyLocation::TimeZoneComponent => {
                        OccurrenceExpectation::Never
                    }
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::RelatedTo(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal => OccurrenceExpectation::OptionalMany,
                    PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::Resources(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event | PropertyLocation::ToDo => {
                        OccurrenceExpectation::OptionalMany
                    }
                    PropertyLocation::Journal
                    | PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::RecurrenceDateTimes(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::OptionalMany,
                    PropertyLocation::FreeBusy | PropertyLocation::TimeZone => {
                        OccurrenceExpectation::Never
                    }
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::DateTimeCompleted(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::Journal
                    | PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::ToDo => OccurrenceExpectation::OptionalOnce,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::PercentComplete(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::Journal
                    | PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::ToDo => OccurrenceExpectation::OptionalOnce,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::DateTimeDue(_) => {
                has_due = true;

                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::Journal
                    | PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::ToDo => OccurrenceExpectation::OptionalOnce,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::FreeBusyTime(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::FreeBusy => OccurrenceExpectation::OptionalMany,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::TimeZoneId(time_zone_id) => {
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    tz_id_occurrence_expectation
                );

                let property_info =
                    PropertyInfo::new(calendar_info, property_location.clone(), ValueType::Text);
                errors.extend_from_slice(
                    ComponentPropertyError::many_from_param_errors(
                        validate_params(&time_zone_id.params, property_info),
                        index,
                        component_property_name(property).to_string(),
                    )
                    .as_slice(),
                );
            }
            ComponentProperty::TimeZoneUrl(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::TimeZone => OccurrenceExpectation::OptionalOnce,
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal
                    | PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::TimeZoneOffsetTo(_) => {
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    tz_offset_to_occurrence_expectation
                );
            }
            ComponentProperty::TimeZoneOffsetFrom(_) => {
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    tz_offset_from_occurrence_expectation
                );
            }
            ComponentProperty::TimeZoneName(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::TimeZoneComponent => OccurrenceExpectation::OptionalMany,
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal
                    | PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::Action(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal
                    | PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::Trigger(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal
                    | PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::Repeat(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal
                    | PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZone
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => {
                        unimplemented!()
                    }
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::IanaProperty(_) => {
                // Nothing to validate
            }
            ComponentProperty::XProperty(_) => {
                // Nothing to validate
            }
        }
    }

    if let Some(message) = check_occurrence(&seen, "DTSTAMP", dt_stamp_occurrence_expectation) {
        errors.push(ComponentPropertyError {
            message,
            location: None,
        });
    }
    if let Some(message) = check_occurrence(&seen, "UID", uid_occurrence_expectation) {
        errors.push(ComponentPropertyError {
            message,
            location: None,
        });
    }
    if let Some(message) = check_occurrence(&seen, "DTSTART", dt_start_expectation) {
        errors.push(ComponentPropertyError {
            message,
            location: None,
        });
    }
    if let Some(message) = check_occurrence(&seen, "TZID", tz_id_occurrence_expectation) {
        errors.push(ComponentPropertyError {
            message,
            location: None,
        });
    }
    if let Some(message) =
        check_occurrence(&seen, "TZOFFSETTO", tz_offset_to_occurrence_expectation)
    {
        errors.push(ComponentPropertyError {
            message,
            location: None,
        });
    }
    if let Some(message) =
        check_occurrence(&seen, "TZOFFSETFROM", tz_offset_from_occurrence_expectation)
    {
        errors.push(ComponentPropertyError {
            message,
            location: None,
        });
    }

    match property_location {
        PropertyLocation::Event => {
            if has_dt_end && has_duration {
                errors.push(ComponentPropertyError {
                    message: "Both DTEND and DURATION properties are present, only one is allowed"
                        .to_string(),
                    location: None,
                });
            }
        }
        PropertyLocation::ToDo => {
            if has_due && has_duration {
                errors.push(ComponentPropertyError {
                    message: "Both DUE and DURATION properties are present, only one is allowed"
                        .to_string(),
                    location: None,
                });
            }

            if has_duration && !has_dt_start {
                errors.push(ComponentPropertyError {
                    message: "DURATION property is present but no DTSTART property is present"
                        .to_string(),
                    location: None,
                });
            }
        }
        _ => {}
    }

    Ok(errors)
}

fn check_encoding_for_binary_values(
    errors: &mut Vec<ComponentPropertyError>,
    property: &ComponentProperty,
    property_index: usize,
) -> anyhow::Result<()> {
    let declared_value_type = get_declared_value_type(property);

    if let Some((value_type, value_type_index)) = declared_value_type {
        match value_type {
            Value::Binary => {
                let mut encoding_param_found = false;
                for param in property.params() {
                    if let Param::Encoding { encoding } = param {
                        encoding_param_found = true;

                        if *encoding != Encoding::Base64 {
                            let mut msg = b"Property is declared to have a binary value but the encoding is set to ".to_vec();
                            encoding
                                .write_model(&mut msg)
                                .context("Failed to write encoding to model")?;
                            msg.extend_from_slice(", instead of BASE64".as_bytes());

                            errors.push(ComponentPropertyError {
                                message: String::from_utf8_lossy(&msg).to_string(),
                                location: Some(ComponentPropertyLocation {
                                    index: property_index,
                                    name: component_property_name(property).to_string(),
                                    property_location: Some(WithinPropertyLocation::Param {
                                        index: value_type_index,
                                        name: "VALUE".to_string(),
                                    }),
                                }),
                            });
                        }
                    }
                }

                if !encoding_param_found {
                    errors.push(ComponentPropertyError {
                        message: "Property is declared to have a binary value but no encoding is set, must be set to BASE64".to_string(),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Param {
                                index: value_type_index,
                                name: "VALUE".to_string(),
                            }),
                        }),
                    });
                }
            }
            Value::Boolean => match property {
                ComponentProperty::XProperty(x_prop) if !is_boolean_valued(&x_prop.value) => {
                    errors.push(ComponentPropertyError {
                            message: "Property is declared to have a boolean value but the value is not a boolean".to_string(),
                            location: Some(ComponentPropertyLocation {
                                index: property_index,
                                name: component_property_name(property).to_string(),
                                property_location: None,
                            }),
                        });
                }
                ComponentProperty::IanaProperty(iana_prop)
                    if !is_boolean_valued(&iana_prop.value) =>
                {
                    errors.push(ComponentPropertyError {
                            message: "Property is declared to have a boolean value but the value is not a boolean".to_string(),
                            location: Some(ComponentPropertyLocation {
                                index: property_index,
                                name: component_property_name(property).to_string(),
                                property_location: None,
                            }),
                        });
                }
                _ => {
                    // Otherwise the value is boolean or not based on Rust typing
                }
            },
            Value::CalendarAddress => {
                let mut not_mailto = false;
                match property {
                    ComponentProperty::Attendee(_) | ComponentProperty::Organizer(_) => {
                        errors.push(ComponentPropertyError {
                            message:
                                "Redundant value specification which matches the default value"
                                    .to_string(),
                            location: Some(ComponentPropertyLocation {
                                index: property_index,
                                name: component_property_name(property).to_string(),
                                property_location: None,
                            }),
                        });
                    }
                    ComponentProperty::XProperty(x_prop) if x_prop.value.starts_with("mailto:") => {
                        not_mailto = true;
                    }
                    ComponentProperty::IanaProperty(iana_prop)
                        if iana_prop.value.starts_with("mailto:") =>
                    {
                        not_mailto = true;
                    }
                    _ => {
                        errors.push(ComponentPropertyError {
                            message: "Property is declared to have a calendar address value but that is not valid for this property".to_string(),
                            location: Some(ComponentPropertyLocation {
                                index: property_index,
                                name: component_property_name(property).to_string(),
                                property_location: None,
                            }),
                        });
                    }
                }

                if !not_mailto {
                    errors.push(ComponentPropertyError {
                        message: "Property is declared to have a calendar address value but the value is a mailto: URI".to_string(),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: None,
                        }),
                    });
                }
            }
            Value::Date => {
                let mut invalid = false;
                match property {
                    // TODO Valid property types need to be listed
                    ComponentProperty::XProperty(x_prop) => {
                        invalid = !is_date_valued(&x_prop.value);
                    }
                    ComponentProperty::IanaProperty(iana_prop) => {
                        invalid = !is_date_valued(&iana_prop.value);
                    }
                    _ => {
                        errors.push(ComponentPropertyError {
                            message: "Property is declared to have a date value but that is not valid for this property".to_string(),
                            location: Some(ComponentPropertyLocation {
                                index: property_index,
                                name: component_property_name(property).to_string(),
                                property_location: None,
                            }),
                        });
                    }
                }

                if invalid {
                    errors.push(ComponentPropertyError {
                        message:
                            "Property is declared to have a date value but the value is not a date"
                                .to_string(),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            Value::DateTime => {
                let mut invalid = false;
                match property {
                    ComponentProperty::DateTimeCompleted(_)
                    | ComponentProperty::DateTimeCreated(_)
                    | ComponentProperty::DateTimeStamp(_)
                    | ComponentProperty::LastModified(_) => {
                        errors.push(ComponentPropertyError {
                            message:
                                "Redundant value specification which matches the default value"
                                    .to_string(),
                            location: Some(ComponentPropertyLocation {
                                index: property_index,
                                name: component_property_name(property).to_string(),
                                property_location: None,
                            }),
                        });
                    }
                    ComponentProperty::XProperty(x_prop) => {
                        invalid = !is_date_time_valued(&x_prop.value);
                    }
                    ComponentProperty::IanaProperty(iana_prop) => {
                        invalid = !is_date_time_valued(&iana_prop.value);
                    }
                    _ => {
                        errors.push(ComponentPropertyError {
                            message: "Property is declared to have a date-time value but that is not valid for this property".to_string(),
                            location: Some(ComponentPropertyLocation {
                                index: property_index,
                                name: component_property_name(property).to_string(),
                                property_location: None,
                            }),
                        });
                    }
                }

                if invalid {
                    errors.push(ComponentPropertyError {
                        message:
                        "Property is declared to have a date-time value but the value is not a date-time"
                            .to_string(),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            Value::Duration => {
                let mut invalid = false;

                match property {
                    // TODO Valid property types need to be listed
                    ComponentProperty::XProperty(x_prop) => {
                        invalid = !is_duration_valued(&x_prop.value);
                    }
                    ComponentProperty::IanaProperty(iana_prop) => {
                        invalid = !is_duration_valued(&iana_prop.value);
                    }
                    _ => {
                        errors.push(ComponentPropertyError {
                            message: "Property is declared to have a duration value but that is not valid for this property".to_string(),
                            location: Some(ComponentPropertyLocation {
                                index: property_index,
                                name: component_property_name(property).to_string(),
                                property_location: None,
                            }),
                        });
                    }
                }

                if invalid {
                    errors.push(ComponentPropertyError {
                        message:
                        "Property is declared to have a duration value but the value is not a duration"
                            .to_string(),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            Value::Float => {
                let mut invalid = false;

                match property {
                    // TODO Valid property types need to be listed
                    ComponentProperty::XProperty(x_prop) => {
                        invalid = !is_float_valued(&x_prop.value);
                    }
                    ComponentProperty::IanaProperty(iana_prop) => {
                        invalid = !is_float_valued(&iana_prop.value);
                    }
                    _ => {
                        errors.push(ComponentPropertyError {
                            message: "Property is declared to have a float value but that is not valid for this property".to_string(),
                            location: Some(ComponentPropertyLocation {
                                index: property_index,
                                name: component_property_name(property).to_string(),
                                property_location: None,
                            }),
                        });
                    }
                }

                if invalid {
                    errors.push(ComponentPropertyError {
                        message:
                        "Property is declared to have a float value but the value is not a float"
                            .to_string(),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            Value::Integer => {
                let mut invalid = false;

                match property {
                    // TODO Valid property types need to be listed
                    ComponentProperty::XProperty(x_prop) => {
                        invalid = !is_integer_valued(&x_prop.value);
                    }
                    ComponentProperty::IanaProperty(iana_prop) => {
                        invalid = !is_integer_valued(&iana_prop.value);
                    }
                    _ => {
                        errors.push(ComponentPropertyError {
                            message: "Property is declared to have an integer value but that is not valid for this property".to_string(),
                            location: Some(ComponentPropertyLocation {
                                index: property_index,
                                name: component_property_name(property).to_string(),
                                property_location: None,
                            }),
                        });
                    }
                }

                if invalid {
                    errors.push(ComponentPropertyError {
                        message:
                        "Property is declared to have an integer value but the value is not an integer"
                            .to_string(),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            Value::Period => {
                let mut invalid = false;

                match property {
                    // TODO Valid property types need to be listed
                    ComponentProperty::XProperty(x_prop) => {
                        invalid = !is_period_valued(&x_prop.value);
                    }
                    ComponentProperty::IanaProperty(iana_prop) => {
                        invalid = !is_period_valued(&iana_prop.value);
                    }
                    _ => {
                        errors.push(ComponentPropertyError {
                            message: "Property is declared to have a period value but that is not valid for this property".to_string(),
                            location: Some(ComponentPropertyLocation {
                                index: property_index,
                                name: component_property_name(property).to_string(),
                                property_location: None,
                            }),
                        });
                    }
                }

                if invalid {
                    errors.push(ComponentPropertyError {
                        message:
                        "Property is declared to have a period value but the value is not a period"
                            .to_string(),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            Value::Recurrence => {
                let mut invalid = false;

                match property {
                    // TODO Valid property types need to be listed
                    ComponentProperty::XProperty(x_prop) => match is_recur_valued(&x_prop.value) {
                        Ok(rule) => match rule.to_model() {
                            Ok(rule) => {
                                validate_recurrence_rule(errors, property, &rule, property_index)?;
                            }
                            Err(e) => {
                                errors.push(ComponentPropertyError {
                                    message: format!(
                                        "Failed to convert recurrence rule to model: {}",
                                        e
                                    ),
                                    location: Some(ComponentPropertyLocation {
                                        index: property_index,
                                        name: component_property_name(property).to_string(),
                                        property_location: Some(WithinPropertyLocation::Value),
                                    }),
                                });
                            }
                        },
                        Err(_) => {
                            invalid = true;
                        }
                    },
                    ComponentProperty::IanaProperty(iana_prop) => {
                        match is_recur_valued(&iana_prop.value) {
                            Ok(rule) => match rule.to_model() {
                                Ok(rule) => {
                                    validate_recurrence_rule(
                                        errors,
                                        property,
                                        &rule,
                                        property_index,
                                    )?;
                                }
                                Err(e) => {
                                    errors.push(ComponentPropertyError {
                                        message: format!(
                                            "Failed to convert recurrence rule to model: {}",
                                            e
                                        ),
                                        location: Some(ComponentPropertyLocation {
                                            index: property_index,
                                            name: component_property_name(property).to_string(),
                                            property_location: Some(WithinPropertyLocation::Value),
                                        }),
                                    });
                                }
                            },
                            Err(_) => {
                                invalid = true;
                            }
                        }
                    }
                    _ => {
                        errors.push(ComponentPropertyError {
                            message: "Property is declared to have a recurrence value but that is not valid for this property".to_string(),
                            location: Some(ComponentPropertyLocation {
                                index: property_index,
                                name: component_property_name(property).to_string(),
                                property_location: None,
                            }),
                        });
                    }
                }

                if invalid {
                    errors.push(ComponentPropertyError {
                        message:
                        "Property is declared to have a recurrence value but the value is not a recurrence"
                            .to_string(),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            Value::Text => {
                let mut invalid = false;

                match property {
                    // TODO Valid property types need to be listed
                    ComponentProperty::XProperty(x_prop) => {
                        invalid = !is_text_valued(&x_prop.value);
                    }
                    ComponentProperty::IanaProperty(iana_prop) => {
                        invalid = !is_text_valued(&iana_prop.value);
                    }
                    _ => {
                        errors.push(ComponentPropertyError {
                            message: "Property is declared to have a text value but that is not valid for this property".to_string(),
                            location: Some(ComponentPropertyLocation {
                                index: property_index,
                                name: component_property_name(property).to_string(),
                                property_location: None,
                            }),
                        });
                    }
                }

                if invalid {
                    errors.push(ComponentPropertyError {
                        message:
                            "Property is declared to have a text value but the value is not a text"
                                .to_string(),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            Value::Time => {
                let mut invalid = false;

                match property {
                    // TODO Valid property types need to be listed
                    ComponentProperty::XProperty(x_prop) => match is_time_valued(&x_prop.value) {
                        Ok(times) => {
                            for (index, time) in times.iter().enumerate() {
                                if let Err(e) = validate_time(time) {
                                    errors.push(ComponentPropertyError {
                                        message: format!(
                                            "Found an invalid time at index {} - {:?}",
                                            index, e
                                        ),
                                        location: Some(ComponentPropertyLocation {
                                            index: property_index,
                                            name: component_property_name(property).to_string(),
                                            property_location: Some(WithinPropertyLocation::Value),
                                        }),
                                    });
                                }
                            }
                        }
                        Err(_) => {
                            invalid = true;
                        }
                    },
                    ComponentProperty::IanaProperty(iana_prop) => {
                        match is_time_valued(&iana_prop.value) {
                            Ok(times) => {
                                for (index, time) in times.iter().enumerate() {
                                    if let Err(e) = validate_time(time) {
                                        errors.push(ComponentPropertyError {
                                            message: format!(
                                                "Found an invalid time at index {} - {:?}",
                                                index, e
                                            ),
                                            location: Some(ComponentPropertyLocation {
                                                index: property_index,
                                                name: component_property_name(property).to_string(),
                                                property_location: Some(
                                                    WithinPropertyLocation::Value,
                                                ),
                                            }),
                                        });
                                    }
                                }
                            }
                            Err(_) => {
                                invalid = true;
                            }
                        }
                    }
                    _ => {
                        errors.push(ComponentPropertyError {
                            message: "Property is declared to have a time value but that is not valid for this property".to_string(),
                            location: Some(ComponentPropertyLocation {
                                index: property_index,
                                name: component_property_name(property).to_string(),
                                property_location: None,
                            }),
                        });
                    }
                }

                if invalid {
                    errors.push(ComponentPropertyError {
                        message:
                            "Property is declared to have a time value but the value is not a time"
                                .to_string(),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            Value::Uri => {
                let mut invalid = false;

                match property {
                    // TODO Valid property types need to be listed
                    ComponentProperty::XProperty(x_prop) => {
                        invalid = !is_uri_valued(&x_prop.value);
                    }
                    ComponentProperty::IanaProperty(iana_prop) => {
                        invalid = !is_uri_valued(&iana_prop.value);
                    }
                    _ => {
                        errors.push(ComponentPropertyError {
                            message: "Property is declared to have a URI value but that is not valid for this property".to_string(),
                            location: Some(ComponentPropertyLocation {
                                index: property_index,
                                name: component_property_name(property).to_string(),
                                property_location: None,
                            }),
                        });
                    }
                }

                if invalid {
                    errors.push(ComponentPropertyError {
                        message:
                            "Property is declared to have a URI value but the value is not a URI"
                                .to_string(),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            Value::UtcOffset => {
                let mut invalid = false;

                match property {
                    // TODO Valid property types need to be listed
                    ComponentProperty::XProperty(x_prop) => {
                        match is_utc_offset_valued(&x_prop.value) {
                            Ok(offset) => {
                                if let Err(e) = validate_utc_offset(&offset) {
                                    errors.push(ComponentPropertyError {
                                        message: format!("Found an invalid UTC offset - {:?}", e),
                                        location: Some(ComponentPropertyLocation {
                                            index: property_index,
                                            name: component_property_name(property).to_string(),
                                            property_location: Some(WithinPropertyLocation::Value),
                                        }),
                                    });
                                }
                            }
                            Err(_) => {
                                invalid = true;
                            }
                        }
                    }
                    ComponentProperty::IanaProperty(iana_prop) => {
                        match is_utc_offset_valued(&iana_prop.value) {
                            Ok(offset) => {
                                if let Err(e) = validate_utc_offset(&offset) {
                                    errors.push(ComponentPropertyError {
                                        message: format!("Found an invalid UTC offset - {:?}", e),
                                        location: Some(ComponentPropertyLocation {
                                            index: property_index,
                                            name: component_property_name(property).to_string(),
                                            property_location: Some(WithinPropertyLocation::Value),
                                        }),
                                    });
                                }
                            }
                            Err(_) => {
                                invalid = true;
                            }
                        }
                    }
                    _ => {
                        errors.push(ComponentPropertyError {
                            message: "Property is declared to have a UTC offset value but that is not valid for this property".to_string(),
                            location: Some(ComponentPropertyLocation {
                                index: property_index,
                                name: component_property_name(property).to_string(),
                                property_location: None,
                            }),
                        });
                    }
                }

                if invalid {
                    errors.push(ComponentPropertyError {
                        message:
                        "Property is declared to have a UTC offset value but the value is not a UTC offset"
                            .to_string(),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            Value::XName(_) | Value::IanaToken(_) => {
                // Nothing to validate, we don't know anything about the values these should take
            }
        }
    }

    Ok(())
}

fn is_boolean_valued(property_value: &str) -> bool {
    property_value.eq_ignore_ascii_case("TRUE") || property_value.eq_ignore_ascii_case("FALSE")
}

fn is_date_valued(property_value: &String) -> bool {
    let mut content = property_value.as_bytes().to_vec();
    content.push(b';');

    let result = separated_list1(char(','), prop_value_date::<Error>)(content.as_bytes());
    match result {
        Ok((rest, period)) => rest.len() == 1,
        _ => false,
    }
}

fn is_date_time_valued(property_value: &String) -> bool {
    let mut content = property_value.as_bytes().to_vec();
    content.push(b';');

    let result = separated_list1(char(','), prop_value_date_time::<Error>)(content.as_bytes());
    match result {
        Ok((rest, period)) => rest.len() == 1,
        _ => false,
    }
}

fn is_duration_valued(property_value: &String) -> bool {
    let mut content = property_value.as_bytes().to_vec();
    content.push(b';');

    let result = separated_list1(char(','), prop_value_duration::<Error>)(content.as_bytes());
    match result {
        Ok((rest, period)) => rest.len() == 1,
        _ => false,
    }
}

fn is_float_valued(property_value: &String) -> bool {
    let mut content = property_value.as_bytes().to_vec();
    content.push(b';');

    let result = separated_list1(char(','), prop_value_float::<Error>)(content.as_bytes());
    match result {
        Ok((rest, period)) => rest.len() == 1,
        _ => false,
    }
}

fn is_integer_valued(property_value: &String) -> bool {
    let mut content = property_value.as_bytes().to_vec();
    content.push(b';');

    let result = separated_list1(char(','), prop_value_integer::<Error>)(content.as_bytes());
    result.is_ok()
}

fn is_period_valued(property_value: &String) -> bool {
    let mut content = property_value.as_bytes().to_vec();
    content.push(b';');

    let result = prop_value_period::<Error>(content.as_bytes());
    match result {
        Ok((rest, period)) => rest.len() == 1,
        _ => false,
    }
}

fn is_recur_valued(property_value: &String) -> anyhow::Result<Vec<crate::parser::RecurRulePart>> {
    let mut content = property_value.as_bytes().to_vec();
    content.push(b'`');

    let result = recur::<Error>(content.as_bytes());
    match result {
        Ok((rest, rule)) if rest.len() == 1 => Ok(rule),
        _ => anyhow::bail!("Not a valid recur rule"),
    }
}

fn is_text_valued(property_value: &String) -> bool {
    let mut content = property_value.as_bytes().to_vec();
    content.push(b'\r');
    content.push(b'\n');

    let result = separated_list1(char(','), prop_value_text::<Error>)(content.as_bytes());
    match result {
        Ok((rest, _)) => rest.len() == 1,
        _ => false,
    }
}

fn is_time_valued(property_value: &String) -> anyhow::Result<Vec<crate::parser::Time>> {
    let mut content = property_value.as_bytes().to_vec();
    content.push(b';');

    let result = separated_list1(char(','), prop_value_time::<Error>)(content.as_bytes());
    match result {
        Ok((rest, times)) if rest.len() == 1 => Ok(times),
        _ => anyhow::bail!("Not a valid time"),
    }
}

fn is_uri_valued(property_value: &String) -> bool {
    let mut content = property_value.as_bytes().to_vec();
    content.push(b';');

    let result = param_value_uri::<Error>(content.as_bytes());
    match result {
        Ok((rest, _)) => rest.len() == 1,
        _ => false,
    }
}

fn is_utc_offset_valued(property_value: &String) -> anyhow::Result<crate::parser::UtcOffset> {
    let mut content = property_value.as_bytes().to_vec();
    content.push(b';');

    let result = prop_value_utc_offset::<Error>(content.as_bytes());
    match result {
        Ok((rest, offset)) if rest.len() == 1 => Ok(offset),
        _ => anyhow::bail!("Not a valid UTC offset"),
    }
}

fn validate_recurrence_rule(
    errors: &mut Vec<ComponentPropertyError>,
    property: &ComponentProperty,
    rule: &RecurrenceRule,
    property_index: usize,
) -> anyhow::Result<()> {
    if rule.parts.is_empty() {
        errors.push(ComponentPropertyError {
            message: "Recurrence rule is empty".to_string(),
            location: Some(ComponentPropertyLocation {
                index: property_index,
                name: component_property_name(property).to_string(),
                property_location: Some(WithinPropertyLocation::Value),
            }),
        });
        return Ok(());
    }

    let freq = match &rule.parts[0] {
        RecurRulePart::Freq(freq) => {
            // Frequency should be the first part, this is correct
            freq
        }
        _ => {
            errors.push(ComponentPropertyError {
                message: "Recurrence rule must start with a frequency".to_string(),
                location: Some(ComponentPropertyLocation {
                    index: property_index,
                    name: component_property_name(property).to_string(),
                    property_location: Some(WithinPropertyLocation::Value),
                }),
            });

            let maybe_freq = rule.parts.iter().find_map(|part| {
                if let RecurRulePart::Freq(freq) = part {
                    Some(freq)
                } else {
                    None
                }
            });

            match maybe_freq {
                Some(freq) => freq,
                None => {
                    errors.push(ComponentPropertyError {
                        message: "No frequency part found in recurrence rule, but it is required. This prevents the rest of the rule being checked".to_string(),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                    return Ok(());
                }
            }
        }
    };

    let mut seen_count = HashMap::<String, u32>::new();
    let add_count = |seen_count: &mut HashMap<String, u32>, key: &str| {
        *seen_count
            .entry(key.to_string())
            .and_modify(|count| *count += 1)
            .or_insert(1)
    };
    for (part_index, part) in rule.parts.iter().enumerate().skip(1) {
        match part {
            RecurRulePart::Freq(_) => {
                errors.push(ComponentPropertyError {
                    message: format!("Repeated FREQ part at index {part_index}"),
                    location: Some(ComponentPropertyLocation {
                        index: property_index,
                        name: component_property_name(property).to_string(),
                        property_location: Some(WithinPropertyLocation::Value),
                    }),
                });
            }
            RecurRulePart::Until(_) => {
                let count = add_count(&mut seen_count, "UNTIL");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated UNTIL part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                // TODO The value of the UNTIL rule part MUST have the same value type as the "DTSTART" property.
            }
            RecurRulePart::Count(_) => {
                let count = add_count(&mut seen_count, "COUNT");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated COUNT part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            RecurRulePart::Interval(_) => {
                let count = add_count(&mut seen_count, "INTERVAL");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated INTERVAL part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            RecurRulePart::BySecList(second_list) => {
                let count = add_count(&mut seen_count, "BYSECOND");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated BYSECOND part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                if !second_list.iter().all(|second| *second <= 60) {
                    errors.push(ComponentPropertyError {
                        message: format!("Invalid BYSECOND part at index {part_index}, seconds must be between 0 and 60"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                // TODO The BYSECOND, BYMINUTE and BYHOUR rule parts MUST NOT be specified when the associated "DTSTART" property has a DATE value type.
            }
            RecurRulePart::ByMinute(minute_list) => {
                let count = add_count(&mut seen_count, "BYMINUTE");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated BYMINUTE part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                if !minute_list.iter().all(|minute| *minute <= 59) {
                    errors.push(ComponentPropertyError {
                        message: format!("Invalid BYMINUTE part at index {part_index}, minutes must be between 0 and 59"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                // TODO The BYSECOND, BYMINUTE and BYHOUR rule parts MUST NOT be specified when the associated "DTSTART" property has a DATE value type.
            }
            RecurRulePart::ByHour(hour_list) => {
                let count = add_count(&mut seen_count, "BYHOUR");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated BYHOUR part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                if !hour_list.iter().all(|hour| *hour <= 23) {
                    errors.push(ComponentPropertyError {
                        message: format!("Invalid BYHOUR part at index {part_index}, hours must be between 0 and 23"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                // TODO The BYSECOND, BYMINUTE and BYHOUR rule parts MUST NOT be specified when the associated "DTSTART" property has a DATE value type.
            }
            RecurRulePart::ByDay(day_list) => {
                let count = add_count(&mut seen_count, "BYDAY");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated BYDAY part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                match freq {
                    RecurFreq::Monthly => {
                        // Offsets are permitted for this frequency
                    }
                    RecurFreq::Yearly => {
                        let is_by_week_number_specified = rule
                            .parts
                            .iter()
                            .any(|part| matches!(part, RecurRulePart::ByWeekNumber(_)));

                        if is_by_week_number_specified
                            && day_list.iter().any(|day| day.offset_weeks.is_some())
                        {
                            errors.push(ComponentPropertyError {
                                message: format!("BYDAY part at index {part_index} has a day with an offset, but the frequency is YEARLY and a BYWEEKNO part is specified"),
                                location: Some(ComponentPropertyLocation {
                                    index: property_index,
                                    name: component_property_name(property).to_string(),
                                    property_location: Some(WithinPropertyLocation::Value),
                                }),
                            });
                        }
                    }
                    _ => {
                        if day_list.iter().any(|day| day.offset_weeks.is_some()) {
                            errors.push(ComponentPropertyError {
                                message: format!("BYDAY part at index {part_index} has a day with an offset, but the frequency is not MONTHLY or YEARLY"),
                                location: Some(ComponentPropertyLocation {
                                    index: property_index,
                                    name: component_property_name(property).to_string(),
                                    property_location: Some(WithinPropertyLocation::Value),
                                }),
                            });
                        }
                    }
                }
            }
            RecurRulePart::ByMonthDay(month_day_list) => {
                let count = add_count(&mut seen_count, "BYMONTHDAY");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated BYMONTHDAY part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                if !month_day_list
                    .iter()
                    .all(|day| (-31 <= *day && *day <= -1) || (1 <= *day && *day <= 31))
                {
                    errors.push(ComponentPropertyError {
                        message: format!("Invalid BYMONTHDAY part at index {part_index}, days must be between 1 and 31, or -31 and -1"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                if freq == &RecurFreq::Weekly {
                    errors.push(ComponentPropertyError {
                        message: format!("BYMONTHDAY part at index {part_index} is not valid for a WEEKLY frequency"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            RecurRulePart::ByYearDay(year_day_list) => {
                let count = add_count(&mut seen_count, "BYYEARDAY");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated BYYEARDAY part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                if !year_day_list
                    .iter()
                    .all(|day| (-366 <= *day && *day <= -1) || (1 <= *day && *day <= 366))
                {
                    errors.push(ComponentPropertyError {
                        message: format!("Invalid BYYEARDAY part at index {part_index}, days must be between 1 and 366, or -366 and -1"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                match freq {
                    RecurFreq::Daily | RecurFreq::Weekly | RecurFreq::Monthly => {
                        errors.push(ComponentPropertyError {
                        message: format!("BYYEARDAY part at index {part_index} is not valid for a DAILY, WEEKLY or MONTHLY frequency"),
                        location: Some(ComponentPropertyLocation {
                               index: property_index,
                               name: component_property_name(property).to_string(),
                               property_location: Some(WithinPropertyLocation::Value),
                           }),
                        });
                    }
                    _ => {}
                }
            }
            RecurRulePart::ByWeekNumber(week_list) => {
                let count = add_count(&mut seen_count, "BYWEEKNO");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated BYWEEKNO part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                if !week_list
                    .iter()
                    .all(|week| (-53 <= *week && *week <= -1) || (1 <= *week && *week <= 53))
                {
                    errors.push(ComponentPropertyError {
                        message: format!("Invalid BYWEEKNO part at index {part_index}, weeks must be between 1 and 53, or -53 and -1"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                if freq != &RecurFreq::Yearly {
                    errors.push(ComponentPropertyError {
                        message: format!("BYWEEKNO part at index {part_index} is only valid for a YEARLY frequency"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            RecurRulePart::ByMonth(month_list) => {
                let count = add_count(&mut seen_count, "BYMONTH");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated BYMONTH part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            RecurRulePart::WeekStart(_) => {
                let count = add_count(&mut seen_count, "WKST");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated WKST part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                let mut is_redundant = true;
                match freq {
                    RecurFreq::Weekly => {
                        let has_non_default_interval = rule.parts.iter().any(|part| matches!(part, RecurRulePart::Interval(interval) if *interval > 1));
                        let by_day_specified = rule
                            .parts
                            .iter()
                            .any(|part| matches!(part, RecurRulePart::ByDay(_)));
                        if has_non_default_interval {
                            is_redundant = false;
                        }
                    }
                    RecurFreq::Yearly => {
                        let by_week_number_specified = rule
                            .parts
                            .iter()
                            .any(|part| matches!(part, RecurRulePart::ByWeekNumber(_)));
                        if by_week_number_specified {
                            is_redundant = false;
                        }
                    }
                    _ => {
                        // Otherwise, it's definitely redundant
                    }
                }

                if is_redundant {
                    errors.push(ComponentPropertyError {
                        message: format!("WKST part at index {part_index} is redundant"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            RecurRulePart::BySetPos(set_pos_list) => {
                let count = add_count(&mut seen_count, "BYSETPOS");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated BYSETPOS part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                if !set_pos_list.iter().all(|set_pos| {
                    (-366 <= *set_pos && *set_pos <= -1) || (1 <= *set_pos && *set_pos <= 366)
                }) {
                    errors.push(ComponentPropertyError {
                        message: format!("Invalid BYSETPOS part at index {part_index}, set positions must be between 1 and 366, or -366 and -1"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                let has_other_by_rule = rule.parts.iter().any(|part| {
                    matches!(
                        part,
                        RecurRulePart::BySecList(_)
                            | RecurRulePart::ByMinute(_)
                            | RecurRulePart::ByHour(_)
                            | RecurRulePart::ByDay(_)
                            | RecurRulePart::ByMonthDay(_)
                            | RecurRulePart::ByYearDay(_)
                            | RecurRulePart::ByWeekNumber(_)
                            | RecurRulePart::ByMonth(_)
                    )
                });
                if !has_other_by_rule {
                    errors.push(ComponentPropertyError {
                        message: format!("BYSETPOS part at index {part_index} is not valid without another BYxxx rule part"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
        }
    }

    Ok(())
}

fn validate_time(time: &crate::parser::Time) -> anyhow::Result<()> {
    if time.hour > 23 {
        anyhow::bail!("Hour must be between 0 and 23");
    }

    if time.minute > 59 {
        anyhow::bail!("Minute must be between 0 and 59");
    }

    if time.second > 60 {
        anyhow::bail!("Second must be between 0 and 60");
    }

    Ok(())
}

fn validate_utc_offset(offset: &crate::parser::UtcOffset) -> anyhow::Result<()> {
    if offset.sign < 0
        && (offset.hours == 0
            && offset.minutes == 0
            && (offset.seconds.is_none() || offset.seconds == Some(0)))
    {
        anyhow::bail!("UTC offset must have a non-zero value if it is negative");
    }

    if offset.minutes > 59 {
        anyhow::bail!("Minutes must be between 0 and 59");
    }

    Ok(())
}

#[derive(Debug)]
struct CalendarInfo {
    /// The ids of the time zones that this calendar defines.
    time_zone_ids: HashSet<String>,
    /// The method for this calendar object, if specified.
    method: Option<String>,
}

impl CalendarInfo {
    fn new(time_zone_ids: HashSet<String>) -> Self {
        CalendarInfo {
            time_zone_ids,
            method: None,
        }
    }
}

#[derive(Debug)]
struct PropertyInfo<'a> {
    /// The location that this property has been used in
    property_location: PropertyLocation,
    /// The required value type for this property
    value_type: ValueType,
    /// If the property has a VALUE parameter, regardless of whether that is valid on this
    /// property, then it will be populated here.
    declared_value_type: Option<crate::common::Value>,
    /// If the property value contains a time, then this field will be set. If that time is UTC,
    /// then this field will be set to true.
    value_is_utc: Option<bool>,
    /// This is an xProperty or ianaProperty
    is_other: bool,
    /// Information about the calendar that contains this property
    calendar_info: &'a CalendarInfo,
}

#[derive(Debug, Clone)]
enum PropertyLocation {
    Calendar,
    Event,
    ToDo,
    Journal,
    FreeBusy,
    TimeZone,
    TimeZoneComponent,
    Other,
}

impl<'a> PropertyInfo<'a> {
    fn new(
        calendar_info: &'a CalendarInfo,
        property_location: PropertyLocation,
        value_type: ValueType,
    ) -> Self {
        PropertyInfo {
            property_location,
            value_type,
            declared_value_type: None,
            value_is_utc: None,
            is_other: false,
            calendar_info,
        }
    }

    fn utc(mut self, is_utc: bool) -> Self {
        self.value_is_utc = Some(is_utc);
        self
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
    Duration,
    Date,
    DateTime,
}

fn add_to_seen(seen: &mut HashMap<String, u32>, key: &str) -> u32 {
    *seen
        .entry(key.to_string())
        .and_modify(|count| *count += 1)
        .or_insert(1)
}

fn validate_calendar_properties(
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
            location: None,
        })
    }
    if let Some(message) = check_occurrence(&seen, "VERSION", OccurrenceExpectation::Once) {
        errors.push(CalendarPropertyError {
            message,
            location: None,
        })
    }

    errors
}

#[derive(Debug, Clone, PartialEq)]
enum OccurrenceExpectation {
    Once,
    OptionalOnce,
    OptionalMany,
    Never,
}

fn check_occurrence(
    seen: &HashMap<String, u32>,
    key: &str,
    expectation: OccurrenceExpectation,
) -> Option<String> {
    match (seen.get(key), expectation) {
        (None | Some(0), OccurrenceExpectation::Once) => Some(format!("{} is required", key)),
        (Some(1), OccurrenceExpectation::Once) => None,
        (_, OccurrenceExpectation::Once) => Some(format!("{} must only appear once", key)),
        (None | Some(0) | Some(1), OccurrenceExpectation::OptionalOnce) => None,
        (_, OccurrenceExpectation::OptionalOnce) => Some(format!("{} must only appear once", key)),
        (_, OccurrenceExpectation::OptionalMany) => None,
        (None | Some(0), OccurrenceExpectation::Never) => None,
        (_, OccurrenceExpectation::Never) => Some(format!("{} is not allowed", key)),
    }
}

fn validate_params(params: &[Param], property_info: PropertyInfo) -> Vec<ParamError> {
    let mut errors = Vec::new();

    for (index, param) in params.iter().enumerate() {
        match param {
            Param::CommonName { name } => {
                validate_cn_param(&mut errors, param, index, &property_info);
            }
            Param::Other { name, .. } | Param::Others { name, .. } if name == "CN" => {
                validate_cn_param(&mut errors, param, index, &property_info);
            }
            Param::CalendarUserType { .. } => {
                validate_cu_type_param(&mut errors, param, index, &property_info);
            }
            Param::Other { name, .. } | Param::Others { name, .. } if name == "CUTYPE" => {
                validate_cu_type_param(&mut errors, param, index, &property_info);
            }
            Param::DelegatedFrom { .. } => {
                validate_delegated_from_param(&mut errors, param, index, &property_info);
            }
            Param::Other { name, .. } | Param::Others { name, .. } if name == "DELEGATED-FROM" => {
                validate_delegated_from_param(&mut errors, param, index, &property_info);
            }
            Param::DelegatedTo { .. } => {
                validate_delegated_to_param(&mut errors, param, index, &property_info);
            }
            Param::Other { name, .. } | Param::Others { name, .. } if name == "DELEGATED-TO" => {
                validate_delegated_to_param(&mut errors, param, index, &property_info);
            }
            Param::DirectoryEntryReference { .. } => {
                validate_dir_param(&mut errors, param, index, &property_info);
            }
            Param::Other { name, .. } | Param::Others { name, .. } if name == "DIR" => {
                validate_dir_param(&mut errors, param, index, &property_info);
            }
            Param::ValueType { .. } => {
                // Nothing to validate yet
            }
            Param::Encoding { .. } => {
                // Nothing further to validate
            }
            Param::FormatType { .. } => {
                // Format type is not validated by this program
            }
            Param::FreeBusyTimeType { .. } => {
                // Nothing further to validate
            }
            Param::Language { .. } => {
                // Nothing further to validate
            }
            Param::Members { .. } => {
                validate_member_param(&mut errors, param, index, &property_info);
            }
            Param::Other { name, .. } | Param::Others { name, .. } if name == "MEMBER" => {
                validate_member_param(&mut errors, param, index, &property_info);
            }
            Param::ParticipationStatus { status } => {
                validate_part_stat_param(&mut errors, param, status, index, &property_info);
            }
            Param::Range { .. } => {
                // The parser should reject wrong values for this param and the builder won't let you
                // specify a wrong value, so not useful to validate in this context.
            }
            Param::Related { .. } => {
                validate_related_param(&mut errors, param, index, &property_info);
            }
            Param::Other { name, .. } | Param::Others { name, .. } if name == "RELATED" => {
                validate_related_param(&mut errors, param, index, &property_info);
            }
            Param::RelationshipType { .. } => {
                // Nothing further to validate
            }
            Param::Role { .. } => {
                validate_role_param(&mut errors, param, index, &property_info);
            }
            Param::Other { name, .. } if name == "ROLE" => {
                validate_role_param(&mut errors, param, index, &property_info);
            }
            Param::Rsvp { .. } => {
                validate_rsvp_param(&mut errors, param, index, &property_info);
            }
            Param::Other { name, .. } if name == "RSVP" => {
                validate_rsvp_param(&mut errors, param, index, &property_info);
            }
            Param::SentBy { address } => {
                validate_sent_by_param(&mut errors, param, address, index, &property_info);
            }
            Param::Other { name, value } if name == "SENT-BY" => {
                validate_sent_by_param(&mut errors, param, value, index, &property_info);
            }
            Param::TimeZoneId { tz_id, unique } => {
                validate_time_zone_id_param(
                    &mut errors,
                    param,
                    tz_id,
                    *unique,
                    index,
                    &property_info,
                );
            }
            Param::Other { name, value } if name == "TZID" => {
                validate_time_zone_id_param(
                    &mut errors,
                    param,
                    value,
                    false,
                    index,
                    &property_info,
                );
            }
            _ => {
                unimplemented!()
            }
        }
    }

    errors
}

// RFC 5545, Section 3.2.2
fn validate_cn_param(
    errors: &mut Vec<ParamError>,
    param: &Param,
    index: usize,
    property_info: &PropertyInfo,
) {
    if !property_info.is_other && property_info.value_type != ValueType::CalendarAddress {
        errors.push(ParamError {
            index,
            name: param_name(param).to_string(),
            message: "Common name (CN) is not allowed for this property type".to_string(),
        });
    }
}

// RFC 5545, Section 3.2.3
fn validate_cu_type_param(
    errors: &mut Vec<ParamError>,
    param: &Param,
    index: usize,
    property_info: &PropertyInfo,
) {
    if !property_info.is_other && property_info.value_type != ValueType::CalendarAddress {
        errors.push(ParamError {
            index,
            name: param_name(param).to_string(),
            message: "Calendar user type (CUTYPE) is not allowed for this property type"
                .to_string(),
        });
    }
}

// RFC 5545, Section 3.2.4
fn validate_delegated_from_param(
    errors: &mut Vec<ParamError>,
    param: &Param,
    index: usize,
    property_info: &PropertyInfo,
) {
    if !property_info.is_other && property_info.value_type != ValueType::CalendarAddress {
        errors.push(ParamError {
            index,
            name: param_name(param).to_string(),
            message: "Delegated from (DELEGATED-FROM) is not allowed for this property type"
                .to_string(),
        });
    }
}

// RFC 5545, Section 3.2.5
fn validate_delegated_to_param(
    errors: &mut Vec<ParamError>,
    param: &Param,
    index: usize,
    property_info: &PropertyInfo,
) {
    if !property_info.is_other && property_info.value_type != ValueType::CalendarAddress {
        errors.push(ParamError {
            index,
            name: param_name(param).to_string(),
            message: "Delegated to (DELEGATED-TO) is not allowed for this property type"
                .to_string(),
        });
    }
}

// RFC 5545, Section 3.2.6
fn validate_dir_param(
    errors: &mut Vec<ParamError>,
    param: &Param,
    index: usize,
    property_info: &PropertyInfo,
) {
    if !property_info.is_other && property_info.value_type != ValueType::CalendarAddress {
        errors.push(ParamError {
            index,
            name: param_name(param).to_string(),
            message: "Directory entry reference (DIR) is not allowed for this property type"
                .to_string(),
        });
    }
}

// RFC 5545, Section 3.2.11
fn validate_member_param(
    errors: &mut Vec<ParamError>,
    param: &Param,
    index: usize,
    property_info: &PropertyInfo,
) {
    if !property_info.is_other && property_info.value_type != ValueType::CalendarAddress {
        errors.push(ParamError {
            index,
            name: param_name(param).to_string(),
            message: "Group or list membership (MEMBER) is not allowed for this property type"
                .to_string(),
        });
    }
}

// RFC 5545, Section 3.2.12
fn validate_part_stat_param(
    errors: &mut Vec<ParamError>,
    param: &Param,
    status: &ParticipationStatusUnknown,
    index: usize,
    property_info: &PropertyInfo,
) {
    match &property_info.property_location {
        PropertyLocation::Event => {
            match status {
                ParticipationStatusUnknown::NeedsAction
                | ParticipationStatusUnknown::Accepted
                | ParticipationStatusUnknown::Declined
                | ParticipationStatusUnknown::Tentative
                | ParticipationStatusUnknown::Delegated
                | ParticipationStatusUnknown::XName(_)
                | ParticipationStatusUnknown::IanaToken(_) => {
                    // Valid values
                }
                _ => {
                    errors.push(ParamError {
                            index,
                            name: param_name(param).to_string(),
                            message: format!("Invalid participation status (PARTSTAT) value [{status:?}] in a VEVENT component context"),
                        });
                }
            }
        }
        PropertyLocation::ToDo => {
            // This component type permits all recognized values
        }
        PropertyLocation::Journal => {
            match status {
                ParticipationStatusUnknown::NeedsAction
                | ParticipationStatusUnknown::Accepted
                | ParticipationStatusUnknown::Declined
                | ParticipationStatusUnknown::XName(_)
                | ParticipationStatusUnknown::IanaToken(_) => {
                    // Valid values
                }
                _ => {
                    errors.push(ParamError {
                            index,
                            name: param_name(param).to_string(),
                            message: format!("Invalid participation status (PARTSTAT) value [{status:?}] in a VJOURNAL component context"),
                        });
                }
            }
        }
        PropertyLocation::Other => {
            // Permit in "other", we don't know how it's being used.
        }
        location => {
            errors.push(ParamError {
                    index,
                    name: param_name(param).to_string(),
                    message: format!("Participation status (PARTSTAT) property is not expected in a [{location:?}] component context"),
                });
        }
    }

    if !property_info.is_other && property_info.value_type != ValueType::CalendarAddress {
        errors.push(ParamError {
            index,
            name: param_name(param).to_string(),
            message: "Participation status (PARTSTAT) is not allowed for this property type"
                .to_string(),
        });
    }
}

// RFC 5545, Section 3.2.14
fn validate_related_param(
    errors: &mut Vec<ParamError>,
    param: &Param,
    index: usize,
    property_info: &PropertyInfo,
) {
    if !property_info.is_other && property_info.value_type != ValueType::Duration {
        errors.push(ParamError {
            index,
            name: param_name(param).to_string(),
            message: "Related (RELATED) is not allowed for this property type".to_string(),
        });
    }
}

// RFC 5545, Section 3.2.16
fn validate_role_param(
    errors: &mut Vec<ParamError>,
    param: &Param,
    index: usize,
    property_info: &PropertyInfo,
) {
    if !property_info.is_other && property_info.value_type != ValueType::CalendarAddress {
        errors.push(ParamError {
            index,
            name: param_name(param).to_string(),
            message: "Participation role (ROLE) is not allowed for this property type".to_string(),
        });
    }
}

// RFC 5545, Section 3.2.17
fn validate_rsvp_param(
    errors: &mut Vec<ParamError>,
    param: &Param,
    index: usize,
    property_info: &PropertyInfo,
) {
    if !property_info.is_other && property_info.value_type != ValueType::CalendarAddress {
        errors.push(ParamError {
            index,
            name: param_name(param).to_string(),
            message: "RSVP expectation (RSVP) is not allowed for this property type".to_string(),
        });
    }
}

// RFC 5545, Section 3.2.18
fn validate_sent_by_param(
    errors: &mut Vec<ParamError>,
    param: &Param,
    address: &str,
    index: usize,
    property_info: &PropertyInfo,
) {
    if !property_info.is_other && property_info.value_type != ValueType::CalendarAddress {
        errors.push(ParamError {
            index,
            name: param_name(param).to_string(),
            message: "Sent by (SENT-BY) is not allowed for this property type".to_string(),
        });
    }

    if !address.starts_with("mailto:") {
        errors.push(ParamError {
            index,
            name: param_name(param).to_string(),
            message: "Sent by (SENT-BY) must be a 'mailto:' URI".to_string(),
        });
    }
}

// RFC 5545, Section 3.2.19
fn validate_time_zone_id_param(
    errors: &mut Vec<ParamError>,
    param: &Param,
    tz_id: &String,
    unique: bool,
    index: usize,
    property_info: &PropertyInfo,
) {
    if property_info.value_type == ValueType::Date {
        errors.push(ParamError {
            index,
            name: param_name(param).to_string(),
            message: "Time zone ID (TZID) is not allowed for the property value type DATE"
                .to_string(),
        });
        return;
    }

    if !unique && !property_info.calendar_info.time_zone_ids.contains(tz_id) {
        errors.push(ParamError {
            index,
            name: param_name(param).to_string(),
            message: format!("Required time zone ID [{tz_id}] is not defined in the calendar"),
        });
    }

    if let Some(true) = property_info.value_is_utc {
        errors.push(ParamError {
            index,
            name: param_name(param).to_string(),
            message: "Time zone ID (TZID) cannot be specified on a property with a UTC time"
                .to_string(),
        });
    }
}

fn get_declared_value_type(property: &ComponentProperty) -> Option<(Value, usize)> {
    property
        .params()
        .iter()
        .enumerate()
        .find_map(|(index, param)| {
            if let Param::ValueType { value } = param {
                return Some((value.clone(), index));
            }

            None
        })
}

fn calendar_property_name(property: &CalendarProperty) -> &str {
    match property {
        CalendarProperty::Version { .. } => "VERSION",
        CalendarProperty::ProductId(_) => "PRODID",
        CalendarProperty::CalendarScale(_) => "CALSCALE",
        CalendarProperty::Method(_) => "METHOD",
        CalendarProperty::XProperty(x_prop) => &x_prop.name,
        CalendarProperty::IanaProperty(iana_prop) => &iana_prop.name,
    }
}

fn component_property_name(property: &ComponentProperty) -> &str {
    match property {
        ComponentProperty::DateTimeStamp(_) => "DTSTAMP",
        ComponentProperty::UniqueIdentifier(_) => "UID",
        ComponentProperty::Description(_) => "DESCRIPTION",
        ComponentProperty::Attendee(_) => "ATTENDEE",
        ComponentProperty::Organizer(_) => "ORGANIZER",
        ComponentProperty::TimeZoneId(_) => "TZID",
        ComponentProperty::DateTimeStart(_) => "DTSTART",
        ComponentProperty::XProperty(x_prop) => &x_prop.name,
        ComponentProperty::IanaProperty(iana_prop) => &iana_prop.name,
        _ => unimplemented!(),
    }
}

fn component_name(component: &CalendarComponent) -> &str {
    match component {
        CalendarComponent::Event(_) => "VEVENT",
        CalendarComponent::ToDo(_) => "VTODO",
        CalendarComponent::Journal(_) => "VJOURNAL",
        CalendarComponent::TimeZone(_) => "VTIMEZONE",
        CalendarComponent::XComponent(component) => &component.name,
        _ => unimplemented!(),
    }
}

fn param_name(param: &Param) -> &str {
    match param {
        Param::CommonName { .. } => "CN",
        Param::CalendarUserType { .. } => "CUTYPE",
        Param::DelegatedFrom { .. } => "DELEGATED-FROM",
        Param::DelegatedTo { .. } => "DELEGATED-TO",
        Param::DirectoryEntryReference { .. } => "DIR",
        Param::ValueType { .. } => "VALUE",
        Param::Encoding { .. } => "ENCODING",
        Param::FormatType { .. } => "FMTTYPE",
        Param::FreeBusyTimeType { .. } => "FBTYPE",
        Param::Language { .. } => "LANGUAGE",
        Param::Members { .. } => "MEMBER",
        Param::ParticipationStatus { .. } => "PARTSTAT",
        Param::Related { .. } => "RELATED",
        Param::Role { .. } => "ROLE",
        Param::Rsvp { .. } => "RSVP",
        Param::SentBy { .. } => "SENT-BY",
        Param::TimeZoneId { .. } => "TZID",
        Param::Other { name, .. } => name,
        Param::Others { name, .. } => name,
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
PRODID:test\r\n\
VERSION;CN=hello:2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In calendar property \"VERSION\" at index 1: Common name (CN) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn common_name_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DESCRIPTION;CN=hello:some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Common name (CN) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn cu_type_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION;CUTYPE=INDIVIDUAL:2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In calendar property \"VERSION\" at index 1: Calendar user type (CUTYPE) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn cu_type_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DESCRIPTION;CUTYPE=INDIVIDUAL:some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Calendar user type (CUTYPE) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn delegated_from_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION;DELEGATED-FROM=\"mailto:hello@test.net\":2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In calendar property \"VERSION\" at index 1: Delegated from (DELEGATED-FROM) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn delegated_from_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DESCRIPTION;DELEGATED-FROM=\"mailto:hello@test.net\":some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Delegated from (DELEGATED-FROM) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn delegated_to_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION;DELEGATED-TO=\"mailto:hello@test.net\":2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In calendar property \"VERSION\" at index 1: Delegated to (DELEGATED-TO) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn delegated_to_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DESCRIPTION;DELEGATED-TO=\"mailto:hello@test.net\":some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Delegated to (DELEGATED-TO) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn dir_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION;DIR=\"ldap://example.com:6666/o=ABC\":2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In calendar property \"VERSION\" at index 1: Directory entry reference (DIR) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn dir_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DESCRIPTION;DIR=\"ldap://example.com:6666/o=ABC\":some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Directory entry reference (DIR) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn encoding_not_set_on_binary_value() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DESCRIPTION;VALUE=BINARY:eA==\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Property is declared to have a binary value but no encoding is set, must be set to BASE64", errors.first().unwrap().to_string());
    }

    #[test]
    fn encoding_set_to_8bit_on_binary_value() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DESCRIPTION;VALUE=BINARY;ENCODING=8BIT:eA==\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Property is declared to have a binary value but the encoding is set to 8BIT, instead of BASE64", errors.first().unwrap().to_string());
    }

    #[test]
    fn member_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION;MEMBER=\"mailto:hello@test.net\":2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In calendar property \"VERSION\" at index 1: Group or list membership (MEMBER) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn member_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DESCRIPTION;MEMBER=\"mailto:hello@test.net\":some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Group or list membership (MEMBER) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn part_stat_wrong_value_in_event() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
ATTENDEE;PARTSTAT=COMPLETED:mailto:hello@test.net\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"ATTENDEE\" at index 2: Invalid participation status (PARTSTAT) value [Completed] in a VEVENT component context", errors.first().unwrap().to_string());
    }

    #[test]
    fn part_stat_wrong_value_in_journal() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
BEGIN:VJOURNAL\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
ATTENDEE;PARTSTAT=IN-PROCESS:mailto:hello@test.net\r\n\
END:VJOURNAL\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_single_error(&errors,"In component \"VJOURNAL\" at index 0, in component property \"ATTENDEE\" at index 2: Invalid participation status (PARTSTAT) value [InProcess] in a VJOURNAL component context");
    }

    #[test]
    fn part_stat_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DESCRIPTION;PARTSTAT=NEEDS-ACTION:some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Participation status (PARTSTAT) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn related_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION;RELATED=END:2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In calendar property \"VERSION\" at index 1: Related (RELATED) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn related_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DESCRIPTION;RELATED=START:some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Related (RELATED) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn role_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION;ROLE=CHAIR:2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In calendar property \"VERSION\" at index 1: Participation role (ROLE) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn role_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DESCRIPTION;ROLE=CHAIN:some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Participation role (ROLE) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn rsvp_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION;RSVP=TRUE:2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In calendar property \"VERSION\" at index 1: RSVP expectation (RSVP) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn rsvp_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DESCRIPTION;RSVP=FALSE:some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: RSVP expectation (RSVP) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn sent_by_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION;SENT-BY=\"mailto:hello@test.net\":2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In calendar property \"VERSION\" at index 1: Sent by (SENT-BY) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn sent_by_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DESCRIPTION;SENT-BY=\"mailto:hello@test.net\":some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_single_error(&errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Sent by (SENT-BY) is not allowed for this property type");
    }

    #[test]
    fn sent_by_with_invalid_protocol() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
ORGANIZER;SENT-BY=\"http:hello@test.net\":mailto:world@test.net\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_single_error(&errors, "In component \"VEVENT\" at index 0, in component property \"ORGANIZER\" at index 2: Sent by (SENT-BY) must be a 'mailto:' URI");
    }

    #[test]
    fn missing_tz_id() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DTSTART;TZID=missing:20240606T220000\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_single_error(&errors, "In component \"VEVENT\" at index 0, in component property \"DTSTART\" at index 2: Required time zone ID [missing] is not defined in the calendar");
    }

    #[test]
    fn tz_id_specified_on_utc_start() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VTIMEZONE\r\n\
TZID:any\r\n\
END:VTIMEZONE\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DTSTART;TZID=any:20240606T220000Z\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 1, in component property \"DTSTART\" at index 2: Time zone ID (TZID) cannot be specified on a property with a UTC time", errors.first().unwrap().to_string());
    }

    #[test]
    fn tz_id_specified_on_date_start() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
BEGIN:VTIMEZONE\r\n\
TZID:any\r\n\
END:VTIMEZONE\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DTSTART;TZID=any:20240606\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 1, in component property \"DTSTART\" at index 2: Time zone ID (TZID) is not allowed for the property value type DATE", errors.first().unwrap().to_string());
    }

    #[test]
    fn x_prop_declares_boolean_but_is_not_boolean() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
X-HELLO;VALUE=BOOLEAN:123\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"X-HELLO\" at index 2: Property is declared to have a boolean value but the value is not a boolean", errors.first().unwrap().to_string());
    }

    #[test]
    fn iana_prop_declares_boolean_but_is_not_boolean() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
HELLO;VALUE=BOOLEAN:123\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"HELLO\" at index 2: Property is declared to have a boolean value but the value is not a boolean", errors.first().unwrap().to_string());
    }

    #[test]
    fn x_prop_declares_date() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
X-HELLO;VALUE=DATE:19900101\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn x_prop_declares_date_and_is_multi() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
X-HELLO;VALUE=DATE:19900101,19920101\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn x_prop_declares_date_and_is_not() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
X-HELLO;VALUE=DATE:TRUE\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"X-HELLO\" at index 2: Property is declared to have a date value but the value is not a date", errors.first().unwrap().to_string());
    }

    fn validate_content(content: &str) -> Vec<ICalendarError> {
        let (rem, object) = crate::parser::ical_object::<Error>(content.as_bytes()).unwrap();
        check_rem(rem, 0);

        validate_model(object.to_model().unwrap()).unwrap()
    }

    fn assert_single_error(errors: &[ICalendarError], msg: &str) {
        if errors.len() != 1 {
            panic!(
                "Expected a single error, but got: {:?}",
                errors.iter().map(|e| e.to_string()).collect::<Vec<_>>()
            );
        }

        assert_eq!(msg, errors.first().unwrap().to_string());
    }
}
