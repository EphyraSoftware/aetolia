mod calendar_properties;
mod component_properties;
mod error;
mod params;
mod recur;
mod value;

use crate::common::Value;
use crate::model::{CalendarComponent, CalendarProperty, ComponentProperty, ICalObject, Param};
use crate::validate::calendar_properties::validate_calendar_properties;
use crate::validate::component_properties::validate_component_properties;
use crate::validate::error::ICalendarError;
use crate::validate::params::validate_params;
pub use error::*;
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

    let validate_alarms = |errors: &mut Vec<ICalendarError>,
                           alarms: &[CalendarComponent],
                           index: usize,
                           name: &str|
     -> anyhow::Result<()> {
        for (alarm_index, alarm) in alarms.iter().enumerate() {
            errors.extend_from_slice(
                ICalendarError::many_from_nested_component_property_errors(
                    validate_component_properties(
                        &calendar_info,
                        PropertyLocation::Alarm,
                        alarm.properties(),
                    )?,
                    index,
                    name.to_string(),
                    alarm_index,
                    component_name(alarm).to_string(),
                )
                .as_slice(),
            );
        }

        Ok(())
    };

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

                validate_alarms(&mut errors, &event.alarms, index, component_name(component))?;
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

                validate_alarms(&mut errors, &to_do.alarms, index, component_name(component))?;
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
                            // Neither the parser nor the builder will let other subcomponents to
                            // be added here.
                            unreachable!()
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
            _ => {
                // Component at the wrong level will get picked up as IANA components
                unreachable!()
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
    #[allow(dead_code)]
    Other,
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
            value_is_utc: None,
            is_other: false,
            calendar_info,
        }
    }

    fn utc(mut self, is_utc: bool) -> Self {
        self.value_is_utc = Some(is_utc);
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
        Param::AltRep { .. } => "ALTREP",
        Param::CommonName { .. } => "CN",
        Param::CalendarUserType { .. } => "CUTYPE",
        Param::DelegatedFrom { .. } => "DELEGATED-FROM",
        Param::DelegatedTo { .. } => "DELEGATED-TO",
        Param::DirectoryEntryReference { .. } => "DIR",
        Param::Encoding { .. } => "ENCODING",
        Param::FormatType { .. } => "FMTTYPE",
        Param::FreeBusyTimeType { .. } => "FBTYPE",
        Param::Language { .. } => "LANGUAGE",
        Param::Members { .. } => "MEMBER",
        Param::ParticipationStatus { .. } => "PARTSTAT",
        Param::Range { .. } => "RANGE",
        Param::Related { .. } => "RELATED",
        Param::RelationshipType { .. } => "RELTYPE",
        Param::Role { .. } => "ROLE",
        Param::Rsvp { .. } => "RSVP",
        Param::SentBy { .. } => "SENT-BY",
        Param::TimeZoneId { .. } => "TZID",
        Param::ValueType { .. } => "VALUE",
        Param::Other { name, .. } => name,
        Param::Others { name, .. } => name,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convert::ToModel;

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

    macro_rules! assert_errors {
        ($errors:expr, $msg:literal $(,$others:literal)* $(,)?) => {
            assert_errors!($errors, &[$msg, $($others,)*]);
        };

        ($errors:expr, $messages:expr) => {
            similar_asserts::assert_eq!($errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().as_slice(), $messages);
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
    fn calendar_with_no_components() {
        let object = ICalObject::builder()
            .add_product_id("-//hacksw/handcal//NONSGML v1.0//EN")
            .finish_property()
            .add_max_version("2.0")
            .finish_property()
            .build();

        let errors = validate_model(object).unwrap();

        assert_errors!(
            errors,
            "No components found in calendar object, required at least one"
        );
    }

    #[test]
    fn component_with_no_properties() {
        let object = ICalObject::builder()
            .add_product_id("-//hacksw/handcal//NONSGML v1.0//EN")
            .finish_property()
            .add_max_version("2.0")
            .finish_property()
            .add_journal_component()
            .finish_component()
            .build();

        let errors = validate_model(object).unwrap();

        assert_errors!(errors, "In component \"VJOURNAL\" at index 0: No properties found in component, required at least one");
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

        assert_errors!(errors, "In calendar property \"VERSION\" at index 1: Common name (CN) is not allowed for this property type");
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

        assert_errors!(errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Common name (CN) is not allowed for this property type");
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

        assert_errors!(errors, "In calendar property \"VERSION\" at index 1: Calendar user type (CUTYPE) is not allowed for this property type");
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

        assert_errors!(errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Calendar user type (CUTYPE) is not allowed for this property type");
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

        assert_errors!(errors, "In calendar property \"VERSION\" at index 1: Delegated from (DELEGATED-FROM) is not allowed for this property type");
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

        assert_errors!(errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Delegated from (DELEGATED-FROM) is not allowed for this property type");
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

        assert_errors!(errors, "In calendar property \"VERSION\" at index 1: Delegated to (DELEGATED-TO) is not allowed for this property type");
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

        assert_errors!(errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Delegated to (DELEGATED-TO) is not allowed for this property type");
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

        assert_errors!(errors, "In calendar property \"VERSION\" at index 1: Directory entry reference (DIR) is not allowed for this property type");
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

        assert_errors!(errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Directory entry reference (DIR) is not allowed for this property type");
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

        assert_errors!(errors, "In component \"VEVENT\" at index 0, in component property \"ATTACH\" at index 2: Property is declared to have a binary value but no encoding is set, must be set to BASE64");
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

        assert_errors!(errors, "In component \"VEVENT\" at index 0, in component property \"ATTACH\" at index 2: Property is declared to have a binary value but the encoding is set to 8BIT, instead of BASE64");
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

        assert_errors!(
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

        assert_errors!(errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: FMTTYPE is not allowed");
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

        assert_errors!(
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

        assert_errors!(errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: FBTYPE is not allowed");
    }

    #[test]
    fn language_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION;LANGUAGE=en-US:2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In calendar property \"VERSION\" at index 1: LANGUAGE is not allowed"
        );
    }

    #[test]
    fn language_on_date_time_start_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DTSTART;LANGUAGE=en-US:19970101T230000\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(errors, "In component \"VEVENT\" at index 0, in component property \"DTSTART\" at index 2: LANGUAGE is not allowed");
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

        assert_errors!(
            errors,
            "In calendar property \"VERSION\" at index 1: Group or list membership (MEMBER) is not allowed for this property type"
        );
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

        assert_errors!(errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Group or list membership (MEMBER) is not allowed for this property type");
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

        assert_errors!(errors, "In component \"VEVENT\" at index 0, in component property \"ATTENDEE\" at index 2: Invalid participation status (PARTSTAT) value [Completed] in a VEVENT component context");
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

        assert_errors!(&errors,"In component \"VJOURNAL\" at index 0, in component property \"ATTENDEE\" at index 2: Invalid participation status (PARTSTAT) value [InProcess] in a VJOURNAL component context");
    }

    #[test]
    fn part_stat_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION;PARTSTAT=ACCEPTED:2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(errors, "In calendar property \"VERSION\" at index 1: Participation status (PARTSTAT) is not allowed for this property type");
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

        assert_errors!(errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Participation status (PARTSTAT) is not allowed for this property type");
    }

    #[test]
    fn range_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION;RANGE=THISANDFUTURE:2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In calendar property \"VERSION\" at index 1: RANGE is not allowed"
        );
    }

    #[test]
    fn range_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DESCRIPTION;RANGE=THISANDFUTURE:some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: RANGE is not allowed");
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

        assert_errors!(errors, "In calendar property \"VERSION\" at index 1: Related (RELATED) is not allowed for this property type");
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

        assert_errors!(errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Related (RELATED) is not allowed for this property type");
    }

    #[test]
    fn relationship_type_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION;RELTYPE=SIBLING:2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(errors, "In calendar property \"VERSION\" at index 1: Relationship type (RELTYPE) is not allowed for this property type");
    }

    #[test]
    fn relationship_type_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DESCRIPTION;RELTYPE=SIBLING:some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Relationship type (RELTYPE) is not allowed for this property type");
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

        assert_errors!(&errors, "In calendar property \"VERSION\" at index 1: Participation role (ROLE) is not allowed for this property type");
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

        assert_errors!(errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Participation role (ROLE) is not allowed for this property type");
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

        assert_errors!(&errors, "In calendar property \"VERSION\" at index 1: RSVP expectation (RSVP) is not allowed for this property type");
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

        assert_errors!(errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: RSVP expectation (RSVP) is not allowed for this property type");
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

        assert_errors!(errors, "In calendar property \"VERSION\" at index 1: Sent by (SENT-BY) is not allowed for this property type");
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

        assert_errors!(&errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Sent by (SENT-BY) is not allowed for this property type");
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

        assert_errors!(&errors, "In component \"VEVENT\" at index 0, in component property \"ORGANIZER\" at index 2: Sent by (SENT-BY) must be a 'mailto:' URI");
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

        assert_errors!(&errors, "In component \"VEVENT\" at index 0, in component property \"DTSTART\" at index 2: Required time zone ID [missing] is not defined in the calendar");
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

        assert_errors!(&errors, "In component \"VEVENT\" at index 1, in component property \"DTSTART\" at index 2: Time zone ID (TZID) cannot be specified on a property with a UTC time");
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

        assert_errors!(&errors, "In component \"VEVENT\" at index 1, in component property \"DTSTART\" at index 2: Time zone ID (TZID) is not allowed for the property value type DATE");
    }

    #[test]
    fn tz_id_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION;TZID=/test:2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In calendar property \"VERSION\" at index 1: TZID is not allowed"
        );
    }

    #[test]
    fn tz_id_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DESCRIPTION;TZID=/test:some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(&errors, "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: TZID is not allowed");
    }

    #[test]
    fn value_on_version_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION;VALUE=TEXT:2.0\r\n\
BEGIN:X-NONE\r\n\
empty:value\r\n\
END:X-NONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In calendar property \"VERSION\" at index 1: VALUE is not allowed"
        );
    }

    #[test]
    fn value_on_description_property() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DESCRIPTION;VALUE=INTEGER:some text\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: Property is declared to have an integer value but that is not valid for this property",
            "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 2: VALUE is not allowed"
        );
    }

    #[test]
    fn event_missing_required_properties() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
BEGIN:VEVENT\r\n\
X-ANY:test\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VEVENT\" at index 0: DTSTAMP is required",
            "In component \"VEVENT\" at index 0: UID is required",
            "In component \"VEVENT\" at index 0: DTSTART is required",
        );
    }

    #[test]
    fn event_duplicate_optional_once_properties() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DTSTART:19900101T000000Z\r\n\
