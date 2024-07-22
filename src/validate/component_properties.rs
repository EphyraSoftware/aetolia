use crate::model::{Action, ComponentProperty, Param};
use crate::validate::value::check_declared_value;
use crate::validate::{
    check_occurrence, component_property_name, validate_params, CalendarInfo,
    ComponentPropertyError, OccurrenceExpectation, PropertyInfo, PropertyKind, PropertyLocation,
    ValueType,
};
use std::collections::HashMap;

macro_rules! check_component_property_occurrence {
    ($errors:ident, $seen:ident, $property:ident, $index:ident, $occur:expr) => {
        let name = $crate::validate::component_property_name($property);
        let count = $crate::validate::add_to_seen(&mut $seen, name);
        if let Some(message) = $crate::validate::check_occurrence(&$seen, name, $occur.clone()) {
            $errors.push(ComponentPropertyError {
                message,
                location: Some($crate::validate::ComponentPropertyLocation {
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

pub(super) fn validate_component_properties(
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
        PropertyLocation::TimeZone
        | PropertyLocation::TimeZoneComponent
        | PropertyLocation::Alarm => OccurrenceExpectation::Never,
        PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };

    let uid_occurrence_expectation = match property_location {
        PropertyLocation::Event
        | PropertyLocation::ToDo
        | PropertyLocation::Journal
        | PropertyLocation::FreeBusy => OccurrenceExpectation::Once,
        PropertyLocation::TimeZone
        | PropertyLocation::TimeZoneComponent
        | PropertyLocation::Alarm => OccurrenceExpectation::Never,
        PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
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
        PropertyLocation::TimeZone | PropertyLocation::Alarm => OccurrenceExpectation::Never,
        PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };

    let tz_id_occurrence_expectation = match property_location {
        PropertyLocation::Event
        | PropertyLocation::ToDo
        | PropertyLocation::Journal
        | PropertyLocation::FreeBusy
        | PropertyLocation::TimeZoneComponent
        | PropertyLocation::Alarm => OccurrenceExpectation::Never,
        PropertyLocation::TimeZone => OccurrenceExpectation::Once,
        PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };

    let tz_offset_to_occurrence_expectation = match property_location {
        PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Once,
        PropertyLocation::Event
        | PropertyLocation::ToDo
        | PropertyLocation::Journal
        | PropertyLocation::FreeBusy
        | PropertyLocation::TimeZone
        | PropertyLocation::Alarm => OccurrenceExpectation::Never,
        PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };

    let tz_offset_from_occurrence_expectation = match property_location {
        PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Once,
        PropertyLocation::Event
        | PropertyLocation::ToDo
        | PropertyLocation::Journal
        | PropertyLocation::FreeBusy
        | PropertyLocation::TimeZone
        | PropertyLocation::Alarm => OccurrenceExpectation::Never,
        PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };

    let mut alarm_action = None;
    let action_occurrence_expectation = match property_location {
        PropertyLocation::Alarm => {
            let actions = properties
                .iter()
                .filter_map(|p| {
                    if let ComponentProperty::Action(action) = p {
                        Some(action.value.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            if actions.len() == 1 {
                alarm_action = actions.first().cloned();
                OccurrenceExpectation::Once
            } else {
                errors.push(ComponentPropertyError {
                    message: format!(
                        "Required exactly one ACTION property but found {}",
                        actions.len()
                    ),
                    location: None,
                });
                return Ok(errors);
            }
        }
        PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };

    let trigger_occurrence_expectation = match property_location {
        PropertyLocation::Alarm => OccurrenceExpectation::Once,
        PropertyLocation::Event
        | PropertyLocation::ToDo
        | PropertyLocation::Journal
        | PropertyLocation::FreeBusy
        | PropertyLocation::TimeZone
        | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
        PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };

    let description_occurrence_expectation = match property_location {
        PropertyLocation::Event | PropertyLocation::ToDo => OccurrenceExpectation::OptionalOnce,
        PropertyLocation::FreeBusy
        | PropertyLocation::TimeZone
        | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
        PropertyLocation::Alarm => match alarm_action.clone().expect("Always present for an alarm")
        {
            Action::Display | Action::Email => OccurrenceExpectation::Once,
            Action::Audio => OccurrenceExpectation::Never,
            _ => OccurrenceExpectation::OptionalMany,
        },
        PropertyLocation::Journal | PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };

    let summary_occurrence_expectation = match property_location {
        PropertyLocation::Event | PropertyLocation::ToDo | PropertyLocation::Journal => {
            OccurrenceExpectation::OptionalOnce
        }
        PropertyLocation::FreeBusy
        | PropertyLocation::TimeZone
        | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
        PropertyLocation::Alarm => match alarm_action.clone().expect("Always present for an alarm")
        {
            Action::Email => OccurrenceExpectation::Once,
            Action::Audio | Action::Display => OccurrenceExpectation::Never,
            _ => OccurrenceExpectation::OptionalMany,
        },
        PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };

    let attendee_occurrence_expectation = match property_location {
        PropertyLocation::Alarm => match alarm_action.clone().expect("Always present for an alarm")
        {
            Action::Email => OccurrenceExpectation::OnceOrMany,
            Action::Audio | Action::Display => OccurrenceExpectation::Never,
            _ => OccurrenceExpectation::OptionalMany,
        },
        PropertyLocation::Event
        | PropertyLocation::ToDo
        | PropertyLocation::Journal
        | PropertyLocation::FreeBusy => OccurrenceExpectation::OptionalMany,
        PropertyLocation::TimeZone | PropertyLocation::TimeZoneComponent => {
            OccurrenceExpectation::Never
        }
        PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };

    let mut has_dt_start = false;
    let mut has_dt_end = false;
    let mut has_duration = false;
    let mut has_due = false;
    let mut has_repeat = false;

    let mut seen = HashMap::<String, u32>::new();
    for (index, property) in properties.iter().enumerate() {
        check_declared_value(&mut errors, property, index)?;

        let do_validate_params = |errors: &mut Vec<ComponentPropertyError>,
                                  property_info: PropertyInfo,
                                  params: &[Param]| {
            errors.extend_from_slice(
                ComponentPropertyError::many_from_param_errors(
                    validate_params(params, property_info),
                    index,
                    component_property_name(property).to_string(),
                )
                .as_slice(),
            );
        };

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
                    PropertyKind::DateTimeStart,
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
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    description_occurrence_expectation
                );

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::Description,
                    ValueType::Text,
                );
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
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                    PropertyKind::Organizer,
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
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    summary_occurrence_expectation
                );
            }
            ComponentProperty::TimeTransparency(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event => OccurrenceExpectation::OptionalOnce,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                    PropertyLocation::Event | PropertyLocation::ToDo | PropertyLocation::Alarm => {
                        OccurrenceExpectation::OptionalOnce
                    }
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );
            }
            ComponentProperty::Attach(attach) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal => OccurrenceExpectation::OptionalMany,
                    PropertyLocation::Alarm => {
                        match alarm_action.clone().expect("Always present for an alarm") {
                            Action::Audio => OccurrenceExpectation::OptionalOnce,
                            Action::Email => OccurrenceExpectation::OptionalMany,
                            Action::Display => OccurrenceExpectation::Never,
                            _ => OccurrenceExpectation::OptionalMany,
                        }
                    }
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
                };
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    occurrence_expectation
                );

                do_validate_params(
                    &mut errors,
                    PropertyInfo::new(
                        calendar_info,
                        property_location.clone(),
                        PropertyKind::Attach,
                        ValueType::Binary,
                    ),
                    &attach.params,
                );
            }
            ComponentProperty::Attendee(attendee) => {
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    attendee_occurrence_expectation
                );

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::Attendee,
                    ValueType::CalendarAddress,
                );
                do_validate_params(&mut errors, property_info, &attendee.params);
            }
            ComponentProperty::Categories(categories) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal => OccurrenceExpectation::OptionalMany,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                    PropertyKind::Categories,
                    ValueType::Text,
                );
                do_validate_params(&mut errors, property_info, &categories.params);
            }
            ComponentProperty::Comment(comment) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal
                    | PropertyLocation::FreeBusy
                    | PropertyLocation::TimeZoneComponent => OccurrenceExpectation::OptionalMany,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                    PropertyKind::Comment,
                    ValueType::Text,
                );
                do_validate_params(&mut errors, property_info, &comment.params);
            }
            ComponentProperty::Contact(_) => {
                let occurrence_expectation = match property_location {
                    PropertyLocation::FreeBusy => OccurrenceExpectation::OptionalOnce,
                    PropertyLocation::Event
                    | PropertyLocation::ToDo
                    | PropertyLocation::Journal => OccurrenceExpectation::OptionalMany,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                    PropertyLocation::ToDo => OccurrenceExpectation::OptionalOnce,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                    PropertyLocation::ToDo => OccurrenceExpectation::OptionalOnce,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                    PropertyLocation::ToDo => OccurrenceExpectation::OptionalOnce,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                    PropertyLocation::FreeBusy => OccurrenceExpectation::OptionalMany,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::TimeZoneId,
                    ValueType::Text,
                );
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
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    action_occurrence_expectation
                );
            }
            ComponentProperty::Trigger(_) => {
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    trigger_occurrence_expectation
                );
            }
            ComponentProperty::Repeat(_) => {
                has_repeat = true;

                let occurrence_expectation = match property_location {
                    PropertyLocation::Alarm => OccurrenceExpectation::OptionalOnce,
                    PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
                    _ => OccurrenceExpectation::Never,
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
    if let Some(message) = check_occurrence(&seen, "ACTION", action_occurrence_expectation) {
        errors.push(ComponentPropertyError {
            message,
            location: None,
        });
    }
    if let Some(message) = check_occurrence(&seen, "TRIGGER", trigger_occurrence_expectation) {
        errors.push(ComponentPropertyError {
            message,
            location: None,
        });
    }
    if let Some(message) =
        check_occurrence(&seen, "DESCRIPTION", description_occurrence_expectation)
    {
        errors.push(ComponentPropertyError {
            message,
            location: None,
        });
    }
    if let Some(message) = check_occurrence(&seen, "SUMMARY", summary_occurrence_expectation) {
        errors.push(ComponentPropertyError {
            message,
            location: None,
        });
    }
    if let Some(message) = check_occurrence(&seen, "ATTENDEE", attendee_occurrence_expectation) {
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
        PropertyLocation::Alarm => {
            if (has_duration && !has_repeat) || (!has_duration && has_repeat) {
                errors.push(ComponentPropertyError {
                    message: "DURATION and REPEAT properties must be present together".to_string(),
                    location: None,
                });
            }
        }
        _ => {}
    }

    Ok(errors)
}
