use crate::common::{Status, Value};
use crate::model::{
    Action, ComponentProperty, DateTimeCompletedProperty, DateTimeDueProperty, DateTimeEndProperty,
    DateTimeStampProperty, DateTimeStartProperty, DurationProperty, FreeBusyTimeProperty,
    LastModifiedProperty, Param, PeriodEnd, StatusProperty,
};
use crate::validate::value::check_declared_value;
use crate::validate::{
    check_occurrence, component_property_name, get_declared_value_type, validate_params,
    CalendarInfo, ComponentPropertyError, ComponentPropertyLocation, OccurrenceExpectation,
    PropertyInfo, PropertyKind, PropertyLocation, ValueType, WithinPropertyLocation,
};
use std::cmp::Ordering;
use std::collections::HashMap;

macro_rules! check_component_property_occurrence {
    ($errors:ident, $seen:ident, $property:ident, $index:ident, $occur:expr) => {
        let name = $crate::validate::component_property_name($property);
        $crate::validate::add_to_seen(&mut $seen, name);
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
            ComponentProperty::DateTimeStamp(date_time_stamp) => {
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    dt_stamp_occurrence_expectation.clone()
                );

                validate_date_time_stamp(&mut errors, date_time_stamp, index);

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::DateTimeStamp,
                    ValueType::DateTime,
                );
                do_validate_params(&mut errors, property_info, &date_time_stamp.params);
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

                validate_date_time_start(
                    &mut errors,
                    date_time_start,
                    index,
                    property_location.clone(),
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
                do_validate_params(&mut errors, property_info, &date_time_start.params);
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
            ComponentProperty::DateTimeCreated(date_time_created) => {
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

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::DateTimeCreated,
                    ValueType::DateTime,
                );
                do_validate_params(&mut errors, property_info, &date_time_created.params);
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
                do_validate_params(&mut errors, property_info, &description.params);
            }
            ComponentProperty::GeographicPosition(geographic_position) => {
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

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::GeographicPosition,
                    ValueType::Float,
                );
                do_validate_params(&mut errors, property_info, &geographic_position.params);
            }
            ComponentProperty::LastModified(last_modified) => {
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

                validate_last_modified(&mut errors, last_modified, index);

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::LastModified,
                    ValueType::DateTime,
                );
                do_validate_params(&mut errors, property_info, &last_modified.params);
            }
            ComponentProperty::Location(location) => {
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

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::Location,
                    ValueType::Text,
                );
                do_validate_params(&mut errors, property_info, &location.params);
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
            ComponentProperty::Priority(priority) => {
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

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::Priority,
                    ValueType::Integer,
                );
                do_validate_params(&mut errors, property_info, &priority.params);
            }
            ComponentProperty::Sequence(sequence) => {
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

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::Sequence,
                    ValueType::Integer,
                );
                do_validate_params(&mut errors, property_info, &sequence.params);
            }
            ComponentProperty::Status(status) => {
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

                validate_status(&mut errors, status, index, property_location.clone());

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::Status,
                    ValueType::Text,
                );
                do_validate_params(&mut errors, property_info, &status.params);
            }
            ComponentProperty::Summary(summary) => {
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    summary_occurrence_expectation
                );

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::Summary,
                    ValueType::Text,
                );
                do_validate_params(&mut errors, property_info, &summary.params);
            }
            ComponentProperty::TimeTransparency(time_transparency) => {
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

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::TimeTransparency,
                    ValueType::Text,
                );
                do_validate_params(&mut errors, property_info, &time_transparency.params);
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
            ComponentProperty::RecurrenceId(recurrence_id) => {
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

                let maybe_dt_start = properties.iter().find_map(|p| match p {
                    ComponentProperty::DateTimeStart(dt_start) => Some(dt_start),
                    _ => None,
                });

                let dt_start_type = maybe_dt_start
                    .and_then(|dt_start| {
                        get_declared_value_type(&ComponentProperty::DateTimeStart(dt_start.clone()))
                    })
                    .map(|v| v.0)
                    .unwrap_or(Value::DateTime);

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::RecurrenceId,
                    if dt_start_type == Value::Date {
                        ValueType::Date
                    } else {
                        ValueType::DateTime
                    },
                );
                do_validate_params(&mut errors, property_info, &recurrence_id.params);
            }
            ComponentProperty::RecurrenceRule(recurrence_rule) => {
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

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::RecurrenceRule,
                    ValueType::Recurrence,
                );
                do_validate_params(&mut errors, property_info, &recurrence_rule.params);
            }
            ComponentProperty::DateTimeEnd(date_time_end) => {
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

                let maybe_dt_start = properties.iter().find_map(|p| match p {
                    ComponentProperty::DateTimeStart(dt_start) => Some(dt_start),
                    _ => None,
                });

                validate_date_time_end(
                    &mut errors,
                    date_time_end,
                    maybe_dt_start,
                    index,
                    property_location.clone(),
                );

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::DateTimeEnd,
                    if date_time_end.time.is_some() {
                        ValueType::DateTime
                    } else {
                        ValueType::Date
                    },
                );
                do_validate_params(&mut errors, property_info, &date_time_end.params);
            }
            ComponentProperty::Duration(duration) => {
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

                let maybe_dt_start = properties.iter().find_map(|p| match p {
                    ComponentProperty::DateTimeStart(dt_start) => Some(dt_start),
                    _ => None,
                });
                validate_duration_property(
                    &mut errors,
                    duration,
                    maybe_dt_start,
                    index,
                    property_location.clone(),
                );

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::Duration,
                    ValueType::Duration,
                );
                do_validate_params(&mut errors, property_info, &duration.params);
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
            ComponentProperty::Contact(contact) => {
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

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::Contact,
                    ValueType::Text,
                );
                do_validate_params(&mut errors, property_info, &contact.params);
            }
            ComponentProperty::ExceptionDateTimes(exception_date_times) => {
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
                    PropertyKind::ExceptionDateTimes,
                    ValueType::DateTime,
                );
                do_validate_params(&mut errors, property_info, &exception_date_times.params);
            }
            ComponentProperty::RequestStatus(request_status) => {
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

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::RequestStatus,
                    ValueType::Text,
                );
                do_validate_params(&mut errors, property_info, &request_status.params);
            }
            ComponentProperty::RelatedTo(related_to) => {
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
                    PropertyKind::Related,
                    ValueType::Text,
                );
                do_validate_params(&mut errors, property_info, &related_to.params);
            }
            ComponentProperty::Resources(resources) => {
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

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::Resources,
                    ValueType::Text,
                );
                do_validate_params(&mut errors, property_info, &resources.params);
            }
            p @ ComponentProperty::RecurrenceDateTimes(recurrence_date_times) => {
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

                let value_type = get_declared_value_type(p)
                    .map(|v| v.0)
                    .unwrap_or(Value::DateTime);

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::RecurrenceDateTimes,
                    match value_type {
                        Value::Date => ValueType::Date,
                        Value::Period => ValueType::Period,
                        // Either a DATE-TIME or something invalid, which will get checked separately.
                        _ => ValueType::DateTime,
                    },
                );
                do_validate_params(&mut errors, property_info, &recurrence_date_times.params);
            }
            ComponentProperty::DateTimeCompleted(date_time_completed) => {
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

                validate_date_time_completed(&mut errors, date_time_completed, index);

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::DateTimeCompleted,
                    ValueType::DateTime,
                );
                do_validate_params(&mut errors, property_info, &date_time_completed.params);
            }
            ComponentProperty::PercentComplete(percent_complete) => {
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

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::PercentComplete,
                    ValueType::Integer,
                );
                do_validate_params(&mut errors, property_info, &percent_complete.params);
            }
            ComponentProperty::DateTimeDue(date_time_due) => {
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

                let maybe_dt_start = properties.iter().find_map(|p| match p {
                    ComponentProperty::DateTimeStart(dt_start) => Some(dt_start),
                    _ => None,
                });

                validate_date_time_due(
                    &mut errors,
                    date_time_due,
                    maybe_dt_start,
                    index,
                    property_location.clone(),
                );

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::DateTimeDue,
                    if date_time_due.time.is_some() {
                        ValueType::DateTime
                    } else {
                        ValueType::Date
                    },
                );
                do_validate_params(&mut errors, property_info, &date_time_due.params);
            }
            ComponentProperty::FreeBusyTime(free_busy_time) => {
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

                validate_free_busy_time(&mut errors, free_busy_time, index);

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::FreeBusyTime,
                    ValueType::Period,
                );
                do_validate_params(&mut errors, property_info, &free_busy_time.params);
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
                do_validate_params(&mut errors, property_info, &time_zone_id.params);
            }
            ComponentProperty::TimeZoneUrl(time_zone_url) => {
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

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::TimeZoneUrl,
                    ValueType::Uri,
                );
                do_validate_params(&mut errors, property_info, &time_zone_url.params);
            }
            ComponentProperty::TimeZoneOffsetTo(time_zone_offset_to) => {
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    tz_offset_to_occurrence_expectation
                );

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::TimeZoneOffsetTo,
                    ValueType::UtcOffset,
                );
                do_validate_params(&mut errors, property_info, &time_zone_offset_to.params);
            }
            ComponentProperty::TimeZoneOffsetFrom(time_zone_offset_from) => {
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    tz_offset_from_occurrence_expectation
                );

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::TimeZoneOffsetFrom,
                    ValueType::UtcOffset,
                );
                do_validate_params(&mut errors, property_info, &time_zone_offset_from.params);
            }
            ComponentProperty::TimeZoneName(time_zone_name) => {
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

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::TimeZoneName,
                    ValueType::Text,
                );
                do_validate_params(&mut errors, property_info, &time_zone_name.params);
            }
            ComponentProperty::Action(action) => {
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    action_occurrence_expectation
                );

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::Action,
                    ValueType::Text,
                );
                do_validate_params(&mut errors, property_info, &action.params);
            }
            ComponentProperty::Trigger(trigger) => {
                check_component_property_occurrence!(
                    errors,
                    seen,
                    property,
                    index,
                    trigger_occurrence_expectation
                );

                let value_type = get_declared_value_type(property)
                    .map(|v| v.0)
                    .unwrap_or(Value::Duration);

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::Trigger,
                    match value_type {
                        Value::DateTime => ValueType::DateTime,
                        // Either a duration, or invalid which will be caught separately.
                        _ => ValueType::Duration,
                    },
                );
                do_validate_params(&mut errors, property_info, trigger.params());
            }
            ComponentProperty::Repeat(repeat) => {
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

                let property_info = PropertyInfo::new(
                    calendar_info,
                    property_location.clone(),
                    PropertyKind::Repeat,
                    ValueType::Integer,
                );
                do_validate_params(&mut errors, property_info, &repeat.params);
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
fn validate_duration_property(
    errors: &mut Vec<ComponentPropertyError>,
    duration_property: &DurationProperty,
    maybe_dt_start: Option<&DateTimeStartProperty>,
    index: usize,
    property_location: PropertyLocation,
) {
    match property_location {
        PropertyLocation::Event | PropertyLocation::ToDo => {
            if let Some(dt_start) = maybe_dt_start {
                if dt_start.time.is_none()
                    && duration_property.duration.weeks.is_none()
                    && duration_property.duration.days.is_none()
                {
                    errors.push(ComponentPropertyError {
                            message: "DURATION must have at least one of weeks or days when DTSTART is a date".to_string(),
                            location: Some(ComponentPropertyLocation {
                                index,
                                name: "DURATION".to_string(),
                                property_location: Some(WithinPropertyLocation::Value),
                            }),
                        });
                }
            }
        }
        _ => {
            // Check is not relevant here
        }
    }
}

/// RFC 5545, 3.8.2.1
fn validate_date_time_completed(
    errors: &mut Vec<ComponentPropertyError>,
    date_time_completed_property: &DateTimeCompletedProperty,
    index: usize,
) {
    if !date_time_completed_property.is_utc {
        errors.push(ComponentPropertyError {
            message: "DTEND must be a UTC date-time".to_string(),
            location: Some(ComponentPropertyLocation {
                index,
                name: "DTEND".to_string(),
                property_location: Some(WithinPropertyLocation::Value),
            }),
        });
    }
}

/// RFC 5545, 3.8.2.2
fn validate_date_time_end(
    errors: &mut Vec<ComponentPropertyError>,
    date_time_end_property: &DateTimeEndProperty,
    maybe_dt_start: Option<&DateTimeStartProperty>,
    index: usize,
    property_location: PropertyLocation,
) {
    // For a VEVENT, the date/date-time types must match at the start and end
    if property_location == PropertyLocation::Event {
        if let Some(dt_start) = maybe_dt_start {
            let dt_start_type =
                get_declared_value_type(&ComponentProperty::DateTimeStart(dt_start.clone()));
            let dt_end_type = get_declared_value_type(&ComponentProperty::DateTimeEnd(
                date_time_end_property.clone(),
            ));

            check_date_time_value_type_match(errors, index, dt_start_type, dt_end_type, "DTEND");
        }
    }

    if let Some(dt_start) = maybe_dt_start {
        match property_location {
            PropertyLocation::Event => {
                if dt_start.is_utc != date_time_end_property.is_utc {
                    errors.push(ComponentPropertyError {
                        message: "DTEND must have the same time type as DTSTART, both UTC or both not UTC".to_string(),
                        location: Some(ComponentPropertyLocation {
                            index,
                            name: "DTEND".to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            PropertyLocation::FreeBusy => {
                if !dt_start.is_utc || !date_time_end_property.is_utc {
                    errors.push(ComponentPropertyError {
                        message: "DTSTART and DTEND for FREEBUSY must be UTC date-times"
                            .to_string(),
                        location: Some(ComponentPropertyLocation {
                            index,
                            name: "DTEND".to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            _ => {
                // Not expected to be specified elsewhere, and not restricted on Other
            }
        }

        match date_time_end_property.date.cmp(&dt_start.date) {
            Ordering::Less => {
                errors.push(ComponentPropertyError {
                    message: "DTEND is before DTSTART".to_string(),
                    location: Some(ComponentPropertyLocation {
                        index,
                        name: "DTEND".to_string(),
                        property_location: Some(WithinPropertyLocation::Value),
                    }),
                });
            }
            Ordering::Equal => {
                // Because the types must match above, not need to check for other combinations here
                if let (Some(dt_end_time), Some(dt_start_time)) =
                    (date_time_end_property.time, dt_start.time)
                {
                    if dt_end_time < dt_start_time {
                        errors.push(ComponentPropertyError {
                            message: "DTEND is before DTSTART".to_string(),
                            location: Some(ComponentPropertyLocation {
                                index,
                                name: "DTEND".to_string(),
                                property_location: Some(WithinPropertyLocation::Value),
                            }),
                        });
                    }
                }
            }
            Ordering::Greater => {
                // Valid
            }
        }
    }
}

fn check_date_time_value_type_match(
    errors: &mut Vec<ComponentPropertyError>,
    index: usize,
    dt_start_type: Option<(Value, usize)>,
    other_type: Option<(Value, usize)>,
    other_type_name: &str,
) {
    match (dt_start_type, other_type) {
        (None, None) => {
            // Fine, both default
        }
        (Some((Value::Date, _)), Some((Value::Date, _))) => {
            // Fine, both date
        }
        (Some((Value::DateTime, _)), Some((Value::DateTime, _))) => {
            // Fine, both date-time
        }
        (Some((Value::DateTime, _)), None) | (None, Some((Value::DateTime, _))) => {
            // Fine, one specified and other at default
        }
        (Some((Value::DateTime, _)) | None, Some((Value::Date, _))) => {
            errors.push(ComponentPropertyError {
                message: format!("DTSTART is date-time but {other_type_name} is date"),
                location: Some(ComponentPropertyLocation {
                    index,
                    name: other_type_name.to_string(),
                    property_location: Some(WithinPropertyLocation::Value),
                }),
            });
        }
        (Some((Value::Date, _)), Some((Value::DateTime, _)) | None) => {
            errors.push(ComponentPropertyError {
                message: format!("DTSTART is date but {other_type_name} is date-time"),
                location: Some(ComponentPropertyLocation {
                    index,
                    name: other_type_name.to_string(),
                    property_location: Some(WithinPropertyLocation::Value),
                }),
            });
        }
        _ => {
            // This is reachable, but any such combination should be rejected later by value type checking
        }
    }
}

/// RFC 5545, 3.8.2.3
fn validate_date_time_due(
    errors: &mut Vec<ComponentPropertyError>,
    date_time_due_property: &DateTimeDueProperty,
    maybe_dt_start: Option<&DateTimeStartProperty>,
    index: usize,
    property_location: PropertyLocation,
) {
    // For a to-do, the date/date-time types must match at the start and end
    if property_location == PropertyLocation::ToDo {
        if let Some(dt_start) = maybe_dt_start {
            let dt_start_type =
                get_declared_value_type(&ComponentProperty::DateTimeStart(dt_start.clone()));
            let dt_due_type = get_declared_value_type(&ComponentProperty::DateTimeDue(
                date_time_due_property.clone(),
            ));

            check_date_time_value_type_match(errors, index, dt_start_type, dt_due_type, "DUE");
        }
    }

    if let Some(dt_start) = maybe_dt_start {
        match property_location {
            PropertyLocation::Event => {
                if dt_start.is_utc != date_time_due_property.is_utc {
                    errors.push(ComponentPropertyError {
                        message:
                            "DUE must have the same time type as DTSTART, both UTC or both not UTC"
                                .to_string(),
                        location: Some(ComponentPropertyLocation {
                            index,
                            name: "DUE".to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            _ => {
                // Not expected to be specified elsewhere, and not restricted on Other
            }
        }

        match date_time_due_property.date.cmp(&dt_start.date) {
            Ordering::Less => {
                errors.push(ComponentPropertyError {
                    message: "DUE is before DTSTART".to_string(),
                    location: Some(ComponentPropertyLocation {
                        index,
                        name: "DUE".to_string(),
                        property_location: Some(WithinPropertyLocation::Value),
                    }),
                });
            }
            Ordering::Equal => {
                // Because the types must match above, not need to check for other combinations here
                if let (Some(dt_end_time), Some(dt_start_time)) =
                    (date_time_due_property.time, dt_start.time)
                {
                    if dt_end_time < dt_start_time {
                        errors.push(ComponentPropertyError {
                            message: "DUE is before DTSTART".to_string(),
                            location: Some(ComponentPropertyLocation {
                                index,
                                name: "DUE".to_string(),
                                property_location: Some(WithinPropertyLocation::Value),
                            }),
                        });
                    }
                }
            }
            Ordering::Greater => {
                // Valid
            }
        }
    }
}

/// RFC 5545, 3.8.2.4
fn validate_date_time_start(
    errors: &mut Vec<ComponentPropertyError>,
    date_time_start_property: &DateTimeStartProperty,
    index: usize,
    property_location: PropertyLocation,
) {
    if !date_time_start_property
        .params
        .iter()
        .any(|p| matches!(p, Param::ValueType { .. }))
        && date_time_start_property.time.is_none()
    {
        errors.push(ComponentPropertyError {
            message: "DTSTART defaults to date-time but only has a date value".to_string(),
            location: Some(ComponentPropertyLocation {
                index,
                name: "DTSTART".to_string(),
                property_location: Some(WithinPropertyLocation::Value),
            }),
        });
    }

    match property_location {
        PropertyLocation::Event => {
            // Nothing further to check, valid
        }
        PropertyLocation::FreeBusy => {
            if date_time_start_property.time.is_none() || !date_time_start_property.is_utc {
                errors.push(ComponentPropertyError {
                    message: "DTSTART for FREEBUSY must be a UTC date-time".to_string(),
                    location: Some(ComponentPropertyLocation {
                        index,
                        name: "DTSTART".to_string(),
                        property_location: Some(WithinPropertyLocation::Value),
                    }),
                });
            }
        }
        PropertyLocation::TimeZoneComponent => {
            if date_time_start_property.time.is_none() || date_time_start_property.is_utc {
                errors.push(ComponentPropertyError {
                    message: "DTSTART must be a local time".to_string(),
                    location: Some(ComponentPropertyLocation {
                        index,
                        name: "DTSTART".to_string(),
                        property_location: Some(WithinPropertyLocation::Value),
                    }),
                });
            }
        }
        _ => {
            unreachable!()
        }
    }
}

fn validate_status(
    errors: &mut Vec<ComponentPropertyError>,
    status: &StatusProperty,
    index: usize,
    property_location: PropertyLocation,
) {
    match property_location {
        PropertyLocation::Event => {
            match status.value {
                Status::Tentative | Status::Confirmed | Status::Cancelled => {
                    // Valid
                }
                _ => {
                    errors.push(ComponentPropertyError {
                        message: format!("Invalid STATUS value for event: {:?}", status.value),
                        location: Some(ComponentPropertyLocation {
                            index,
                            name: "STATUS".to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
        }
        PropertyLocation::ToDo => {
            match status.value {
                Status::NeedsAction | Status::Completed | Status::InProcess | Status::Cancelled => {
                    // Valid
                }
                _ => {
                    errors.push(ComponentPropertyError {
                        message: format!("Invalid STATUS value for to-do: {:?}", status.value),
                        location: Some(ComponentPropertyLocation {
                            index,
                            name: "STATUS".to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
        }
        PropertyLocation::Journal => {
            match status.value {
                Status::Draft | Status::Final | Status::Cancelled => {
                    // Valid
                }
                _ => {
                    errors.push(ComponentPropertyError {
                        message: format!("Invalid STATUS value for journal: {:?}", status.value),
                        location: Some(ComponentPropertyLocation {
                            index,
                            name: "STATUS".to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
        }
        PropertyLocation::Other => {
            // Permit any
        }
        _ => {
            // Property occurrence checks should have prevented this being reached
            unreachable!()
        }
    }
}

/// RFC 5545, 3.8.2.6
fn validate_free_busy_time(
    errors: &mut Vec<ComponentPropertyError>,
    free_busy_time_property: &FreeBusyTimeProperty,
    index: usize,
) {
    if !free_busy_time_property.value.iter().all(|p| {
        p.start.2
            && match p.end {
                PeriodEnd::DateTime((_, _, is_utc)) => is_utc,
                PeriodEnd::Duration(_) => true,
            }
    }) {
        errors.push(ComponentPropertyError {
            message: "FREEBUSY periods must be UTC".to_string(),
            location: Some(ComponentPropertyLocation {
                index,
                name: "FREEBUSY".to_string(),
                property_location: Some(WithinPropertyLocation::Value),
            }),
        });
    }

    let date_times = free_busy_time_property
        .value
        .iter()
        .map(|p| p.expand().unwrap())
        .collect::<Vec<_>>();
    let all_ordered = date_times.windows(2).all(|w| {
        let (s1, e1) = &w[0];
        let (s2, e2) = &w[1];

        match s1.cmp(s2) {
            Ordering::Less => true,
            Ordering::Equal => {
                matches!(e1.cmp(e2), Ordering::Less)
            }
            _ => false,
        }
    });

    if !all_ordered {
        errors.push(ComponentPropertyError {
            message: "FREEBUSY periods should be ordered".to_string(),
            location: Some(ComponentPropertyLocation {
                index,
                name: "FREEBUSY".to_string(),
                property_location: Some(WithinPropertyLocation::Value),
            }),
        });
    }
}

// RFC 5545, 3.8.7.2
fn validate_date_time_stamp(
    errors: &mut Vec<ComponentPropertyError>,
    date_time_stamp_property: &DateTimeStampProperty,
    index: usize,
) {
    if !date_time_stamp_property.is_utc {
        errors.push(ComponentPropertyError {
            message: "DTSTAMP must be a UTC date-time".to_string(),
            location: Some(ComponentPropertyLocation {
                index,
                name: "DTSTAMP".to_string(),
                property_location: Some(WithinPropertyLocation::Value),
            }),
        });
    }
}

// RFC 5545, 3.8.7.3
fn validate_last_modified(
    errors: &mut Vec<ComponentPropertyError>,
    last_modified_property: &LastModifiedProperty,
    index: usize,
) {
    if !last_modified_property.is_utc {
        errors.push(ComponentPropertyError {
            message: "LAST-MODIFIED must be a UTC date-time".to_string(),
            location: Some(ComponentPropertyLocation {
                index,
                name: "LAST-MODIFIED".to_string(),
                property_location: Some(WithinPropertyLocation::Value),
            }),
        });
    }
}