DTSTART:19900101T000000Z\r\n\
CLASS:PUBLIC\r\n\
CLASS:PUBLIC\r\n\
CREATED:19900101T000000Z\r\n\
CREATED:19900101T000000Z\r\n\
DESCRIPTION:some text\r\n\
DESCRIPTION:some text\r\n\
GEO:1.1;2.2\r\n\
GEO:1.1;2.2\r\n\
LAST-MODIFIED:19900101T000000Z\r\n\
LAST-MODIFIED:19900101T000000Z\r\n\
LOCATION:some location\r\n\
LOCATION:some location\r\n\
ORGANIZER:mailto:hello@test.net\r\n\
ORGANIZER:mailto:hello@test.net\r\n\
PRIORITY:5\r\n\
PRIORITY:5\r\n\
SEQUENCE:0\r\n\
SEQUENCE:0\r\n\
STATUS:CONFIRMED\r\n\
STATUS:CONFIRMED\r\n\
SUMMARY:some summary\r\n\
SUMMARY:some summary\r\n\
TRANSP:OPAQUE\r\n\
TRANSP:OPAQUE\r\n\
URL:http://example.com\r\n\
URL:http://example.com\r\n\
RECURRENCE-ID:19900101T000000Z\r\n\
RECURRENCE-ID:19900101T000000Z\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VEVENT\" at index 0, in component property \"DTSTART\" at index 3: DTSTART must only appear once",
            "In component \"VEVENT\" at index 0, in component property \"CLASS\" at index 5: CLASS must only appear once",
            "In component \"VEVENT\" at index 0, in component property \"CREATED\" at index 7: CREATED must only appear once",
            "In component \"VEVENT\" at index 0, in component property \"DESCRIPTION\" at index 9: DESCRIPTION must only appear once",
            "In component \"VEVENT\" at index 0, in component property \"GEO\" at index 11: GEO must only appear once",
            "In component \"VEVENT\" at index 0, in component property \"LAST-MODIFIED\" at index 13: LAST-MODIFIED must only appear once",
            "In component \"VEVENT\" at index 0, in component property \"LOCATION\" at index 15: LOCATION must only appear once",
            "In component \"VEVENT\" at index 0, in component property \"ORGANIZER\" at index 17: ORGANIZER must only appear once",
            "In component \"VEVENT\" at index 0, in component property \"PRIORITY\" at index 19: PRIORITY must only appear once",
            "In component \"VEVENT\" at index 0, in component property \"SEQUENCE\" at index 21: SEQUENCE must only appear once",
            "In component \"VEVENT\" at index 0, in component property \"STATUS\" at index 23: STATUS must only appear once",
            "In component \"VEVENT\" at index 0, in component property \"SUMMARY\" at index 25: SUMMARY must only appear once",
            "In component \"VEVENT\" at index 0, in component property \"TRANSP\" at index 27: TRANSP must only appear once",
            "In component \"VEVENT\" at index 0, in component property \"URL\" at index 29: URL must only appear once",
            "In component \"VEVENT\" at index 0, in component property \"RECURRENCE-ID\" at index 31: RECURRENCE-ID must only appear once",
        );
    }

    #[test]
    fn event_duplicate_date_time_end() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DTEND:19900101T000000Z\r\n\
