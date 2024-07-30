mod calendar_properties;
mod component_properties;
mod error;
mod params;
mod recur;
mod value;

use crate::common::Value;
use crate::convert::ToModel;
use crate::model::{CalendarComponent, CalendarProperty, ComponentProperty, ICalObject, Param};
use crate::parser::Error;
use crate::serialize::WriteModel;
use crate::validate::calendar_properties::validate_calendar_properties;
use crate::validate::component_properties::validate_component_properties;
use crate::validate::error::ICalendarError;
use crate::validate::params::validate_params;
use anyhow::Context;
pub use error::*;
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

                for (alarm_index, alarm) in event.alarms.iter().enumerate() {
                    errors.extend_from_slice(
                        ICalendarError::many_from_nested_component_property_errors(
                            validate_component_properties(
                                &calendar_info,
                                PropertyLocation::Alarm,
                                alarm.properties(),
                            )?,
                            index,
                            component_name(component).to_string(),
                            alarm_index,
                            component_name(alarm).to_string(),
                        )
                        .as_slice(),
                    );
                }
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

                for (alarm_index, alarm) in to_do.alarms.iter().enumerate() {
                    errors.extend_from_slice(
                        ICalendarError::many_from_nested_component_property_errors(
                            validate_component_properties(
                                &calendar_info,
                                PropertyLocation::Alarm,
                                alarm.properties(),
                            )?,
                            index,
                            component_name(component).to_string(),
                            alarm_index,
                            component_name(alarm).to_string(),
                        )
                        .as_slice(),
                    );
                }
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

                if time_zone.components.is_empty() {
                    errors.push(ICalendarError {
                        message: "No standard or daylight components found in time zone, required at least one"
                            .to_string(),
                        location: Some(ICalendarLocation::Component(ComponentLocation {
                            index,
                            name: component_name(component).to_string(),
                            location: None,
                        })),
                    });
                }

                for (tz_component_index, tz_component) in time_zone.components.iter().enumerate() {
                    match tz_component {
                        CalendarComponent::Standard(standard) => {
                            errors.extend_from_slice(
                                ICalendarError::many_from_nested_component_property_errors(
                                    validate_component_properties(
                                        &calendar_info,
                                        PropertyLocation::TimeZoneComponent,
                                        &standard.properties,
                                    )?,
                                    index,
                                    component_name(component).to_string(),
                                    tz_component_index,
                                    component_name(tz_component).to_string(),
                                )
                                .as_slice(),
                            );
                        }
                        CalendarComponent::Daylight(daylight) => {
                            errors.extend_from_slice(
                                ICalendarError::many_from_nested_component_property_errors(
                                    validate_component_properties(
                                        &calendar_info,
                                        PropertyLocation::TimeZoneComponent,
                                        &daylight.properties,
                                    )?,
                                    index,
                                    component_name(component).to_string(),
                                    tz_component_index,
                                    component_name(tz_component).to_string(),
                                )
                                .as_slice(),
                            );
                        }
                        _ => {
                            errors.push(ICalendarError {
                                message: "Component is not allowed in time zone".to_string(),
                                location: Some(ICalendarLocation::Component(ComponentLocation {
                                    index,
                                    name: component_name(component).to_string(),
                                    location: Some(Box::new(WithinComponentLocation::Component(
                                        ComponentLocation {
                                            index: tz_component_index,
                                            name: component_name(tz_component).to_string(),
                                            location: None,
                                        },
                                    ))),
                                })),
                            });
                        }
                    }
                }
            }
            CalendarComponent::IanaComponent(iana_component) => {
                errors.extend_from_slice(
                    ICalendarError::many_from_component_property_errors(
                        validate_component_properties(
                            &calendar_info,
                            PropertyLocation::Other,
                            &iana_component.properties,
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
            CalendarComponent::Alarm(_)
            | CalendarComponent::Standard(_)
            | CalendarComponent::Daylight(_) => {
                errors.push(ICalendarError {
                    message: "Component is not allowed at the top level".to_string(),
                    location: Some(ICalendarLocation::Component(ComponentLocation {
                        index,
                        name: component_name(component).to_string(),
                        location: None,
                    })),
                });
            }
        }
    }

    Ok(errors)
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
    /// The property kind that is the context for validating a property value or param
    property_kind: PropertyKind,
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

#[derive(Debug)]
enum PropertyKind {
    Attach,
    Version,
    Other,
    DateTimeStart,
    Description,
    Organizer,
    TimeZoneId,
    Attendee,
    Categories,
    Comment,
    GeographicPosition,
    Location,
    PercentComplete,
    Priority,
    Resources,
    Status,
    Summary,
    DateTimeCompleted,
    DateTimeEnd,
    DateTimeDue,
    Duration,
    FreeBusyTime,
    TimeTransparency,
    TimeZoneName,
    TimeZoneOffsetTo,
    TimeZoneOffsetFrom,
    TimeZoneUrl,
    Contact,
    RecurrenceId,
    Related,
    ExceptionDateTimes,
    RecurrenceDateTimes,
    RecurrenceRule,
    Action,
    Repeat,
    Trigger,
    DateTimeCreated,
    DateTimeStamp,
    LastModified,
    Sequence,
    RequestStatus,
}

#[derive(Debug, Clone, PartialEq)]
enum PropertyLocation {
    Calendar,
    Event,
    ToDo,
    Journal,
    FreeBusy,
    TimeZone,
    TimeZoneComponent,
    Other,
    Alarm,
}

impl<'a> PropertyInfo<'a> {
    fn new(
        calendar_info: &'a CalendarInfo,
        property_location: PropertyLocation,
        property_kind: PropertyKind,
        value_type: ValueType,
    ) -> Self {
        PropertyInfo {
            property_location,
            property_kind,
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
    Binary,
    Float,
    Integer,
    Period,
    UtcOffset,
    Uri,
    Recurrence,
}

fn add_to_seen(seen: &mut HashMap<String, u32>, key: &str) -> u32 {
    *seen
        .entry(key.to_string())
        .and_modify(|count| *count += 1)
        .or_insert(1)
}

#[derive(Debug, Clone, PartialEq)]
enum OccurrenceExpectation {
    Once,
    OnceOrMany,
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
        (None | Some(0), OccurrenceExpectation::OnceOrMany) => Some(format!("{} is required", key)),
        (_, OccurrenceExpectation::OnceOrMany) => None,
        (None | Some(0) | Some(1), OccurrenceExpectation::OptionalOnce) => None,
        (_, OccurrenceExpectation::OptionalOnce) => Some(format!("{} must only appear once", key)),
        (_, OccurrenceExpectation::OptionalMany) => None,
        (None | Some(0), OccurrenceExpectation::Never) => None,
        (_, OccurrenceExpectation::Never) => Some(format!("{} is not allowed", key)),
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
        ComponentProperty::Attach(_) => "ATTACH",
        ComponentProperty::Categories(_) => "CATEGORIES",
        ComponentProperty::Classification(_) => "CLASS",
        ComponentProperty::Comment(_) => "COMMENT",
        ComponentProperty::Description(_) => "DESCRIPTION",
        ComponentProperty::GeographicPosition(_) => "GEO",
        ComponentProperty::Location(_) => "LOCATION",
        ComponentProperty::PercentComplete(_) => "PERCENT-COMPLETE",
        ComponentProperty::Priority(_) => "PRIORITY",
        ComponentProperty::Resources(_) => "RESOURCES",
        ComponentProperty::Status(_) => "STATUS",
        ComponentProperty::Summary(_) => "SUMMARY",
        ComponentProperty::DateTimeCompleted(_) => "COMPLETED",
        ComponentProperty::DateTimeEnd(_) => "DTEND",
        ComponentProperty::DateTimeDue(_) => "DUE",
        ComponentProperty::DateTimeStart(_) => "DTSTART",
        ComponentProperty::Duration(_) => "DURATION",
        ComponentProperty::FreeBusyTime(_) => "FREEBUSY",
        ComponentProperty::TimeTransparency(_) => "TRANSP",
        ComponentProperty::TimeZoneId(_) => "TZID",
        ComponentProperty::TimeZoneName(_) => "TZNAME",
        ComponentProperty::TimeZoneOffsetFrom(_) => "TZOFFSETFROM",
        ComponentProperty::TimeZoneOffsetTo(_) => "TZOFFSETTO",
        ComponentProperty::TimeZoneUrl(_) => "TZURL",
        ComponentProperty::Attendee(_) => "ATTENDEE",
        ComponentProperty::Contact(_) => "CONTACT",
        ComponentProperty::Organizer(_) => "ORGANIZER",
        ComponentProperty::RecurrenceId(_) => "RECURRENCE-ID",
        ComponentProperty::RelatedTo(_) => "RELATED-TO",
        ComponentProperty::Url(_) => "URL",
        ComponentProperty::UniqueIdentifier(_) => "UID",
        ComponentProperty::ExceptionDateTimes(_) => "EXDATE",
        ComponentProperty::RecurrenceDateTimes(_) => "RDATE",
        ComponentProperty::RecurrenceRule(_) => "RRULE",
        ComponentProperty::Action(_) => "ACTION",
        ComponentProperty::Repeat(_) => "REPEAT",
        ComponentProperty::Trigger(_) => "TRIGGER",
        ComponentProperty::DateTimeCreated(_) => "CREATED",
        ComponentProperty::DateTimeStamp(_) => "DTSTAMP",
        ComponentProperty::LastModified(_) => "LAST-MODIFIED",
        ComponentProperty::Sequence(_) => "SEQUENCE",
        ComponentProperty::IanaProperty(iana_prop) => &iana_prop.name,
        ComponentProperty::XProperty(x_prop) => &x_prop.name,
        ComponentProperty::RequestStatus(_) => "REQUEST-STATUS",
    }
}

fn component_name(component: &CalendarComponent) -> &str {
    match component {
        CalendarComponent::Event(_) => "VEVENT",
        CalendarComponent::ToDo(_) => "VTODO",
        CalendarComponent::Journal(_) => "VJOURNAL",
        CalendarComponent::TimeZone(_) => "VTIMEZONE",
        CalendarComponent::FreeBusy(_) => "VFREEBUSY",
        CalendarComponent::Alarm(_) => "VALARM",
        CalendarComponent::Standard(_) => "STANDARD",
        CalendarComponent::Daylight(_) => "DAYLIGHT",
        CalendarComponent::IanaComponent(component) => &component.name,
        CalendarComponent::XComponent(component) => &component.name,
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

    macro_rules! assert_no_errors {
        ($errors:expr) => {
            if !$errors.is_empty() {
                panic!(
                    "Expected no errors, but got: {:?}",
                    $errors
                        .iter()
                        .map(|e| e.to_string())
                        .collect::<std::vec::Vec<_>>()
                );
            }
        };
    }

    macro_rules! assert_single_error {
        ($errors:expr, $msg:expr) => {
            if $errors.is_empty() {
                panic!("Expected a single error, but validation passed");
            }

            if $errors.len() != 1 {
                panic!(
                    "Expected a single error, but got: {:?}",
                    $errors
                        .iter()
                        .map(|e| e.to_string())
                        .collect::<std::vec::Vec<_>>()
                );
            }

            assert_eq!($msg, $errors.first().unwrap().to_string());
        };
    }

    #[test]
    fn sample_passes_validation() {
        let content = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//hacksw/handcal//NONSGML v1.0//EN\r\n\
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

        assert_no_errors!(&errors);
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

        assert_single_error!(errors, "In calendar property \"VERSION\" at index 1: Common name (CN) is not allowed for this property type");
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

        assert_single_error!(errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Common name (CN) is not allowed for this property type");
    }

    #[test]
    fn calendar_user_type_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION;CUTYPE=INDIVIDUAL:2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_single_error!(errors, "In calendar property \"VERSION\" at index 1: Calendar user type (CUTYPE) is not allowed for this property type");
    }

    #[test]
    fn calendar_user_type_on_description_property() {
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

        assert_single_error!(errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Calendar user type (CUTYPE) is not allowed for this property type");
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

        assert_single_error!(errors, "In calendar property \"VERSION\" at index 1: Delegated from (DELEGATED-FROM) is not allowed for this property type");
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

        assert_single_error!(errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Delegated from (DELEGATED-FROM) is not allowed for this property type");
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

        assert_single_error!(errors, "In calendar property \"VERSION\" at index 1: Delegated to (DELEGATED-TO) is not allowed for this property type");
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

        assert_single_error!(errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Delegated to (DELEGATED-TO) is not allowed for this property type");
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

        assert_single_error!(errors, "In calendar property \"VERSION\" at index 1: Directory entry reference (DIR) is not allowed for this property type");
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

        assert_single_error!(errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Directory entry reference (DIR) is not allowed for this property type");
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
ATTACH;VALUE=BINARY:eA==\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_single_error!(errors, "In component \"VEVENT\" at index 0, in component property \"ATTACH\" at index 2: Property is declared to have a binary value but no encoding is set, must be set to BASE64");
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
ATTACH;VALUE=BINARY;ENCODING=8BIT:eA==\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_single_error!(errors, "In component \"VEVENT\" at index 0, in component property \"ATTACH\" at index 2: Property is declared to have a binary value but the encoding is set to 8BIT, instead of BASE64");
    }

    #[test]
    fn fmt_type_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION;FMTTYPE=text/plain:2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_single_error!(
            errors,
            "In calendar property \"VERSION\" at index 1: FMTTYPE is not allowed"
        );
    }

    #[test]
    fn fmt_type_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DESCRIPTION;FMTTYPE=text/plain:some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_single_error!(errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: FMTTYPE is not allowed");
    }

    #[test]
    fn free_busy_type_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION;FBTYPE=BUSY:2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_single_error!(
            errors,
            "In calendar property \"VERSION\" at index 1: FBTYPE is not allowed"
        );
    }

    #[test]
    fn free_busy_type_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DESCRIPTION;FBTYPE=BUSY:some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_single_error!(errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: FBTYPE is not allowed");
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

        assert_single_error!(&errors,"In component \"VJOURNAL\" at index 0, in component property \"ATTENDEE\" at index 2: Invalid participation status (PARTSTAT) value [InProcess] in a VJOURNAL component context");
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

        assert_single_error!(&errors, "In calendar property \"VERSION\" at index 1: RSVP expectation (RSVP) is not allowed for this property type");
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

        assert_single_error!(&errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Sent by (SENT-BY) is not allowed for this property type");
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

        assert_single_error!(&errors, "In component \"VEVENT\" at index 0, in component property \"ORGANIZER\" at index 2: Sent by (SENT-BY) must be a 'mailto:' URI");
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

        assert_single_error!(&errors, "In component \"VEVENT\" at index 0, in component property \"DTSTART\" at index 2: Required time zone ID [missing] is not defined in the calendar");
    }

    #[test]
    fn tz_id_specified_on_utc_start() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VTIMEZONE\r\n\
TZID:any\r\n\
BEGIN:STANDARD\r\n\
DTSTART:19671029T020000\r\n\
TZOFFSETFROM:-0400\r\n\
TZOFFSETTO:-0500\r\n\
END:STANDARD\r\n\
END:VTIMEZONE\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DTSTART;TZID=any:20240606T220000Z\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_single_error!(&errors, "In component \"VEVENT\" at index 1, in component property \"DTSTART\" at index 2: Time zone ID (TZID) cannot be specified on a property with a UTC time");
    }

    #[test]
    fn tz_id_specified_on_date_start() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
BEGIN:VTIMEZONE\r\n\
TZID:any\r\n\
BEGIN:STANDARD\r\n\
DTSTART:19671029T020000\r\n\
TZOFFSETFROM:-0400\r\n\
TZOFFSETTO:-0500\r\n\
END:STANDARD\r\n\
END:VTIMEZONE\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DTSTART;VALUE=DATE;TZID=any:20240606\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_single_error!(&errors, "In component \"VEVENT\" at index 1, in component property \"DTSTART\" at index 2: Time zone ID (TZID) is not allowed for the property value type DATE");
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

    #[test]
    fn alarm_with_no_action() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
BEGIN:VALARM\r\n\
TRIGGER;VALUE=DURATION:P3W\r\n\
END:VALARM\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_eq!(errors.len(), 1);
        assert_eq!("In component \"VEVENT\" at index 0, in nested component \"VALARM\" at index 0: Required exactly one ACTION property but found 0", errors.first().unwrap().to_string());
    }

    #[test]
    fn audio_alarm_with_duplicate_attach() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
BEGIN:VALARM\r\n\
ACTION:AUDIO\r\n\
TRIGGER;VALUE=DURATION:P3W\r\n\
ATTACH:ftp://example.com/pub/sounds/bell-01.aud\r\n\
ATTACH:ftp://example.com/pub/sounds/bell-01.aud\r\n\
END:VALARM\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_single_error!(&errors, "In component \"VEVENT\" at index 0, in nested component \"VALARM\" at index 0, in nested component property \"ATTACH\" at index 3: ATTACH must only appear once");
    }

    fn validate_content(content: &str) -> Vec<ICalendarError> {
        let (rem, object) = crate::parser::ical_object::<Error>(content.as_bytes()).unwrap();
        check_rem(rem, 0);

        validate_model(object.to_model().unwrap()).unwrap()
    }
}
