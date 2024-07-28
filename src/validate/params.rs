use crate::common::ParticipationStatusUnknown;
use crate::model::Param;
use crate::validate::error::ParamError;
use crate::validate::{
    param_name, OccurrenceExpectation, PropertyInfo, PropertyKind, PropertyLocation, ValueType,
};
use std::collections::HashMap;

macro_rules! check_property_param_occurrence {
    ($errors:ident, $seen:ident, $param:ident, $index:ident, $occur:expr) => {
        let name = $crate::validate::param_name($param);
        let count = $crate::validate::add_to_seen($seen, name);
        if let Some(message) = $crate::validate::check_occurrence(&$seen, name, $occur.clone()) {
            $errors.push($crate::validate::ParamError {
                index: $index,
                name: name.to_string(),
                message,
            });
        }
    };
}

pub(super) fn validate_params(params: &[Param], property_info: PropertyInfo) -> Vec<ParamError> {
    let mut errors = Vec::new();

    let mut seen = HashMap::<String, u32>::new();
    for (index, param) in params.iter().enumerate() {
        match param {
            Param::CommonName { name } => {
                validate_common_name_param(&mut errors, &mut seen, param, index, &property_info);
            }
            Param::Other { name, .. } | Param::Others { name, .. } if name == "CN" => {
                validate_common_name_param(&mut errors, &mut seen, param, index, &property_info);
            }
            Param::CalendarUserType { .. } => {
                validate_calendar_user_type_param(
                    &mut errors,
                    &mut seen,
                    param,
                    index,
                    &property_info,
                );
            }
            Param::Other { name, .. } | Param::Others { name, .. } if name == "CUTYPE" => {
                validate_calendar_user_type_param(
                    &mut errors,
                    &mut seen,
                    param,
                    index,
                    &property_info,
                );
            }
            Param::DelegatedFrom { .. } => {
                validate_delegated_from_param(&mut errors, &mut seen, param, index, &property_info);
            }
            Param::Other { name, .. } | Param::Others { name, .. } if name == "DELEGATED-FROM" => {
                validate_delegated_from_param(&mut errors, &mut seen, param, index, &property_info);
            }
            Param::DelegatedTo { .. } => {
                validate_delegated_to_param(&mut errors, &mut seen, param, index, &property_info);
            }
            Param::Other { name, .. } | Param::Others { name, .. } if name == "DELEGATED-TO" => {
                validate_delegated_to_param(&mut errors, &mut seen, param, index, &property_info);
            }
            Param::DirectoryEntryReference { .. } => {
                validate_dir_param(&mut errors, &mut seen, param, index, &property_info);
            }
            Param::Other { name, .. } | Param::Others { name, .. } if name == "DIR" => {
                validate_dir_param(&mut errors, &mut seen, param, index, &property_info);
            }
            Param::ValueType { .. } => {
                validate_value_type_param(&mut errors, &mut seen, param, index, &property_info);
            }
            Param::Other { name, .. } if name == "VALUE" => {
                validate_value_type_param(&mut errors, &mut seen, param, index, &property_info);
            }
            Param::Encoding { .. } => {
                // Nothing further to validate
            }
            Param::FormatType { .. } => {
                validate_fmt_type_param(&mut errors, &mut seen, param, index, &property_info);
                // Format type is not further validated by this program
            }
            Param::Other { name, .. } if name == "FMTTYPE" => {
                validate_fmt_type_param(&mut errors, &mut seen, param, index, &property_info);
            }
            Param::Others { name, .. } if name == "FMTTYPE" => {
                errors.push(ParamError {
                    index,
                    name: param_name(param).to_string(),
                    message: "FMTTYPE may not have multiple values".to_string(),
                });
            }
            Param::FreeBusyTimeType { .. } => {
                validate_free_busy_time_type_param(
                    &mut errors,
                    &mut seen,
                    param,
                    index,
                    &property_info,
                );
            }
            Param::Language { .. } => {
                validate_language_param(&mut errors, &mut seen, param, index, &property_info);
                // Language is not further validated by this program
            }
            Param::Other { name, .. } if name == "LANGUAGE" => {
                validate_language_param(&mut errors, &mut seen, param, index, &property_info);
            }
            Param::Members { .. } => {
                validate_member_param(&mut errors, &mut seen, param, index, &property_info);
            }
            Param::Other { name, .. } | Param::Others { name, .. } if name == "MEMBER" => {
                validate_member_param(&mut errors, &mut seen, param, index, &property_info);
            }
            Param::ParticipationStatus { status } => {
                validate_part_stat_param(
                    &mut errors,
                    &mut seen,
                    param,
                    status,
                    index,
                    &property_info,
                );
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
                validate_role_param(&mut errors, &mut seen, param, index, &property_info);
            }
            Param::Other { name, .. } if name == "ROLE" => {
                validate_role_param(&mut errors, &mut seen, param, index, &property_info);
            }
            Param::Rsvp { .. } => {
                validate_rsvp_param(&mut errors, &mut seen, param, index, &property_info);
            }
            Param::Other { name, .. } if name == "RSVP" => {
                validate_rsvp_param(&mut errors, &mut seen, param, index, &property_info);
            }
            Param::SentBy { address } => {
                validate_sent_by_param(
                    &mut errors,
                    &mut seen,
                    param,
                    address,
                    index,
                    &property_info,
                );
            }
            Param::Other { name, value } if name == "SENT-BY" => {
                validate_sent_by_param(&mut errors, &mut seen, param, value, index, &property_info);
            }
            Param::TimeZoneId { tz_id, unique } => {
                validate_time_zone_id_param(
                    &mut errors,
                    &mut seen,
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
                    &mut seen,
                    param,
                    value,
                    false,
                    index,
                    &property_info,
                );
            }
            Param::AltRep { .. } => {}
            _ => {
                unimplemented!()
            }
        }
    }

    errors
}