DTEND:19900101T000000Z\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VEVENT\" at index 0, in component property \"DTEND\" at index 3: DTEND must only appear once",
        );
    }

    #[test]
    fn event_duplicate_duration() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DURATION:PT1H\r\n\
DURATION:PT1H\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VEVENT\" at index 0, in component property \"DURATION\" at index 3: DURATION must only appear once",
        );
    }

    #[test]
    fn event_both_date_time_end_and_duration() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DTEND:19900101T000000Z\r\n\
DURATION:PT1H\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VEVENT\" at index 0: Both DTEND and DURATION properties are present, only one is allowed",
        );
    }

    #[test]
    fn todo_missing_required_properties() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
BEGIN:VTODO\r\n\
X-ANY:test\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VTODO\" at index 0: DTSTAMP is required",
            "In component \"VTODO\" at index 0: UID is required",
        );
    }

    #[test]
    fn todo_duplicate_optional_once_properties() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
BEGIN:VTODO\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
CLASS:PUBLIC\r\n\
CLASS:PUBLIC\r\n\
COMPLETE:19900101T000000Z\r\n\
COMPLETE:19900101T000000Z\r\n\
CREATED:19900101T000000Z\r\n\
CREATED:19900101T000000Z\r\n\
DESCRIPTION:some text\r\n\
DESCRIPTION:some text\r\n\
DTSTART:19900101T000000Z\r\n\
DTSTART:19900101T000000Z\r\n\
GEO:1.1;2.2\r\n\
GEO:1.1;2.2\r\n\
LAST-MODIFIED:19900101T000000Z\r\n\
LAST-MODIFIED:19900101T000000Z\r\n\
LOCATION:some location\r\n\
LOCATION:some location\r\n\
ORGANIZER:mailto:hello@test.net\r\n\
ORGANIZER:mailto:hello@test.net\r\n\
PERCENT-COMPLETE:50\r\n\
PERCENT-COMPLETE:50\r\n\
PRIORITY:5\r\n\
PRIORITY:5\r\n\
RECURRENCE-ID:19900101T000000Z\r\n\
RECURRENCE-ID:19900101T000000Z\r\n\
SEQUENCE:0\r\n\
SEQUENCE:0\r\n\
STATUS:COMPLETED\r\n\
STATUS:COMPLETED\r\n\
SUMMARY:some summary\r\n\
SUMMARY:some summary\r\n\
URL:http://example.com\r\n\
URL:http://example.com\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VTODO\" at index 0, in component property \"CLASS\" at index 3: CLASS must only appear once",
            "In component \"VTODO\" at index 0, in component property \"CREATED\" at index 7: CREATED must only appear once",
            "In component \"VTODO\" at index 0, in component property \"DESCRIPTION\" at index 9: DESCRIPTION must only appear once",
            "In component \"VTODO\" at index 0, in component property \"DTSTART\" at index 11: DTSTART must only appear once",
            "In component \"VTODO\" at index 0, in component property \"GEO\" at index 13: GEO must only appear once",
            "In component \"VTODO\" at index 0, in component property \"LAST-MODIFIED\" at index 15: LAST-MODIFIED must only appear once",
            "In component \"VTODO\" at index 0, in component property \"LOCATION\" at index 17: LOCATION must only appear once",
            "In component \"VTODO\" at index 0, in component property \"ORGANIZER\" at index 19: ORGANIZER must only appear once",
            "In component \"VTODO\" at index 0, in component property \"PERCENT-COMPLETE\" at index 21: PERCENT-COMPLETE must only appear once",
            "In component \"VTODO\" at index 0, in component property \"PRIORITY\" at index 23: PRIORITY must only appear once",
            "In component \"VTODO\" at index 0, in component property \"RECURRENCE-ID\" at index 25: RECURRENCE-ID must only appear once",
            "In component \"VTODO\" at index 0, in component property \"SEQUENCE\" at index 27: SEQUENCE must only appear once",
            "In component \"VTODO\" at index 0, in component property \"STATUS\" at index 29: STATUS must only appear once",
            "In component \"VTODO\" at index 0, in component property \"SUMMARY\" at index 31: SUMMARY must only appear once",
            "In component \"VTODO\" at index 0, in component property \"URL\" at index 33: URL must only appear once",
        );
    }

    #[test]
    fn todo_duplicate_due() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
BEGIN:VTODO\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DUE:19900101T000000Z\r\n\
DUE:19900101T000000Z\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VTODO\" at index 0, in component property \"DUE\" at index 3: DUE must only appear once",
        );
    }

    #[test]
    fn todo_duplicate_duration() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
BEGIN:VTODO\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DTSTART:19900101T000000Z\r\n\
DURATION:PT1H\r\n\
DURATION:PT1H\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VTODO\" at index 0, in component property \"DURATION\" at index 4: DURATION must only appear once",
        );
    }

    #[test]
    fn todo_duration_without_date_time_start() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
BEGIN:VTODO\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DURATION:PT1H\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VTODO\" at index 0: DURATION property is present but no DTSTART property is present",
        );
    }

    #[test]
    fn todo_both_due_and_duration() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
BEGIN:VTODO\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DTSTART:19900101T000000Z\r\n\
DUE:19900101T000000Z\r\n\
DURATION:PT1H\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VTODO\" at index 0: Both DUE and DURATION properties are present, only one is allowed",
        );
    }

    #[test]
    fn journal_missing_required_properties() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
