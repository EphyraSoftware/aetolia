mod error;

use crate::common::{Encoding, ParticipationStatusUnknown, Status, Value};
use crate::model::{CalendarComponent, CalendarProperty, ComponentProperty, ICalObject, Param};
use crate::serialize::WriteModel;
use crate::validate::error::{CalendarPropertyError, ICalendarError, ParamError};
use anyhow::Context;
pub use error::*;
use std::collections::HashSet;

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

    let calendar_info = CalendarInfo::new(time_zone_ids);

    errors.extend_from_slice(
        ICalendarError::many_from_calendar_property_errors(validate_calendar_properties(
            &ical_object,
            &calendar_info,
        ))
        .as_slice(),
    );

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

fn validate_component_properties(
    calendar_info: &CalendarInfo,
    property_location: PropertyLocation,
    properties: &[ComponentProperty],
) -> anyhow::Result<Vec<ComponentPropertyError>> {
    let mut errors = Vec::new();

    for (index, property) in properties.iter().enumerate() {
        check_encoding_for_binary_values(&mut errors, property, index)?;

        match property {
            ComponentProperty::Description(description) => {
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
            ComponentProperty::Attendee(attendee) => {
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
            ComponentProperty::Organizer(organizer) => {
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
            ComponentProperty::DateTimeStart(date_time_start) => {
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
            ComponentProperty::TimeZoneId(time_zone_id) => {
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
            ComponentProperty::IanaProperty(_) => {
                // Nothing to validate
            }
            _ => {
                unimplemented!()
            }
        }
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
        if value_type == Value::Binary {
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
    }

    Ok(())
}

#[derive(Debug)]
struct CalendarInfo {
    /// The ids of the time zones that this calendar defines
    time_zone_ids: HashSet<String>,
}

impl CalendarInfo {
    fn new(time_zone_ids: HashSet<String>) -> Self {
        CalendarInfo { time_zone_ids }
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
    TimeZone,
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

fn validate_calendar_properties(
    ical_object: &ICalObject,
    calendar_info: &CalendarInfo,
) -> Vec<CalendarPropertyError> {
    let mut errors = Vec::new();

    for (index, property) in ical_object.properties.iter().enumerate() {
        match property {
            CalendarProperty::Version(version) => {
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

    println!("{:?}", property_info.value_is_utc);
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
        _ => unimplemented!(),
    }
}

fn component_property_name(property: &ComponentProperty) -> &str {
    match property {
        ComponentProperty::Description(_) => "DESCRIPTION",
        ComponentProperty::Attendee(_) => "ATTENDEE",
        ComponentProperty::Organizer(_) => "ORGANIZER",
        ComponentProperty::TimeZoneId(_) => "TZID",
        ComponentProperty::DateTimeStart(_) => "DTSTART",
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
VERSION;CN=hello:2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In calendar property \"VERSION\" at index 0: Common name (CN) is not allowed for this property type", errors.first().unwrap().to_string());
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
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 0: Common name (CN) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn cu_type_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
VERSION;CUTYPE=INDIVIDUAL:2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In calendar property \"VERSION\" at index 0: Calendar user type (CUTYPE) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn cu_type_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
DESCRIPTION;CUTYPE=INDIVIDUAL:some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 0: Calendar user type (CUTYPE) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn delegated_from_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
VERSION;DELEGATED-FROM=\"mailto:hello@test.net\":2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In calendar property \"VERSION\" at index 0: Delegated from (DELEGATED-FROM) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn delegated_from_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
DESCRIPTION;DELEGATED-FROM=\"mailto:hello@test.net\":some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 0: Delegated from (DELEGATED-FROM) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn delegated_to_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
VERSION;DELEGATED-TO=\"mailto:hello@test.net\":2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In calendar property \"VERSION\" at index 0: Delegated to (DELEGATED-TO) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn delegated_to_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
DESCRIPTION;DELEGATED-TO=\"mailto:hello@test.net\":some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 0: Delegated to (DELEGATED-TO) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn dir_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
VERSION;DIR=\"ldap://example.com:6666/o=ABC\":2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In calendar property \"VERSION\" at index 0: Directory entry reference (DIR) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn dir_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
DESCRIPTION;DIR=\"ldap://example.com:6666/o=ABC\":some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 0: Directory entry reference (DIR) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn encoding_not_set_on_binary_value() {
        let content = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
DESCRIPTION;VALUE=BINARY:eA==\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 0: Property is declared to have a binary value but no encoding is set, must be set to BASE64", errors.first().unwrap().to_string());
    }

    #[test]
    fn encoding_set_to_8bit_on_binary_value() {
        let content = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
DESCRIPTION;VALUE=BINARY;ENCODING=8BIT:eA==\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 0: Property is declared to have a binary value but the encoding is set to 8BIT, instead of BASE64", errors.first().unwrap().to_string());
    }

    #[test]
    fn member_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
VERSION;MEMBER=\"mailto:hello@test.net\":2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In calendar property \"VERSION\" at index 0: Group or list membership (MEMBER) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn member_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
DESCRIPTION;MEMBER=\"mailto:hello@test.net\":some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 0: Group or list membership (MEMBER) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn part_stat_wrong_value_in_event() {
        let content = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
ATTENDEE;PARTSTAT=COMPLETED:mailto:hello@test.net\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"ATTENDEE\" at index 0: Invalid participation status (PARTSTAT) value [Completed] in a VEVENT component context", errors.first().unwrap().to_string());
    }

    #[test]
    fn part_stat_wrong_value_in_journal() {
        let content = "BEGIN:VCALENDAR\r\n\
BEGIN:VJOURNAL\r\n\
ATTENDEE;PARTSTAT=IN-PROCESS:mailto:hello@test.net\r\n\
END:VJOURNAL\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VJOURNAL\" at index 0, in component property \"ATTENDEE\" at index 0: Invalid participation status (PARTSTAT) value [InProcess] in a VJOURNAL component context", errors.first().unwrap().to_string());
    }

    #[test]
    fn part_stat_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
DESCRIPTION;PARTSTAT=NEEDS-ACTION:some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 0: Participation status (PARTSTAT) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn related_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
VERSION;RELATED=END:2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In calendar property \"VERSION\" at index 0: Related (RELATED) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn related_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
DESCRIPTION;RELATED=START:some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 0: Related (RELATED) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn role_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
VERSION;ROLE=CHAIR:2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In calendar property \"VERSION\" at index 0: Participation role (ROLE) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn role_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
DESCRIPTION;ROLE=CHAIN:some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 0: Participation role (ROLE) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn rsvp_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
VERSION;RSVP=TRUE:2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In calendar property \"VERSION\" at index 0: RSVP expectation (RSVP) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn rsvp_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
DESCRIPTION;RSVP=FALSE:some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 0: RSVP expectation (RSVP) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn sent_by_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
VERSION;SENT-BY=\"mailto:hello@test.net\":2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In calendar property \"VERSION\" at index 0: Sent by (SENT-BY) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn sent_by_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
DESCRIPTION;SENT-BY=\"mailto:hello@test.net\":some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 0: Sent by (SENT-BY) is not allowed for this property type", errors.first().unwrap().to_string());
    }

    #[test]
    fn sent_by_with_invalid_protocol() {
        let content = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
ORGANIZER;SENT-BY=\"http:hello@test.net\":mailto:world@test.net\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"ORGANIZER\" at index 0: Sent by (SENT-BY) must be a 'mailto:' URI", errors.first().unwrap().to_string());
    }

    #[test]
    fn missing_tz_id() {
        let content = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
DTSTART;TZID=missing:20240606T220000\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in component property \"DTSTART\" at index 0: Required time zone ID [missing] is not defined in the calendar", errors.first().unwrap().to_string());
    }

    #[test]
    fn tz_id_specified_on_utc_start() {
        let content = "BEGIN:VCALENDAR\r\n\
BEGIN:VTIMEZONE\r\n\
TZID:any\r\n\
END:VTIMEZONE\r\n\
BEGIN:VEVENT\r\n\
DTSTART;TZID=any:20240606T220000Z\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 1, in component property \"DTSTART\" at index 0: Time zone ID (TZID) cannot be specified on a property with a UTC time", errors.first().unwrap().to_string());
    }

    #[test]
    fn tz_id_specified_on_date_start() {
        let content = "BEGIN:VCALENDAR\r\n\
BEGIN:VTIMEZONE\r\n\
TZID:any\r\n\
END:VTIMEZONE\r\n\
BEGIN:VEVENT\r\n\
DTSTART;TZID=any:20240606\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 1, in component property \"DTSTART\" at index 0: Time zone ID (TZID) is not allowed for the property value type DATE", errors.first().unwrap().to_string());
    }

    fn validate_content(content: &str) -> Vec<ICalendarError> {
        let (rem, object) = crate::parser::ical_object::<Error>(content.as_bytes()).unwrap();
        check_rem(rem, 0);

        validate_model(object.to_model().unwrap()).unwrap()
    }
}