// RFC 5545, Section 3.2.1
fn validate_alt_rep_param(
    errors: &mut Vec<ParamError>,
    seen: &mut HashMap<String, u32>,
    param: &Param,
    index: usize,
    property_info: &PropertyInfo,
) {
    if !property_info.is_other && property_info.value_type != ValueType::Text {
        errors.push(ParamError {
            index,
            name: param_name(param).to_string(),
            message: "Alternate text representation (ALTREP) is not allowed for this property type"
                .to_string(),
        });
        return;
    }

    let occurrence_expectation = match property_info.property_kind {
        PropertyKind::Comment
        | PropertyKind::Description
        | PropertyKind::Location
        | PropertyKind::Resources
        | PropertyKind::Summary
        | PropertyKind::Contact => OccurrenceExpectation::OptionalOnce,
        PropertyKind::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };
    check_property_param_occurrence!(errors, seen, param, index, occurrence_expectation);
}

// RFC 5545, Section 3.2.2
fn validate_common_name_param(
    errors: &mut Vec<ParamError>,
    seen: &mut HashMap<String, u32>,
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
        return;
    }

    let occurrence_expectation = match property_info.property_kind {
        PropertyKind::Attendee => attendee_common_expectation(property_info),
        PropertyKind::Organizer => OccurrenceExpectation::OptionalOnce,
        PropertyKind::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };
    check_property_param_occurrence!(errors, seen, param, index, occurrence_expectation);
}

// RFC 5545, Section 3.2.3
fn validate_calendar_user_type_param(
    errors: &mut Vec<ParamError>,
    seen: &mut HashMap<String, u32>,
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
        return;
    }

    let occurrence_expectation = match property_info.property_kind {
        PropertyKind::Attendee => attendee_common_expectation(property_info),
        PropertyKind::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };
    check_property_param_occurrence!(errors, seen, param, index, occurrence_expectation);
}

// RFC 5545, Section 3.2.4
fn validate_delegated_from_param(
    errors: &mut Vec<ParamError>,
    seen: &mut HashMap<String, u32>,
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
        return;
    }

    let occurrence_expectation = match property_info.property_kind {
        PropertyKind::Attendee => attendee_common_expectation(property_info),
        PropertyKind::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };
    check_property_param_occurrence!(errors, seen, param, index, occurrence_expectation);
}

// RFC 5545, Section 3.2.5
fn validate_delegated_to_param(
    errors: &mut Vec<ParamError>,
    seen: &mut HashMap<String, u32>,
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
        return;
    }

    let occurrence_expectation = match property_info.property_kind {
        PropertyKind::Attendee => attendee_common_expectation(property_info),
        PropertyKind::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };
    check_property_param_occurrence!(errors, seen, param, index, occurrence_expectation);
}

// RFC 5545, Section 3.2.6
fn validate_dir_param(
    errors: &mut Vec<ParamError>,
    seen: &mut HashMap<String, u32>,
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
        return;
    }

    let occurrence_expectation = match property_info.property_kind {
        PropertyKind::Attendee => attendee_common_expectation(property_info),
        PropertyKind::Organizer => OccurrenceExpectation::OptionalOnce,
        PropertyKind::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };
    check_property_param_occurrence!(errors, seen, param, index, occurrence_expectation);
}

// RFC 5545, Section 3.2.8
fn validate_fmt_type_param(
    errors: &mut Vec<ParamError>,
    seen: &mut HashMap<String, u32>,
    param: &Param,
    index: usize,
    property_info: &PropertyInfo,
) {
    let occurrence_expectation = match property_info.property_kind {
        PropertyKind::Attach => OccurrenceExpectation::OptionalOnce,
        PropertyKind::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };
    check_property_param_occurrence!(errors, seen, param, index, occurrence_expectation);
}

// RFC 5545, Section 3.2.9
fn validate_free_busy_time_type_param(
    errors: &mut Vec<ParamError>,
    seen: &mut HashMap<String, u32>,
    param: &Param,
    index: usize,
    property_info: &PropertyInfo,
) {
    let occurrence_expectation = match property_info.property_kind {
        PropertyKind::FreeBusyTime => OccurrenceExpectation::OptionalOnce,
        PropertyKind::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };
    check_property_param_occurrence!(errors, seen, param, index, occurrence_expectation);
}