BEGIN:VJOURNAL\r\n\
X-ANY:test\r\n\
END:VJOURNAL\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VJOURNAL\" at index 0: DTSTAMP is required",
            "In component \"VJOURNAL\" at index 0: UID is required",
        );
    }

    #[test]
    fn journal_duplicate_optional_once_properties() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
BEGIN:VJOURNAL\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
CLASS:PUBLIC\r\n\
CLASS:PUBLIC\r\n\
CREATED:19900101T000000Z\r\n\
CREATED:19900101T000000Z\r\n\
DTSTART:19900101T000000Z\r\n\
DTSTART:19900101T000000Z\r\n\
LAST-MODIFIED:19900101T000000Z\r\n\
LAST-MODIFIED:19900101T000000Z\r\n\
ORGANIZER:mailto:hello@test.net\r\n\
ORGANIZER:mailto:hello@test.net\r\n\
RECURRENCE-ID:19900101T000000Z\r\n\
RECURRENCE-ID:19900101T000000Z\r\n\
SEQUENCE:0\r\n\
SEQUENCE:0\r\n\
STATUS:FINAL\r\n\
STATUS:FINAL\r\n\
SUMMARY:some summary\r\n\
SUMMARY:some summary\r\n\
URL:http://example.com\r\n\
URL:http://example.com\r\n\
END:VJOURNAL\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VJOURNAL\" at index 0, in component property \"CLASS\" at index 3: CLASS must only appear once",
            "In component \"VJOURNAL\" at index 0, in component property \"CREATED\" at index 5: CREATED must only appear once",
            "In component \"VJOURNAL\" at index 0, in component property \"DTSTART\" at index 7: DTSTART must only appear once",
            "In component \"VJOURNAL\" at index 0, in component property \"LAST-MODIFIED\" at index 9: LAST-MODIFIED must only appear once",
            "In component \"VJOURNAL\" at index 0, in component property \"ORGANIZER\" at index 11: ORGANIZER must only appear once",
            "In component \"VJOURNAL\" at index 0, in component property \"RECURRENCE-ID\" at index 13: RECURRENCE-ID must only appear once",
            "In component \"VJOURNAL\" at index 0, in component property \"SEQUENCE\" at index 15: SEQUENCE must only appear once",
            "In component \"VJOURNAL\" at index 0, in component property \"STATUS\" at index 17: STATUS must only appear once",
            "In component \"VJOURNAL\" at index 0, in component property \"SUMMARY\" at index 19: SUMMARY must only appear once",
            "In component \"VJOURNAL\" at index 0, in component property \"URL\" at index 21: URL must only appear once"
        );
    }

    #[test]
    fn free_busy_missing_required_properties() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
BEGIN:VFREEBUSY\r\n\
X-ANY:test\r\n\
END:VFREEBUSY\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VFREEBUSY\" at index 0: DTSTAMP is required",
            "In component \"VFREEBUSY\" at index 0: UID is required",
        );
    }

    #[test]
    fn free_busy_duplicate_optional_once_properties() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
BEGIN:VFREEBUSY\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
CONTACT:mailto:hello@test.net\r\n\
CONTACT:mailto:hello@test.net\r\n\
DTSTART:19900101T000000Z\r\n\
DTSTART:19900101T000000Z\r\n\
DTEND:19900101T000000Z\r\n\
DTEND:19900101T000000Z\r\n\
ORGANIZER:mailto:admin@test.net\r\n\
ORGANIZER:mailto:admin@test.net\r\n\
URL:http://example.com\r\n\
URL:http://example.com\r\n\
END:VFREEBUSY\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VFREEBUSY\" at index 0, in component property \"CONTACT\" at index 3: CONTACT must only appear once",
            "In component \"VFREEBUSY\" at index 0, in component property \"DTSTART\" at index 5: DTSTART must only appear once",
            "In component \"VFREEBUSY\" at index 0, in component property \"DTEND\" at index 7: DTEND must only appear once",
            "In component \"VFREEBUSY\" at index 0, in component property \"ORGANIZER\" at index 9: ORGANIZER must only appear once",
            "In component \"VFREEBUSY\" at index 0, in component property \"URL\" at index 11: URL must only appear once",
        );
    }

    #[test]
    fn time_zone_missing_required_properties() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
BEGIN:VTIMEZONE\r\n\
X-ANY:test\r\n\
BEGIN:STANDARD\r\n\
DTSTART:19900101T000000\r\n\
TZOFFSETTO:+0000\r\n\
TZOFFSETFROM:+0000\r\n\
END:STANDARD\r\n\
END:VTIMEZONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VTIMEZONE\" at index 0: TZID is required",
        );
    }

    #[test]
    fn time_zone_missing_required_nested_components() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
BEGIN:VTIMEZONE\r\n\
TZID:America/New_York\r\n\
END:VTIMEZONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VTIMEZONE\" at index 0: No standard or daylight components found in time zone, required at least one",
        );
    }

    #[test]
    fn time_zone_duplicate_optional_once_properties() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
BEGIN:VTIMEZONE\r\n\
TZID:America/New_York\r\n\
LAST-MODIFIED:20050809T050000Z\r\n\
LAST-MODIFIED:20050809T050000Z\r\n\
TZURL:http://example.com\r\n\
TZURL:http://example.com\r\n\
BEGIN:STANDARD\r\n\
DTSTART:19900101T000000\r\n\
TZOFFSETTO:+0000\r\n\
TZOFFSETFROM:+0000\r\n\
END:STANDARD\r\n\
END:VTIMEZONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VTIMEZONE\" at index 0, in component property \"LAST-MODIFIED\" at index 2: LAST-MODIFIED must only appear once",
            "In component \"VTIMEZONE\" at index 0, in component property \"TZURL\" at index 4: TZURL must only appear once",
        );
    }

    #[test]
    fn time_zone_nested_missing_required_properties() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
BEGIN:VTIMEZONE\r\n\
TZID:America/New_York\r\n\
BEGIN:STANDARD\r\n\
X-ANY:test\r\n\
END:STANDARD\r\n\
BEGIN:DAYLIGHT\r\n\
X-ANY:test\r\n\
END:DAYLIGHT\r\n\
END:VTIMEZONE\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VTIMEZONE\" at index 0, in nested component \"STANDARD\" at index 0: DTSTART is required",
            "In component \"VTIMEZONE\" at index 0, in nested component \"STANDARD\" at index 0: TZOFFSETTO is required",
            "In component \"VTIMEZONE\" at index 0, in nested component \"STANDARD\" at index 0: TZOFFSETFROM is required",
            "In component \"VTIMEZONE\" at index 0, in nested component \"DAYLIGHT\" at index 1: DTSTART is required",
            "In component \"VTIMEZONE\" at index 0, in nested component \"DAYLIGHT\" at index 1: TZOFFSETTO is required",
            "In component \"VTIMEZONE\" at index 0, in nested component \"DAYLIGHT\" at index 1: TZOFFSETFROM is required",
        );
    }

    #[test]
    fn alarm_missing_action() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
