use crate::parser::property::types::{
    CalendarScaleProperty, IanaProperty, MethodProperty, ProductId, VersionProperty, XProperty,
};
use crate::parser::property::{
    AttachProperty, AttendeeProperty, CategoriesProperty, ClassificationProperty, CommentProperty,
    ContactProperty, CreatedProperty, DateTimeCompletedProperty, DateTimeDueProperty,
    DateTimeEndProperty, DateTimeStampProperty, DateTimeStartProperty, DescriptionProperty,
    DurationProperty, ExceptionDateTimesProperty, FreeBusyTimeProperty, GeographicPositionProperty,
    LastModifiedProperty, LocationProperty, OrganizerProperty, PercentCompleteProperty,
    PriorityProperty, RecurrenceDateTimesProperty, RecurrenceIdProperty, RecurrenceRuleProperty,
    RelatedToProperty, RequestStatusProperty, ResourcesProperty, SequenceProperty, StatusProperty,
    SummaryProperty, TimeTransparencyProperty, UniqueIdentifierProperty, UrlProperty,
};
use crate::parser::ContentLine;

#[derive(Debug)]
pub struct ICalendar<'a> {
    pub properties: Vec<CalendarProperty<'a>>,
    pub components: Vec<CalendarComponent<'a>>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CalendarProperty<'a> {
    ProductId(ProductId<'a>),
    Version(VersionProperty<'a>),
    CalScale(CalendarScaleProperty<'a>),
    Method(MethodProperty<'a>),
    XProp(XProperty<'a>),
    IanaProp(IanaProperty<'a>),
}

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
    XProp(XProperty<'a>),
    IanaProp(IanaProperty<'a>),
}

#[derive(Debug)]
pub enum CalendarComponent<'a> {
    Event {
        properties: Vec<ComponentProperty<'a>>,
    },
    IanaComp {
        name: &'a [u8],
        lines: Vec<ContentLine<'a>>,
    },
    XComp {
        name: &'a [u8],
        lines: Vec<ContentLine<'a>>,
    },
}