// RFC 5545, Section 3.2.10
fn validate_language_param(
    errors: &mut Vec<ParamError>,
    seen: &mut HashMap<String, u32>,
    param: &Param,
    index: usize,
    property_info: &PropertyInfo,
) {
    let occurrence_expectation = match property_info.property_kind {
        PropertyKind::Categories
        | PropertyKind::Comment
        | PropertyKind::Description
        | PropertyKind::Location
        | PropertyKind::Resources
        | PropertyKind::Summary
        | PropertyKind::TimeZoneName
        | PropertyKind::Attendee
        | PropertyKind::Contact
        | PropertyKind::Organizer => OccurrenceExpectation::OptionalOnce,
        PropertyKind::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };
    check_property_param_occurrence!(errors, seen, param, index, occurrence_expectation);
}

// RFC 5545, Section 3.2.11
fn validate_member_param(
    errors: &mut Vec<ParamError>,
    seen: &mut HashMap<String, u32>,
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
        return;
    }

    let occurrence_expectation = match property_info.property_kind {
        PropertyKind::Attendee => attendee_common_expectation(property_info),
        PropertyKind::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };
    check_property_param_occurrence!(errors, seen, param, index, occurrence_expectation);
}

// RFC 5545, Section 3.2.12
fn validate_part_stat_param(
    errors: &mut Vec<ParamError>,
    seen: &mut HashMap<String, u32>,
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
        return;
    }

    let occurrence_expectation = match property_info.property_kind {
        PropertyKind::Attendee => attendee_common_expectation(property_info),
        PropertyKind::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };
    check_property_param_occurrence!(errors, seen, param, index, occurrence_expectation);
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
    seen: &mut HashMap<String, u32>,
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
        return;
    }

    let occurrence_expectation = match property_info.property_kind {
        PropertyKind::Attendee => attendee_common_expectation(property_info),
        PropertyKind::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };
    check_property_param_occurrence!(errors, seen, param, index, occurrence_expectation);
}

// RFC 5545, Section 3.2.17
fn validate_rsvp_param(
    errors: &mut Vec<ParamError>,
    seen: &mut HashMap<String, u32>,
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
        return;
    }

    let occurrence_expectation = match property_info.property_kind {
        PropertyKind::Attendee => attendee_common_expectation(property_info),
        PropertyKind::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };
    check_property_param_occurrence!(errors, seen, param, index, occurrence_expectation);
}

// RFC 5545, Section 3.2.18
fn validate_sent_by_param(
    errors: &mut Vec<ParamError>,
    seen: &mut HashMap<String, u32>,
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
        return;
    }

    if !address.starts_with("mailto:") {
        errors.push(ParamError {
            index,
            name: param_name(param).to_string(),
            message: "Sent by (SENT-BY) must be a 'mailto:' URI".to_string(),
        });
    }

    let occurrence_expectation = match property_info.property_kind {
        PropertyKind::Attendee => attendee_common_expectation(property_info),
        PropertyKind::Organizer => OccurrenceExpectation::OptionalOnce,
        PropertyKind::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };
    check_property_param_occurrence!(errors, seen, param, index, occurrence_expectation);
}

// RFC 5545, Section 3.2.19
fn validate_time_zone_id_param(
    errors: &mut Vec<ParamError>,
    seen: &mut HashMap<String, u32>,
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

    let occurrence_expectation = match property_info.property_kind {
        PropertyKind::DateTimeStart => match property_info.property_location {
            PropertyLocation::TimeZoneComponent => OccurrenceExpectation::Never,
            _ => OccurrenceExpectation::OptionalOnce,
        },
        PropertyKind::DateTimeEnd | PropertyKind::DateTimeDue => {
            OccurrenceExpectation::OptionalOnce
        }
        PropertyKind::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };
    check_property_param_occurrence!(errors, seen, param, index, occurrence_expectation);
}

// RFC 5545, Section 3.2.20
fn validate_value_type_param(
    errors: &mut Vec<ParamError>,
    seen: &mut HashMap<String, u32>,
    param: &Param,
    index: usize,
    property_info: &PropertyInfo,
) {
    let occurrence_expectation = match property_info.property_kind {
        PropertyKind::Attach
        | PropertyKind::DateTimeStart
        | PropertyKind::DateTimeEnd
        | PropertyKind::DateTimeDue => OccurrenceExpectation::OptionalOnce,
        PropertyKind::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    };
    check_property_param_occurrence!(errors, seen, param, index, occurrence_expectation);
}

fn attendee_common_expectation(property_info: &PropertyInfo) -> OccurrenceExpectation {
    match property_info.property_location {
        PropertyLocation::Event | PropertyLocation::ToDo | PropertyLocation::Journal => {
            OccurrenceExpectation::OptionalOnce
        }
        PropertyLocation::FreeBusy | PropertyLocation::Alarm => OccurrenceExpectation::Never,
        PropertyLocation::Other => OccurrenceExpectation::OptionalMany,
        _ => OccurrenceExpectation::Never,
    }
}