BEGIN:VALARM\r\n\
X-ANY:test\r\n\
END:VALARM\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VEVENT\" at index 0, in nested component \"VALARM\" at index 0: Required exactly one ACTION property but found 0",
        );
    }

    #[test]
    fn alarm_missing_required_properties() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
BEGIN:VALARM\r\n\
ACTION:AUDIO\r\n\
END:VALARM\r\n\
BEGIN:VALARM\r\n\
ACTION:DISPLAY\r\n\
END:VALARM\r\n\
BEGIN:VALARM\r\n\
ACTION:EMAIL\r\n\
END:VALARM\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VEVENT\" at index 0, in nested component \"VALARM\" at index 0: TRIGGER is required",
            "In component \"VEVENT\" at index 0, in nested component \"VALARM\" at index 1: TRIGGER is required",
            "In component \"VEVENT\" at index 0, in nested component \"VALARM\" at index 1: DESCRIPTION is required",
            "In component \"VEVENT\" at index 0, in nested component \"VALARM\" at index 2: TRIGGER is required",
            "In component \"VEVENT\" at index 0, in nested component \"VALARM\" at index 2: DESCRIPTION is required",
            "In component \"VEVENT\" at index 0, in nested component \"VALARM\" at index 2: SUMMARY is required",
        );
    }

    #[test]
    fn alarm_missing_duplicate_optional_once_properties() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
BEGIN:VALARM\r\n\
ACTION:AUDIO\r\n\
TRIGGER:P3W\r\n\
DURATION:PT15M\r\n\
DURATION:PT15M\r\n\
REPEAT:2\r\n\
REPEAT:2\r\n\
ATTACH:ftp://example.com/pub/sounds/bell-01.aud\r\n\
ATTACH:ftp://example.com/pub/sounds/bell-01.aud\r\n\
END:VALARM\r\n\
BEGIN:VALARM\r\n\
ACTION:DISPLAY\r\n\
TRIGGER:P3W\r\n\
DESCRIPTION:Breakfast meeting with executive\r\n\
DURATION:PT15M\r\n\
DURATION:PT15M\r\n\
REPEAT:2\r\n\
REPEAT:2\r\n\
END:VALARM\r\n\
BEGIN:VALARM\r\n\
ACTION:EMAIL\r\n\
TRIGGER:P3W\r\n\
SUMMARY:New event\r\n\
DESCRIPTION:Breakfast meeting with executive\r\n\
DURATION:PT15M\r\n\
DURATION:PT15M\r\n\
REPEAT:2\r\n\
REPEAT:2\r\n\
END:VALARM\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VEVENT\" at index 0, in nested component \"VALARM\" at index 0, in nested component property \"DURATION\" at index 3: DURATION must only appear once",
            "In component \"VEVENT\" at index 0, in nested component \"VALARM\" at index 0, in nested component property \"REPEAT\" at index 5: REPEAT must only appear once",
            "In component \"VEVENT\" at index 0, in nested component \"VALARM\" at index 0, in nested component property \"ATTACH\" at index 7: ATTACH must only appear once",
            "In component \"VEVENT\" at index 0, in nested component \"VALARM\" at index 1, in nested component property \"DURATION\" at index 4: DURATION must only appear once",
            "In component \"VEVENT\" at index 0, in nested component \"VALARM\" at index 1, in nested component property \"REPEAT\" at index 6: REPEAT must only appear once",
            "In component \"VEVENT\" at index 0, in nested component \"VALARM\" at index 2, in nested component property \"DURATION\" at index 5: DURATION must only appear once",
            "In component \"VEVENT\" at index 0, in nested component \"VALARM\" at index 2, in nested component property \"REPEAT\" at index 7: REPEAT must only appear once",
        );
    }

    #[test]
    fn alarm_duration_and_trigger_not_present_together() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
BEGIN:VALARM\r\n\
ACTION:AUDIO\r\n\
TRIGGER:P3W\r\n\
DURATION:PT15M\r\n\
END:VALARM\r\n\
BEGIN:VALARM\r\n\
ACTION:AUDIO\r\n\
TRIGGER:P3W\r\n\
REPEAT:2\r\n\
END:VALARM\r\n\
BEGIN:VALARM\r\n\
ACTION:DISPLAY\r\n\
TRIGGER:P3W\r\n\
DESCRIPTION:Breakfast meeting with executive\r\n\
DURATION:PT15M\r\n\
END:VALARM\r\n\
BEGIN:VALARM\r\n\
ACTION:DISPLAY\r\n\
TRIGGER:P3W\r\n\
DESCRIPTION:Breakfast meeting with executive\r\n\
REPEAT:2\r\n\
END:VALARM\r\n\
BEGIN:VALARM\r\n\
ACTION:EMAIL\r\n\
TRIGGER:P3W\r\n\
SUMMARY:New event\r\n\
DESCRIPTION:Breakfast meeting with executive\r\n\
DURATION:PT15M\r\n\
END:VALARM\r\n\
BEGIN:VALARM\r\n\
ACTION:EMAIL\r\n\
TRIGGER:P3W\r\n\
SUMMARY:New event\r\n\
DESCRIPTION:Breakfast meeting with executive\r\n\
REPEAT:2\r\n\
END:VALARM\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VEVENT\" at index 0, in nested component \"VALARM\" at index 0: DURATION and REPEAT properties must be present together",
            "In component \"VEVENT\" at index 0, in nested component \"VALARM\" at index 1: DURATION and REPEAT properties must be present together",
            "In component \"VEVENT\" at index 0, in nested component \"VALARM\" at index 2: DURATION and REPEAT properties must be present together",
            "In component \"VEVENT\" at index 0, in nested component \"VALARM\" at index 3: DURATION and REPEAT properties must be present together",
            "In component \"VEVENT\" at index 0, in nested component \"VALARM\" at index 4: DURATION and REPEAT properties must be present together",
            "In component \"VEVENT\" at index 0, in nested component \"VALARM\" at index 5: DURATION and REPEAT properties must be present together",
        );
    }

    #[test]
    fn default_value_specified() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
