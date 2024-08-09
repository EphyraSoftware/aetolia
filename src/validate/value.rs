use crate::common::{Encoding, Value};
use crate::convert::ToModel;
use crate::model::{
    AttendeeProperty, ComponentProperty, DateTimeDueProperty, DateTimeEndProperty,
    DateTimeStartProperty, EncodingParam, ExceptionDateTimesProperty, OrganizerProperty, Param,
    RecurrenceDateTimesProperty, RecurrenceDateTimesPropertyValue, RecurrenceIdProperty,
};
use crate::parser::recur::recur;
use crate::parser::uri::param_value_uri;
use crate::parser::{
    prop_value_binary, prop_value_date, prop_value_date_time, prop_value_duration,
    prop_value_float, prop_value_integer, prop_value_period, prop_value_text, prop_value_time,
    prop_value_utc_offset, Error,
};
use crate::prelude::TriggerValue;
use crate::serialize::WriteModel;
use crate::validate::recur::validate_recurrence_rule;
use crate::validate::{
    component_property_name, get_declared_value_type, validate_time, validate_utc_offset,
    ComponentPropertyError, ComponentPropertyLocation, PropertyLocation, WithinPropertyLocation,
};
use anyhow::Context;
use nom::character::streaming::char;
use nom::multi::separated_list1;
use nom::AsBytes;

