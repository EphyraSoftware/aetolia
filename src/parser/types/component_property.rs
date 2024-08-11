use crate::common::{Status, TimeTransparency};
use crate::parser::types::{IanaProperty, XProperty};
use crate::parser::uri::Uri;
use crate::parser::{
    DateOrDateTime, DateOrDateTimeOrPeriod, DateTime, Duration, ParamValue, Period, RecurRulePart,
    UtcOffset,
};

#[derive(Debug, PartialEq)]
pub enum ComponentProperty<'a> {
    DateTimeStamp(DateTimeStampProperty<'a>),
    UniqueIdentifier(UniqueIdentifierProperty<'a>),
    DateTimeStart(DateTimeStartProperty<'a>),
    Classification(ClassificationProperty<'a>),
    DateTimeCreated(CreatedProperty<'a>),
    Description(DescriptionProperty<'a>),
    GeographicPosition(GeographicPositionProperty<'a>),
    LastModified(LastModifiedProperty<'a>),
    Location(LocationProperty<'a>),
    Organizer(OrganizerProperty<'a>),
    Priority(PriorityProperty<'a>),
    Sequence(SequenceProperty<'a>),
    Status(StatusProperty<'a>),
    Summary(SummaryProperty<'a>),
    TimeTransparency(TimeTransparencyProperty<'a>),
    Url(UrlProperty<'a>),
    RecurrenceId(RecurrenceIdProperty<'a>),
    RecurrenceRule(RecurrenceRuleProperty<'a>),
    DateTimeEnd(DateTimeEndProperty<'a>),
    Duration(DurationProperty<'a>),
    Attach(AttachProperty<'a>),
    Attendee(AttendeeProperty<'a>),
    Categories(CategoriesProperty<'a>),
    Comment(CommentProperty<'a>),
    Contact(ContactProperty<'a>),
    ExceptionDateTimes(ExceptionDateTimesProperty<'a>),
    RequestStatus(RequestStatusProperty<'a>),
    RelatedTo(RelatedToProperty<'a>),
    Resources(ResourcesProperty<'a>),
    RecurrenceDateTimes(RecurrenceDateTimesProperty<'a>),
    DateTimeCompleted(DateTimeCompletedProperty<'a>),
    PercentComplete(PercentCompleteProperty<'a>),
    DateTimeDue(DateTimeDueProperty<'a>),
    FreeBusyTime(FreeBusyTimeProperty<'a>),
    TimeZoneId(TimeZoneIdProperty<'a>),
    TimeZoneUrl(TimeZoneUrlProperty<'a>),
    TimeZoneOffsetTo(TimeZoneOffsetProperty<'a>),
    TimeZoneOffsetFrom(TimeZoneOffsetProperty<'a>),
    TimeZoneName(TimeZoneNameProperty<'a>),
    Action(ActionProperty<'a>),
    Trigger(TriggerProperty<'a>),
    RepeatCount(RepeatProperty<'a>),
    XProperty(XProperty<'a>),
    IanaProperty(IanaProperty<'a>),
}

#[derive(Debug, Eq, PartialEq)]
pub enum AttachValue<'a> {
    Uri(&'a [u8]),
    Binary(&'a [u8]),
}

#[derive(Debug, Eq, PartialEq)]
pub struct AttachProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: AttachValue<'a>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CategoriesProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: Vec<Vec<u8>>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Classification<'a> {
    Public,
    Private,
    Confidential,
    XName(&'a [u8]),
    IanaToken(&'a [u8]),
}

#[derive(Debug, Eq, PartialEq)]
pub struct ClassificationProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: Classification<'a>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CommentProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct DescriptionProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub struct GeographicPositionProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct LocationProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct PercentCompleteProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: u8,
}

#[derive(Debug, Eq, PartialEq)]
pub struct PriorityProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: u8,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ResourcesProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: Vec<Vec<u8>>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct StatusProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: Status,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SummaryProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct DateTimeCompletedProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: DateTime,
}

#[derive(Debug, Eq, PartialEq)]
pub struct DateTimeEndProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: DateOrDateTime,
}

#[derive(Debug, Eq, PartialEq)]
pub struct DateTimeDueProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: DateOrDateTime,
}

#[derive(Debug, Eq, PartialEq)]
pub struct DateTimeStartProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: DateOrDateTime,
}

#[derive(Debug, Eq, PartialEq)]
pub struct DurationProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: Duration,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FreeBusyTimeProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: Vec<Period>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TimeTransparencyProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: TimeTransparency,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TimeZoneIdProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub unique_registry_id: bool,
    pub value: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TimeZoneNameProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TimeZoneOffsetProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: UtcOffset,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TimeZoneUrlProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: &'a [u8],
}

#[derive(Debug, Eq, PartialEq)]
pub struct AttendeeProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: &'a [u8],
}

#[derive(Debug, Eq, PartialEq)]
pub struct ContactProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizerProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: &'a [u8],
}

#[derive(Debug, Eq, PartialEq)]
pub struct RecurrenceIdProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: DateOrDateTime,
}

#[derive(Debug, Eq, PartialEq)]
pub struct RelatedToProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UrlProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: Uri<'a>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UniqueIdentifierProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ExceptionDateTimesProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: Vec<DateOrDateTime>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct RecurrenceDateTimesProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: Vec<DateOrDateTimeOrPeriod>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct RecurrenceRuleProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: Vec<RecurRulePart>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Action<'a> {
    Audio,
    Display,
    Email,
    XName(&'a [u8]),
    IanaToken(&'a [u8]),
}

#[derive(Debug, Eq, PartialEq)]
pub struct ActionProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: Action<'a>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct RepeatProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub enum DurationOrDateTime {
    Duration(Duration),
    DateTime(DateTime),
}

#[derive(Debug, Eq, PartialEq)]
pub struct TriggerProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: DurationOrDateTime,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CreatedProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: DateTime,
}

#[derive(Debug, Eq, PartialEq)]
pub struct DateTimeStampProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: DateTime,
}

#[derive(Debug, Eq, PartialEq)]
pub struct LastModifiedProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: DateTime,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SequenceProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct RequestStatusProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub status_code: Vec<u32>,
    pub status_description: Vec<u8>,
    pub exception_data: Option<Vec<u8>>,
}