ATTACH;VALUE=URI:ftp://example.com/pub/sounds/bell-01.aud\r\n\
DTEND;VALUE=DATE-TIME:19900101T000000Z\r\n\
DTSTART;VALUE=DATE-TIME:19900101T000000Z\r\n\
EXDATE;VALUE=DATE-TIME:19900101T000000Z\r\n\
RDATE;VALUE=DATE-TIME:19900101T000000Z\r\n\
BEGIN:VALARM\r\n\
ACTION:DISPLAY\r\n\
DESCRIPTION:Breakfast meeting with executive\r\n\
TRIGGER;VALUE=DURATION:PT15M\r\n\
END:VALARM\r\n\
END:VEVENT\r\n\
BEGIN:VTODO\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DUE;VALUE=DATE-TIME:19900101T000000Z\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VEVENT\" at index 0, in component property \"ATTACH\" at index 2: Redundant value specification which matches the default value",
            "In component \"VEVENT\" at index 0, in component property \"DTEND\" at index 3: Redundant value specification which matches the default value",
            "In component \"VEVENT\" at index 0, in component property \"DTSTART\" at index 4: Redundant value specification which matches the default value",
            "In component \"VEVENT\" at index 0, in component property \"EXDATE\" at index 5: Redundant value specification which matches the default value",
            "In component \"VEVENT\" at index 0, in component property \"RDATE\" at index 6: Redundant value specification which matches the default value",
            "In component \"VEVENT\" at index 0, in nested component \"VALARM\" at index 0, in nested component property \"TRIGGER\" at index 2: Redundant value specification which matches the default value",
            "In component \"VTODO\" at index 1, in component property \"DUE\" at index 2: Redundant value specification which matches the default value",
        );
    }

    #[test]
    fn iana_component() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
BEGIN:ANY\r\n\
DTSTART:19900101T000000Z\r\n\
END:ANY\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_no_errors!(errors);
    }

    #[test]
    fn standard_at_top_level() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
BEGIN:DAYLIGHT\r\n\
X-ANY:test\r\n\
END:DAYLIGHT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        // Gets picked up as IANA
        assert_no_errors!(errors);
    }

    #[test]
    fn x_property_value_type_checks() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
X-TIME-INVALID-HOUR;VALUE=TIME:250000\r\n\
X-TIME-INVALID-MINUTE;VALUE=TIME:007000\r\n\
X-TIME-INVALID-SECOND;VALUE=TIME:000070\r\n\
X-TIME-VALID;VALUE=TIME:235960\r\n\
X-UTC-OFFSET-NEGATIVE-ZERO;VALUE=UTC-OFFSET:-000000\r\n\
X-UTC-OFFSET-NEGATIVE-NON-ZERO;VALUE=UTC-OFFSET:-000001\r\n\
X-UTC-OFFSET-INVALID-MINUTE;VALUE=UTC-OFFSET:+006000\r\n\
X-BASE-64;VALUE=BINARY;ENCODING=8BIT:nope\r\n\
X-BASE-64;VALUE=BINARY;ENCODING=BASE64:##\r\n\
X-BOOLEAN;VALUE=BOOLEAN:wendy\r\n\
X-CAL-ADDRESS-NOT-URL;VALUE=CAL-ADDRESS:test\r\n\
X-CAL-ADDRESS-NOT-MAILTO;VALUE=CAL-ADDRESS:mailto:hello@test.net\r\n\
X-DATE;VALUE=DATE:19900101T000120\r\n\
X-DATE-TIME;VALUE=DATE-TIME:19900101T000000P\r\n\
X-DURATION;VALUE=DURATION:3W\r\n\
X-FLOAT;VALUE=FLOAT:3.14.15\r\n\
X-INTEGER;VALUE=INTEGER:3.14\r\n\
X-PERIOD;VALUE=PERIOD:19900101T000000Z/19900101T000000W\r\n\
X-RECUR;VALUE=RECUR:19900101T000000Z\r\n\
X-TEXT;VALUE=TEXT:\\p\r\n\
X-URI;VALUE=URI:hello\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VEVENT\" at index 0, in component property \"X-TIME-INVALID-HOUR\" at index 2: Found an invalid time at index 0 - Hour must be between 0 and 23",
            "In component \"VEVENT\" at index 0, in component property \"X-TIME-INVALID-MINUTE\" at index 3: Found an invalid time at index 0 - Minute must be between 0 and 59",
            "In component \"VEVENT\" at index 0, in component property \"X-TIME-INVALID-SECOND\" at index 4: Found an invalid time at index 0 - Second must be between 0 and 60",
            "In component \"VEVENT\" at index 0, in component property \"X-UTC-OFFSET-NEGATIVE-ZERO\" at index 6: Found an invalid UTC offset - UTC offset must have a non-zero value if it is negative",
            "In component \"VEVENT\" at index 0, in component property \"X-UTC-OFFSET-INVALID-MINUTE\" at index 8: Found an invalid UTC offset - Minutes must be between 0 and 59",
            "In component \"VEVENT\" at index 0, in component property \"X-BASE-64\" at index 9: Property is declared to have a binary value but the encoding is set to 8BIT, instead of BASE64",
            "In component \"VEVENT\" at index 0, in component property \"X-BASE-64\" at index 10: Property is declared to have a binary value but the value is not base64",
            "In component \"VEVENT\" at index 0, in component property \"X-BOOLEAN\" at index 11: Property is declared to have a boolean value but the value is not a boolean",
            "In component \"VEVENT\" at index 0, in component property \"X-CAL-ADDRESS-NOT-URL\" at index 12: Property is declared to have a calendar address value but that is not valid for this property",
            "In component \"VEVENT\" at index 0, in component property \"X-CAL-ADDRESS-NOT-URL\" at index 12: Property is declared to have a calendar address value but the value is a mailto: URI",
            "In component \"VEVENT\" at index 0, in component property \"X-DATE\" at index 14: Property is declared to have a date value but the value is not a date",
            "In component \"VEVENT\" at index 0, in component property \"X-DATE-TIME\" at index 15: Property is declared to have a date-time value but the value is not a date-time",
            "In component \"VEVENT\" at index 0, in component property \"X-DURATION\" at index 16: Property is declared to have a duration value but the value is not a duration",
            "In component \"VEVENT\" at index 0, in component property \"X-FLOAT\" at index 17: Property is declared to have a float value but the value is not a float",
            "In component \"VEVENT\" at index 0, in component property \"X-INTEGER\" at index 18: Property is declared to have an integer value but the value is not an integer",
            "In component \"VEVENT\" at index 0, in component property \"X-PERIOD\" at index 19: Property is declared to have a period value but the value is not a period",
            "In component \"VEVENT\" at index 0, in component property \"X-RECUR\" at index 20: Property is declared to have a recurrence value but the value is not a recurrence",
            "In component \"VEVENT\" at index 0, in component property \"X-TEXT\" at index 21: Property is declared to have a text value but the value is not a text",
            "In component \"VEVENT\" at index 0, in component property \"X-URI\" at index 22: Property is declared to have a URI value but the value is not a URI",
        );
    }

    #[test]
    fn iana_property_value_type_checks() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