pub(super) fn check_declared_value(
    errors: &mut Vec<ComponentPropertyError>,
    maybe_dt_start: Option<&DateTimeStartProperty>,
    property: &ComponentProperty,
    property_index: usize,
) -> anyhow::Result<()> {
    let declared_value_type = get_declared_value_type(property);

    let push_redundant_error_msg =
        |errors: &mut Vec<ComponentPropertyError>,
         property_index: usize,
         property: &ComponentProperty| {
            errors.push(ComponentPropertyError {
                message: "Redundant value specification which matches the default value"
                    .to_string(),
                location: Some(ComponentPropertyLocation {
                    index: property_index,
                    name: component_property_name(property).to_string(),
                    property_location: None,
                }),
            });
        };

    if let Some((value_type, value_type_index)) = declared_value_type {
        match value_type {
            Value::Binary => {
                let mut found_encoding = None;
                for param in property.params() {
                    if let Param::Encoding(EncodingParam { encoding }) = param {
                        found_encoding = Some(encoding.clone());

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

                            return Ok(());
                        }
                    }
                }

                if found_encoding.is_none() {
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
                    return Ok(());
                }

                let require_base64 = |v: &str| match found_encoding.expect("Always present") {
                    Encoding::Base64 => {
                        if !is_base64_valued(v) {
                            errors.push(ComponentPropertyError {
                                message: "Property is declared to have a binary value but the value is not base64".to_string(),
                                location: Some(ComponentPropertyLocation {
                                    index: property_index,
                                    name: component_property_name(property).to_string(),
                                    property_location: Some(WithinPropertyLocation::Value),
                                }),
                            });
                        }
                    }
                    _ => {
                        unreachable!("Encoding has already been checked to be Base64")
                    }
                };

                match property {
                    ComponentProperty::Attach(attach) => {
                        require_base64(&attach.value);
                    }
                    ComponentProperty::XProperty(x_prop) => {
                        require_base64(&x_prop.value);
                    }
                    ComponentProperty::IanaProperty(iana_prop) => {
                        require_base64(&iana_prop.value);
                    }
                    _ => {
                        errors.push(ComponentPropertyError {
                            message: "Property is declared to have a binary value but that is not valid for this property".to_string(),
                            location: Some(ComponentPropertyLocation {
                                index: property_index,
                                name: component_property_name(property).to_string(),
                                property_location: Some(WithinPropertyLocation::Value),
                            }),
                        });
                    }
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
                    ComponentProperty::Attendee(AttendeeProperty { value, .. })
                    | ComponentProperty::Organizer(OrganizerProperty { value, .. }) => {
                        push_redundant_error_msg(errors, property_index, property);

                        if !value.starts_with("mailto:") {
                            not_mailto = true;
                        }
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
                    ComponentProperty::DateTimeStart(DateTimeStartProperty {
                        value: date_time,
                        ..
                    })
                    | ComponentProperty::DateTimeEnd(DateTimeEndProperty {
                        value: date_time,
                        ..
                    })
                    | ComponentProperty::DateTimeDue(DateTimeDueProperty {
                        value: date_time,
                        ..
                    })
                    | ComponentProperty::RecurrenceId(RecurrenceIdProperty {
                        value: date_time,
                        ..
                    }) => {
                        if date_time.is_date_time() {
                            errors.push(ComponentPropertyError {
                                message: "Property is declared to have a date value but the value is a date-time".to_string(),
                                location: Some(ComponentPropertyLocation {
                                    index: property_index,
                                    name: component_property_name(property).to_string(),
                                    property_location: Some(WithinPropertyLocation::Value),
                                }),
                            });
                        }
                    }
                    ComponentProperty::ExceptionDateTimes(ExceptionDateTimesProperty {
                        value: date_times,
                        ..
                    }) => {
                        if date_times.iter().any(|dt| dt.is_date_time()) {
                            errors.push(ComponentPropertyError {
                                message: "Property is declared to have date values but one of values is a date-time".to_string(),
                                location: Some(ComponentPropertyLocation {
                                    index: property_index,
                                    name: component_property_name(property).to_string(),
                                    property_location: Some(WithinPropertyLocation::Value),
                                }),
                            });
                        }
                    }
                    ComponentProperty::RecurrenceDateTimes(RecurrenceDateTimesProperty {
                        value: RecurrenceDateTimesPropertyValue::DateTimes(date_times),
                        ..
                    }) => {
                        if date_times.iter().any(|dt| dt.is_date_time()) {
                            errors.push(ComponentPropertyError {
                                message: "Property is declared to have date values but one of values is a date-time".to_string(),
                                location: Some(ComponentPropertyLocation {
                                    index: property_index,
                                    name: component_property_name(property).to_string(),
                                    property_location: Some(WithinPropertyLocation::Value),
                                }),
                            });
                        }
                    }
                    ComponentProperty::RecurrenceDateTimes(RecurrenceDateTimesProperty {
                        value: RecurrenceDateTimesPropertyValue::Periods(periods),
                        ..
                    }) => {
                        if !periods.is_empty() {
                            errors.push(ComponentPropertyError {
                                message: "Property is declared to have a date-time value contains periods".to_string(),
                                location: Some(ComponentPropertyLocation {
                                    index: property_index,
                                    name: component_property_name(property).to_string(),
                                    property_location: Some(WithinPropertyLocation::Value),
                                }),
                            });
                        }
                    }
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
                    | ComponentProperty::DateTimeDue(_)
                    | ComponentProperty::RecurrenceId(_)
                    | ComponentProperty::ExceptionDateTimes(_)
                    | ComponentProperty::LastModified(_) => {
                        push_redundant_error_msg(errors, property_index, property);
                    }
                    ComponentProperty::RecurrenceDateTimes(RecurrenceDateTimesProperty {
                        value: RecurrenceDateTimesPropertyValue::DateTimes(_),
                        ..
                    }) => {
                        push_redundant_error_msg(errors, property_index, property);
                    }
                    ComponentProperty::RecurrenceDateTimes(RecurrenceDateTimesProperty {
                        value: RecurrenceDateTimesPropertyValue::Periods(_),
                        ..
                    }) => {
                        errors.push(ComponentPropertyError {
                            message:
                                "Property is declared to have a date-time value contains periods"
                                    .to_string(),
                            location: Some(ComponentPropertyLocation {
                                index: property_index,
                                name: component_property_name(property).to_string(),
                                property_location: Some(WithinPropertyLocation::Value),
                            }),
                        });
                    }
                    ComponentProperty::DateTimeStart(dtstart) => {
                        push_redundant_error_msg(errors, property_index, property);
                        if dtstart.value.is_date() {
                            errors.push(ComponentPropertyError {
                                message: "Property is declared to have a date-time value but the value is a date".to_string(),
                                location: Some(ComponentPropertyLocation {
                                    index: property_index,
                                    name: component_property_name(property).to_string(),
                                    property_location: Some(WithinPropertyLocation::Value),
                                }),
                            });
                        }
                    }
                    ComponentProperty::DateTimeEnd(dt_end) => {
                        push_redundant_error_msg(errors, property_index, property);
                        if dt_end.value.is_date() {
                            errors.push(ComponentPropertyError {
                                message: "Property is declared to have a date-time value but the value is a date".to_string(),
                                location: Some(ComponentPropertyLocation {
                                    index: property_index,
                                    name: component_property_name(property).to_string(),
                                    property_location: Some(WithinPropertyLocation::Value),
                                }),
                            });
                        }
                    }
                    ComponentProperty::Trigger(trigger) => {
                        match trigger.value {
                            TriggerValue::Relative(_) => {
                                // Valid
                            }
                            TriggerValue::Absolute(_) => {
                                errors.push(ComponentPropertyError {
                                    message: "Property is declared to have a date-time value but has an absolute trigger".to_string(),
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
                    ComponentProperty::Duration(_) => {
                        push_redundant_error_msg(errors, property_index, property);
                    }
                    ComponentProperty::Trigger(trigger) => {
                        push_redundant_error_msg(errors, property_index, property);
                        match trigger.value {
                            TriggerValue::Relative(_) => {
                                // Valid
                            }
                            TriggerValue::Absolute(_) => {
                                errors.push(ComponentPropertyError {
                                    message: "Property is declared to have a duration value but has an absolute trigger".to_string(),
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
                    ComponentProperty::GeographicPosition(_) => {
                        push_redundant_error_msg(errors, property_index, property);
                    }
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
                    ComponentProperty::PercentComplete(_)
                    | ComponentProperty::Priority(_)
                    | ComponentProperty::Repeat(_) => {
                        push_redundant_error_msg(errors, property_index, property);
                    }
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
                    ComponentProperty::FreeBusyTime(_) => {
                        push_redundant_error_msg(errors, property_index, property);
                    }
                    ComponentProperty::RecurrenceDateTimes(RecurrenceDateTimesProperty {
                        value: RecurrenceDateTimesPropertyValue::DateTimes(_),
                        ..
                    }) => {
                        errors.push(ComponentPropertyError {
                            message:
                                "Property is declared to have a period value contains date-times"
                                    .to_string(),
                            location: Some(ComponentPropertyLocation {
                                index: property_index,
                                name: component_property_name(property).to_string(),
                                property_location: Some(WithinPropertyLocation::Value),
                            }),
                        });
                    }
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
                    ComponentProperty::RecurrenceRule(_) => {
                        push_redundant_error_msg(errors, property_index, property);
                    }
                    ComponentProperty::XProperty(x_prop) => match is_recur_valued(&x_prop.value) {
                        Ok(rule) => match rule.to_model() {
                            Ok(rule) => {
                                validate_recurrence_rule(
                                    errors,
                                    property,
                                    &rule,
                                    maybe_dt_start,
                                    PropertyLocation::Other,
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
                    },
                    ComponentProperty::IanaProperty(iana_prop) => {
                        match is_recur_valued(&iana_prop.value) {
                            Ok(rule) => match rule.to_model() {
                                Ok(rule) => {
                                    validate_recurrence_rule(
                                        errors,
                                        property,
                                        &rule,
                                        maybe_dt_start,
                                        PropertyLocation::Other,
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
                    ComponentProperty::Categories(_)
                    | ComponentProperty::Classification(_)
                    | ComponentProperty::Comment(_)
                    | ComponentProperty::Description(_)
                    | ComponentProperty::Location(_)
                    | ComponentProperty::Resources(_)
                    | ComponentProperty::Status(_)
                    | ComponentProperty::Summary(_)
                    | ComponentProperty::TimeTransparency(_)
                    | ComponentProperty::TimeZoneId(_)
                    | ComponentProperty::TimeZoneName(_)
                    | ComponentProperty::Contact(_)
                    | ComponentProperty::UniqueIdentifier(_)
                    | ComponentProperty::Action(_)
                    | ComponentProperty::RequestStatus(_) => {
                        push_redundant_error_msg(errors, property_index, property);
                    }
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
                let require_uri = |errors: &mut Vec<ComponentPropertyError>, v: &str| {
                    if !is_uri_valued(v) {
                        errors.push(ComponentPropertyError {
                            message: "Property is declared to have a URI value but the value is not a URI".to_string(),
                            location: Some(ComponentPropertyLocation {
                                index: property_index,
                                name: component_property_name(property).to_string(),
                                property_location: Some(WithinPropertyLocation::Value),
                            }),
                        });
                    }
                };

                match property {
                    ComponentProperty::Url(url) => {
                        push_redundant_error_msg(errors, property_index, property);
                        require_uri(errors, &url.value);
                    }
                    ComponentProperty::Attach(attach) => {
                        push_redundant_error_msg(errors, property_index, property);
                        require_uri(errors, &attach.value);
                    }
                    ComponentProperty::XProperty(x_prop) => {
                        require_uri(errors, &x_prop.value);
                    }
                    ComponentProperty::IanaProperty(iana_prop) => {
                        require_uri(errors, &iana_prop.value);
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
            }
            Value::UtcOffset => {
                let mut invalid = false;

                match property {
                    ComponentProperty::TimeZoneOffsetFrom(_)
                    | ComponentProperty::TimeZoneOffsetTo(_) => {
                        push_redundant_error_msg(errors, property_index, property);
                    }
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

fn is_base64_valued(property_value: &str) -> bool {
    let mut content = property_value.as_bytes().to_vec();
    content.push(b';');

    let result = prop_value_binary::<Error>(content.as_bytes());
    match result {
        Ok((rest, _)) => rest.len() == 1,
        _ => false,
    }
}

fn is_boolean_valued(property_value: &str) -> bool {
    property_value.eq_ignore_ascii_case("TRUE") || property_value.eq_ignore_ascii_case("FALSE")
}

fn is_date_valued(property_value: &String) -> bool {
    let mut content = property_value.as_bytes().to_vec();
    content.push(b';');

    let result = separated_list1(char(','), prop_value_date::<Error>)(content.as_bytes());
    match result {
        Ok((rest, _)) => rest.len() == 1,
        _ => false,
    }
}

fn is_date_time_valued(property_value: &String) -> bool {
    let mut content = property_value.as_bytes().to_vec();
    content.push(b';');

    let result = separated_list1(char(','), prop_value_date_time::<Error>)(content.as_bytes());
    match result {
        Ok((rest, _)) => rest.len() == 1,
        _ => false,
    }
}

fn is_duration_valued(property_value: &String) -> bool {
    let mut content = property_value.as_bytes().to_vec();
    content.push(b';');

    let result = separated_list1(char(','), prop_value_duration::<Error>)(content.as_bytes());
    match result {
        Ok((rest, _)) => rest.len() == 1,
        _ => false,
    }
}

fn is_float_valued(property_value: &String) -> bool {
    let mut content = property_value.as_bytes().to_vec();
    content.push(b';');

    let result = separated_list1(char(','), prop_value_float::<Error>)(content.as_bytes());
    match result {
        Ok((rest, _)) => rest.len() == 1,
        _ => false,
    }
}

fn is_integer_valued(property_value: &String) -> bool {
    let mut content = property_value.as_bytes().to_vec();
    content.push(b';');

    let result = separated_list1(char(','), prop_value_integer::<Error>)(content.as_bytes());
    match result {
        Ok((rest, _)) => rest.len() == 1,
        _ => false,
    }
}

fn is_period_valued(property_value: &String) -> bool {
    let mut content = property_value.as_bytes().to_vec();
    content.push(b';');

    let result = prop_value_period::<Error>(content.as_bytes());
    match result {
        Ok((rest, _)) => rest.len() == 1,
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

fn is_uri_valued(property_value: &str) -> bool {
    let mut content = property_value.as_bytes().to_vec();
    content.push(b'\n');

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