TIME-INVALID-HOUR;VALUE=TIME:250000\r\n\
TIME-INVALID-MINUTE;VALUE=TIME:007000\r\n\
TIME-INVALID-SECOND;VALUE=TIME:000070\r\n\
TIME-VALID;VALUE=TIME:235960\r\n\
UTC-OFFSET-NEGATIVE-ZERO;VALUE=UTC-OFFSET:-000000\r\n\
UTC-OFFSET-NEGATIVE-NON-ZERO;VALUE=UTC-OFFSET:-000001\r\n\
UTC-OFFSET-INVALID-MINUTE;VALUE=UTC-OFFSET:+006000\r\n\
BASE-64;VALUE=BINARY;ENCODING=8BIT:nope\r\n\
BASE-64;VALUE=BINARY;ENCODING=BASE64:##\r\n\
BOOLEAN;VALUE=BOOLEAN:wendy\r\n\
CAL-ADDRESS-NOT-URL;VALUE=CAL-ADDRESS:test\r\n\
CAL-ADDRESS-NOT-MAILTO;VALUE=CAL-ADDRESS:mailto:hello@test.net\r\n\
DATE;VALUE=DATE:19900101T000120\r\n\
DATE-TIME;VALUE=DATE-TIME:19900101T000000P\r\n\
OTHER-DURATION;VALUE=DURATION:3W\r\n\
FLOAT;VALUE=FLOAT:3.14.15\r\n\
INTEGER;VALUE=INTEGER:3.14\r\n\
PERIOD;VALUE=PERIOD:19900101T000000Z/19900101T000000W\r\n\
RECUR;VALUE=RECUR:19900101T000000Z\r\n\
TEXT;VALUE=TEXT:\\p\r\n\
OTHER-URI;VALUE=URI:hello\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VEVENT\" at index 0, in component property \"TIME-INVALID-HOUR\" at index 2: Found an invalid time at index 0 - Hour must be between 0 and 23",
            "In component \"VEVENT\" at index 0, in component property \"TIME-INVALID-MINUTE\" at index 3: Found an invalid time at index 0 - Minute must be between 0 and 59",
            "In component \"VEVENT\" at index 0, in component property \"TIME-INVALID-SECOND\" at index 4: Found an invalid time at index 0 - Second must be between 0 and 60",
            "In component \"VEVENT\" at index 0, in component property \"UTC-OFFSET-NEGATIVE-ZERO\" at index 6: Found an invalid UTC offset - UTC offset must have a non-zero value if it is negative",
            "In component \"VEVENT\" at index 0, in component property \"UTC-OFFSET-INVALID-MINUTE\" at index 8: Found an invalid UTC offset - Minutes must be between 0 and 59",
            "In component \"VEVENT\" at index 0, in component property \"BASE-64\" at index 9: Property is declared to have a binary value but the encoding is set to 8BIT, instead of BASE64",
            "In component \"VEVENT\" at index 0, in component property \"BASE-64\" at index 10: Property is declared to have a binary value but the value is not base64",
            "In component \"VEVENT\" at index 0, in component property \"BOOLEAN\" at index 11: Property is declared to have a boolean value but the value is not a boolean",
            "In component \"VEVENT\" at index 0, in component property \"CAL-ADDRESS-NOT-URL\" at index 12: Property is declared to have a calendar address value but that is not valid for this property",
            "In component \"VEVENT\" at index 0, in component property \"CAL-ADDRESS-NOT-URL\" at index 12: Property is declared to have a calendar address value but the value is a mailto: URI",
            "In component \"VEVENT\" at index 0, in component property \"DATE\" at index 14: Property is declared to have a date value but the value is not a date",
            "In component \"VEVENT\" at index 0, in component property \"DATE-TIME\" at index 15: Property is declared to have a date-time value but the value is not a date-time",
            "In component \"VEVENT\" at index 0, in component property \"OTHER-DURATION\" at index 16: Property is declared to have a duration value but the value is not a duration",
            "In component \"VEVENT\" at index 0, in component property \"FLOAT\" at index 17: Property is declared to have a float value but the value is not a float",
            "In component \"VEVENT\" at index 0, in component property \"INTEGER\" at index 18: Property is declared to have an integer value but the value is not an integer",
            "In component \"VEVENT\" at index 0, in component property \"PERIOD\" at index 19: Property is declared to have a period value but the value is not a period",
            "In component \"VEVENT\" at index 0, in component property \"RECUR\" at index 20: Property is declared to have a recurrence value but the value is not a recurrence",
            "In component \"VEVENT\" at index 0, in component property \"TEXT\" at index 21: Property is declared to have a text value but the value is not a text",
            "In component \"VEVENT\" at index 0, in component property \"OTHER-URI\" at index 22: Property is declared to have a URI value but the value is not a URI",
        );
    }

    #[test]
    fn recur_invalid_occurrence() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DTSTART:19900101T000000Z\r\n\
RRULE:FREQ=MONTHLY;COUNT=5;BYDAY=1SU\r\n\
RRULE:COUNT=5\r\n\
RRULE:COUNT=5;FREQ=MONTHLY;BYDAY=1SU\r\n\
RRULE:FREQ=MONTHLY;FREQ=WEEKLY;BYDAY=1SU\r\n\
RRULE:FREQ=WEEKLY;UNTIL=19900101T000000Z;UNTIL=19900101T000001Z\r\n\
RRULE:FREQ=WEEKLY;COUNT=3;COUNT=5\r\n\
RRULE:FREQ=WEEKLY;INTERVAL=2;INTERVAL=2\r\n\
RRULE:FREQ=WEEKLY;BYSECOND=1;BYSECOND=1\r\n\
RRULE:FREQ=WEEKLY;BYMINUTE=1;BYMINUTE=1\r\n\
RRULE:FREQ=WEEKLY;BYHOUR=1;BYHOUR=1\r\n\
RRULE:FREQ=MONTHLY;BYDAY=1SU;BYDAY=1SU\r\n\
RRULE:FREQ=YEARLY;BYMONTHDAY=1;BYMONTHDAY=1\r\n\
RRULE:FREQ=YEARLY;BYYEARDAY=1;BYYEARDAY=1\r\n\
RRULE:FREQ=YEARLY;BYWEEKNO=1;BYWEEKNO=1\r\n\
RRULE:FREQ=WEEKLY;BYMONTH=1;BYMONTH=1\r\n\
RRULE:FREQ=WEEKLY;INTERVAL=2;BYDAY=SU;WKST=SU;WKST=SU\r\n\
RRULE:FREQ=YEARLY;BYDAY=1SU;BYSETPOS=1;BYSETPOS=1\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VEVENT\" at index 0, in component property \"RRULE\" at index 4: No frequency part found in recurrence rule, but it is required. This prevents the rest of the rule being checked",
             "In component \"VEVENT\" at index 0, in component property \"RRULE\" at index 5: Recurrence rule must start with a frequency",
            "In component \"VEVENT\" at index 0, in component property \"RRULE\" at index 6: Repeated FREQ part at index 1",
            "In component \"VEVENT\" at index 0, in component property \"RRULE\" at index 7: Repeated UNTIL part at index 2",
            "In component \"VEVENT\" at index 0, in component property \"RRULE\" at index 8: Repeated COUNT part at index 2",
            "In component \"VEVENT\" at index 0, in component property \"RRULE\" at index 9: Repeated INTERVAL part at index 2",
            "In component \"VEVENT\" at index 0, in component property \"RRULE\" at index 10: Repeated BYSECOND part at index 2",
            "In component \"VEVENT\" at index 0, in component property \"RRULE\" at index 11: Repeated BYMINUTE part at index 2",
            "In component \"VEVENT\" at index 0, in component property \"RRULE\" at index 12: Repeated BYHOUR part at index 2",
            "In component \"VEVENT\" at index 0, in component property \"RRULE\" at index 13: Repeated BYDAY part at index 2",
            "In component \"VEVENT\" at index 0, in component property \"RRULE\" at index 14: Repeated BYMONTHDAY part at index 2",
            "In component \"VEVENT\" at index 0, in component property \"RRULE\" at index 15: Repeated BYYEARDAY part at index 2",
            "In component \"VEVENT\" at index 0, in component property \"RRULE\" at index 16: Repeated BYWEEKNO part at index 2",
            "In component \"VEVENT\" at index 0, in component property \"RRULE\" at index 17: Repeated BYMONTH part at index 2",
            "In component \"VEVENT\" at index 0, in component property \"RRULE\" at index 18: Repeated WKST part at index 4",
            "In component \"VEVENT\" at index 0, in component property \"RRULE\" at index 19: Repeated BYSETPOS part at index 3",
        );
    }

    #[test]
    fn recur_invalid_time_range() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
METHOD:send\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:123\r\n\
DTSTART:19900101T000000Z\r\n\
RRULE:FREQ=WEEKLY;BYSECOND=74\r\n\
RRULE:FREQ=WEEKLY;BYMINUTE=98\r\n\
RRULE:FREQ=WEEKLY;BYHOUR=25\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VEVENT\" at index 0, in component property \"RRULE\" at index 3: Invalid BYSECOND part at index 1, seconds must be between 0 and 60",
            "In component \"VEVENT\" at index 0, in component property \"RRULE\" at index 4: Invalid BYMINUTE part at index 1, minutes must be between 0 and 59",
            "In component \"VEVENT\" at index 0, in component property \"RRULE\" at index 5: Invalid BYHOUR part at index 1, hours must be between 0 and 23",
        );
    }

    #[test]
    fn recur_mismatched_date_time_start_type() {
        let content = "BEGIN:VCALENDAR\r\n\
PRODID:test\r\n\
VERSION:2.0\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:1\r\n\
DTSTART;VALUE=DATE:19900101\r\n\
RRULE:FREQ=WEEKLY;UNTIL=19900101T000000Z\r\n\
END:VEVENT\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:2\r\n\
DTSTART:19900101T000000Z\r\n\
RRULE:FREQ=WEEKLY;UNTIL=19900101\r\n\
END:VEVENT\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:3\r\n\
DTSTART:19900101T000000Z\r\n\
RRULE:FREQ=WEEKLY;UNTIL=19900101T000000\r\n\
END:VEVENT\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:4\r\n\
DTSTART;TZID=/America/New_York:19900101T000000\r\n\
RRULE:FREQ=WEEKLY;UNTIL=19900101T000000\r\n\
END:VEVENT\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:19900101T000000Z\r\n\
UID:4\r\n\
DTSTART:19900101T000000\r\n\
RRULE:FREQ=WEEKLY;UNTIL=19900101T000000Z\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(
            errors,
            "In component \"VEVENT\" at index 0, in component property \"RRULE\" at index 3: UNTIL part at index 1 is a date-time, but the associated DTSTART property is a date",
            "In component \"VEVENT\" at index 1, in component property \"RRULE\" at index 3: UNTIL part at index 1 is a date, but the associated DTSTART property is a date-time",
            "In component \"VEVENT\" at index 2, in component property \"RRULE\" at index 3: UNTIL part at index 1 must be a UTC time if the associated DTSTART property is a UTC time or a local time with a timezone",
            "In component \"VEVENT\" at index 3, in component property \"RRULE\" at index 3: UNTIL part at index 1 must be a UTC time if the associated DTSTART property is a UTC time or a local time with a timezone",
            "In component \"VEVENT\" at index 4, in component property \"RRULE\" at index 3: UNTIL part at index 1 must be a local time if the associated DTSTART property is a local time",
        );
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
TRIGGER:P3W\r\n\
ATTACH:ftp://example.com/pub/sounds/bell-01.aud\r\n\
ATTACH:ftp://example.com/pub/sounds/bell-01.aud\r\n\
END:VALARM\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let errors = validate_content(content);

        assert_errors!(&errors, "In component \"VEVENT\" at index 0, in nested component \"VALARM\" at index 0, in nested component property \"ATTACH\" at index 3: ATTACH must only appear once");
    }

    fn validate_content(content: &str) -> Vec<ICalendarError> {
        let (rem, object) = crate::parser::ical_object::<Error>(content.as_bytes()).unwrap();
        check_rem(rem, 0);

        validate_model(object.to_model().unwrap()).unwrap()
    }
}
