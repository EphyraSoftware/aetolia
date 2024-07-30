pub mod duration;
pub mod recur;

use crate::model::object::ICalObjectBuilder;
use crate::model::param::Param;
use crate::model::param::{impl_other_component_params_builder, impl_other_params_builder};
use crate::model::{
    add_is_utc, altrep_param, common_name_param, directory_entry_reference_param, language_param,
    sent_by_param, tz_id_param,
};
use std::fmt::Display;
use std::marker::PhantomData;

use crate::common::{
    CalendarDateTime, CalendarUserType, Encoding, FreeBusyTimeType, ParticipationStatusUnknown,
    Range, Related, RelationshipType, Role, Status, TimeTransparency, Value,
};
pub use duration::*;
pub use recur::*;

pub trait AddComponentProperty {
    fn add_property(&mut self, property: ComponentProperty);
}

#[derive(Debug, Eq, PartialEq)]
pub enum Classification {
    Public,
    Private,
    Confidential,
    XName(String),
    IanaToken(String),
}

impl Display for Classification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Classification::Public => "PUBLIC".to_string(),
            Classification::Private => "PRIVATE".to_string(),
            Classification::Confidential => "CONFIDENTIAL".to_string(),
            Classification::XName(name) => name.to_string(),
            Classification::IanaToken(token) => token.to_string(),
        };
        write!(f, "{}", str)
    }
}

pub enum StatusEvent {
    Tentative,
    Confirmed,
    Cancelled,
}

impl From<StatusEvent> for Status {
    fn from(status: StatusEvent) -> Self {
        match status {
            StatusEvent::Tentative => Status::Tentative,
            StatusEvent::Confirmed => Status::Confirmed,
            StatusEvent::Cancelled => Status::Cancelled,
        }
    }
}

pub enum StatusToDo {
    NeedsAction,
    Completed,
    InProcess,
    Cancelled,
}

impl From<StatusToDo> for Status {
    fn from(status: StatusToDo) -> Self {
        match status {
            StatusToDo::NeedsAction => Status::NeedsAction,
            StatusToDo::Completed => Status::Completed,
            StatusToDo::InProcess => Status::InProcess,
            StatusToDo::Cancelled => Status::Cancelled,
        }
    }
}

pub enum StatusJournal {
    Draft,
    Final,
    Cancelled,
}

impl From<StatusJournal> for Status {
    fn from(status: StatusJournal) -> Self {
        match status {
            StatusJournal::Draft => Status::Draft,
            StatusJournal::Final => Status::Final,
            StatusJournal::Cancelled => Status::Cancelled,
        }
    }
}

macro_rules! impl_finish_property_build {
    ($ev:expr) => {
        pub fn finish_property(mut self) -> ICalObjectBuilder {
            self.owner.inner.properties.push($ev(self.inner));
            self.owner
        }
    };
}

macro_rules! impl_finish_component_property_build {
    ($ev:expr) => {
        pub fn finish_property(mut self) -> P {
            self.owner.add_property($ev(self.inner));
            self.owner
        }
    };
}

pub enum CalendarProperty {
    ProductId(ProductIdProperty),
    Version(VersionProperty),
    CalendarScale(CalendarScaleProperty),
    Method(MethodProperty),
    XProperty(XProperty),
    IanaProperty(IanaProperty),
}

pub struct ProductIdProperty {
    pub(crate) value: String,
    pub(crate) params: Vec<Param>,
}

pub struct ProductIdPropertyBuilder {
    owner: ICalObjectBuilder,
    inner: ProductIdProperty,
}

impl ProductIdPropertyBuilder {
    pub(crate) fn new(owner: ICalObjectBuilder, value: String) -> ProductIdPropertyBuilder {
        ProductIdPropertyBuilder {
            owner,
            inner: ProductIdProperty {
                value,
                params: Vec::new(),
            },
        }
    }

    impl_finish_property_build!(CalendarProperty::ProductId);
}

impl_other_params_builder!(ProductIdPropertyBuilder);

pub struct VersionProperty {
    pub(crate) min_version: Option<String>,
    pub(crate) max_version: String,
    pub(crate) params: Vec<Param>,
}

pub struct VersionPropertyBuilder {
    owner: ICalObjectBuilder,
    inner: VersionProperty,
}

impl VersionPropertyBuilder {
    pub(crate) fn new(
        owner: ICalObjectBuilder,
        min_version: Option<String>,
        max_version: String,
    ) -> VersionPropertyBuilder {
        VersionPropertyBuilder {
            owner,
            inner: VersionProperty {
                min_version,
                max_version,
                params: Vec::new(),
            },
        }
    }

    impl_finish_property_build!(CalendarProperty::Version);
}

impl_other_params_builder!(VersionPropertyBuilder);

pub struct CalendarScaleProperty {
    pub(crate) value: String,
    pub(crate) params: Vec<Param>,
}

pub struct CalendarScalePropertyBuilder {
    owner: ICalObjectBuilder,
    inner: CalendarScaleProperty,
}

impl CalendarScalePropertyBuilder {
    pub(crate) fn new(owner: ICalObjectBuilder, value: String) -> CalendarScalePropertyBuilder {
        CalendarScalePropertyBuilder {
            owner,
            inner: CalendarScaleProperty {
                value,
                params: Vec::new(),
            },
        }
    }

    impl_finish_property_build!(CalendarProperty::CalendarScale);
}

impl_other_params_builder!(CalendarScalePropertyBuilder);

pub struct MethodProperty {
    pub(crate) value: String,
    pub(crate) params: Vec<Param>,
}

pub struct MethodPropertyBuilder {
    owner: ICalObjectBuilder,
    inner: MethodProperty,
}

impl MethodPropertyBuilder {
    pub(crate) fn new(owner: ICalObjectBuilder, value: String) -> MethodPropertyBuilder {
        MethodPropertyBuilder {
            owner,
            inner: MethodProperty {
                params: Vec::new(),
                value,
            },
        }
    }

    impl_finish_property_build!(CalendarProperty::Method);
}

impl_other_params_builder!(MethodPropertyBuilder);

#[derive(Debug)]
pub enum ComponentProperty {
    DateTimeStamp(DateTimeStampProperty),
    UniqueIdentifier(UniqueIdentifierProperty),
    DateTimeStart(DateTimeStartProperty),
    Classification(ClassificationProperty),
    DateTimeCreated(CreatedProperty),
    Description(DescriptionProperty),
    GeographicPosition(GeographicPositionProperty),
    LastModified(LastModifiedProperty),
    Location(LocationProperty),
    Organizer(OrganizerProperty),
    Priority(PriorityProperty),
    Sequence(SequenceProperty),
    Summary(SummaryProperty),
    TimeTransparency(TimeTransparencyProperty),
    RequestStatus(RequestStatusProperty),
    Url(UrlProperty),
    RecurrenceId(RecurrenceIdProperty),
    RecurrenceRule(RecurrenceRuleProperty),
    DateTimeEnd(DateTimeEndProperty),
    Duration(DurationProperty),
    Attach(AttachProperty),
    Attendee(AttendeeProperty),
    Categories(CategoriesProperty),
    Comment(CommentProperty),
    Contact(ContactProperty),
    ExceptionDateTimes(ExceptionDateTimesProperty),
    Status(StatusProperty),
    RelatedTo(RelatedToProperty),
    Resources(ResourcesProperty),
    RecurrenceDateTimes(RecurrenceDateTimesProperty),
    DateTimeCompleted(DateTimeCompletedProperty),
    PercentComplete(PercentCompleteProperty),
    DateTimeDue(DateTimeDueProperty),
    FreeBusyTime(FreeBusyTimeProperty),
    TimeZoneId(TimeZoneIdProperty),
    TimeZoneUrl(TimeZoneUrlProperty),
    TimeZoneOffsetTo(TimeZoneOffsetToProperty),
    TimeZoneOffsetFrom(TimeZoneOffsetFromProperty),
    TimeZoneName(TimeZoneNameProperty),
    Action(ActionProperty),
    Trigger(Trigger),
    Repeat(RepeatProperty),
    IanaProperty(IanaProperty),
    XProperty(XProperty),
}

impl ComponentProperty {
    pub fn params(&self) -> &[Param] {
        match self {
            ComponentProperty::DateTimeStamp(p) => &p.params,
            ComponentProperty::UniqueIdentifier(p) => &p.params,
            ComponentProperty::DateTimeStart(p) => &p.params,
            ComponentProperty::Classification(p) => &p.params,
            ComponentProperty::DateTimeCreated(p) => &p.params,
            ComponentProperty::Description(p) => &p.params,
            ComponentProperty::GeographicPosition(p) => &p.params,
            ComponentProperty::LastModified(p) => &p.params,
            ComponentProperty::Location(p) => &p.params,
            ComponentProperty::Organizer(p) => &p.params,
            ComponentProperty::Priority(p) => &p.params,
            ComponentProperty::Sequence(p) => &p.params,
            ComponentProperty::Summary(p) => &p.params,
            ComponentProperty::TimeTransparency(p) => &p.params,
            ComponentProperty::RequestStatus(p) => &p.params,
            ComponentProperty::Url(p) => &p.params,
            ComponentProperty::RecurrenceId(p) => &p.params,
            ComponentProperty::RecurrenceRule(p) => &p.params,
            ComponentProperty::DateTimeEnd(p) => &p.params,
            ComponentProperty::Duration(p) => &p.params,
            ComponentProperty::Attach(p) => &p.params,
            ComponentProperty::Attendee(p) => &p.params,
            ComponentProperty::Categories(p) => &p.params,
            ComponentProperty::Comment(p) => &p.params,
            ComponentProperty::Contact(p) => &p.params,
            ComponentProperty::ExceptionDateTimes(p) => &p.params,
            ComponentProperty::Status(p) => &p.params,
            ComponentProperty::RelatedTo(p) => &p.params,
            ComponentProperty::Resources(p) => &p.params,
            ComponentProperty::RecurrenceDateTimes(p) => &p.params,
            ComponentProperty::DateTimeCompleted(p) => &p.params,
            ComponentProperty::PercentComplete(p) => &p.params,
            ComponentProperty::DateTimeDue(p) => &p.params,
            ComponentProperty::FreeBusyTime(p) => &p.params,
            ComponentProperty::TimeZoneId(p) => &p.params,
            ComponentProperty::TimeZoneUrl(p) => &p.params,
            ComponentProperty::TimeZoneOffsetTo(p) => &p.params,
            ComponentProperty::TimeZoneOffsetFrom(p) => &p.params,
            ComponentProperty::TimeZoneName(p) => &p.params,
            ComponentProperty::Action(p) => &p.params,
            ComponentProperty::Trigger(p) => match p {
                Trigger::Relative(p) => &p.params,
                Trigger::Absolute(p) => &p.params,
            },
            ComponentProperty::Repeat(p) => &p.params,
            ComponentProperty::IanaProperty(p) => &p.params,
            ComponentProperty::XProperty(p) => &p.params,
        }
    }
}

#[derive(Debug)]
pub enum Trigger {
    Relative(RelativeTriggerProperty),
    Absolute(AbsoluteTriggerProperty),
}

impl Trigger {
    pub fn params(&self) -> &[Param] {
        match self {
            Trigger::Relative(p) => &p.params,
            Trigger::Absolute(p) => &p.params,
        }
    }
}

#[derive(Debug)]
pub struct XProperty {
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) params: Vec<Param>,
}

pub struct XPropertyBuilder {
    owner: ICalObjectBuilder,
    inner: XProperty,
}

impl XPropertyBuilder {
    pub(crate) fn new(owner: ICalObjectBuilder, name: String, value: String) -> XPropertyBuilder {
        XPropertyBuilder {
            owner,
            inner: XProperty {
                name,
                value,
                params: Vec::new(),
            },
        }
    }

    impl_finish_property_build!(CalendarProperty::XProperty);
}

impl_other_params_builder!(XPropertyBuilder);

#[derive(Debug)]
pub struct IanaProperty {
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) params: Vec<Param>,
}

pub struct IanaPropertyBuilder {
    owner: ICalObjectBuilder,
    inner: IanaProperty,
}

impl IanaPropertyBuilder {
    pub(crate) fn new(
        owner: ICalObjectBuilder,
        name: String,
        value: String,
    ) -> IanaPropertyBuilder {
        IanaPropertyBuilder {
            owner,
            inner: IanaProperty {
                name,
                value,
                params: Vec::new(),
            },
        }
    }

    impl_finish_property_build!(CalendarProperty::IanaProperty);
}

impl_other_params_builder!(IanaPropertyBuilder);

pub struct XComponentPropertyBuilder<P> {
    owner: P,
    inner: XProperty,
}

impl<P> XComponentPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, name: String, value: String) -> XComponentPropertyBuilder<P> {
        XComponentPropertyBuilder {
            owner,
            inner: XProperty {
                params: Vec::new(),
                name,
                value,
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::XProperty);
}

impl_other_component_params_builder!(XComponentPropertyBuilder<P>);

pub struct IanaComponentPropertyBuilder<P> {
    owner: P,
    inner: IanaProperty,
}

impl<P> IanaComponentPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, name: String, value: String) -> IanaComponentPropertyBuilder<P> {
        IanaComponentPropertyBuilder {
            owner,
            inner: IanaProperty {
                name,
                value,
                params: Vec::new(),
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::IanaProperty);
}

impl_other_component_params_builder!(IanaComponentPropertyBuilder<P>);

#[derive(Debug)]
pub struct DateTimeStampProperty {
    pub(crate) date: time::Date,
    pub(crate) time: time::Time,
    pub(crate) is_utc: bool,
    pub(crate) params: Vec<Param>,
}

pub struct DateTimeStampPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: DateTimeStampProperty,
}

impl<P> DateTimeStampPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(
        owner: P,
        date: time::Date,
        time: time::Time,
    ) -> DateTimeStampPropertyBuilder<P> {
        DateTimeStampPropertyBuilder {
            owner,
            inner: DateTimeStampProperty {
                date,
                time,
                is_utc: false,
                params: Vec::new(),
            },
        }
    }

    add_is_utc!();

    impl_finish_component_property_build!(ComponentProperty::DateTimeStamp);
}

impl_other_component_params_builder!(DateTimeStampPropertyBuilder<P>);

#[derive(Debug)]
pub struct UniqueIdentifierProperty {
    pub(crate) value: String,
    pub(crate) params: Vec<Param>,
}

pub struct UniqueIdentifierPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: UniqueIdentifierProperty,
}

impl<P> UniqueIdentifierPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, value: String) -> UniqueIdentifierPropertyBuilder<P> {
        UniqueIdentifierPropertyBuilder {
            owner,
            inner: UniqueIdentifierProperty {
                value,
                params: Vec::new(),
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::UniqueIdentifier);
}

impl_other_component_params_builder!(UniqueIdentifierPropertyBuilder<P>);

#[derive(Debug, Clone)]
pub struct DateTimeStartProperty {
    pub(crate) date: time::Date,
    pub(crate) time: Option<time::Time>,
    pub(crate) is_utc: bool,
    pub(crate) params: Vec<Param>,
}

pub struct DateTimeStartPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: DateTimeStartProperty,
}

impl<P> DateTimeStartPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(
        owner: P,
        date: time::Date,
        time: Option<time::Time>,
    ) -> DateTimeStartPropertyBuilder<P> {
        let mut params = Vec::new();

        // The default is DATE-TIME. If the time is None, then it is a DATE and although it's
        // optional, this will default to setting the value here.
        if time.is_none() {
            params.push(Param::ValueType { value: Value::Date })
        }

        DateTimeStartPropertyBuilder {
            owner,
            inner: DateTimeStartProperty {
                date,
                time,
                is_utc: false,
                params,
            },
        }
    }

    tz_id_param!();

    add_is_utc!();

    impl_finish_component_property_build!(ComponentProperty::DateTimeStart);
}

impl_other_component_params_builder!(DateTimeStartPropertyBuilder<P>);

#[derive(Debug)]
pub struct ClassificationProperty {
    pub(crate) value: Classification,
    pub(crate) params: Vec<Param>,
}

pub struct ClassificationPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: ClassificationProperty,
}

impl<P> ClassificationPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, value: Classification) -> ClassificationPropertyBuilder<P> {
        ClassificationPropertyBuilder {
            owner,
            inner: ClassificationProperty {
                value,
                params: Vec::new(),
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::Classification);
}

impl_other_component_params_builder!(ClassificationPropertyBuilder<P>);

#[derive(Debug)]
pub struct CreatedProperty {
    pub(crate) date: time::Date,
    pub(crate) time: time::Time,
    pub(crate) is_utc: bool,
    pub(crate) params: Vec<Param>,
}

pub struct CreatedPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: CreatedProperty,
}

impl<P> CreatedPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, date: time::Date, time: time::Time) -> CreatedPropertyBuilder<P> {
        CreatedPropertyBuilder {
            owner,
            inner: CreatedProperty {
                date,
                time,
                is_utc: false,
                params: Vec::new(),
            },
        }
    }

    add_is_utc!();

    impl_finish_component_property_build!(ComponentProperty::DateTimeCreated);
}

impl_other_component_params_builder!(CreatedPropertyBuilder<P>);

#[derive(Debug)]
pub struct DescriptionProperty {
    pub(crate) value: String,
    pub(crate) params: Vec<Param>,
}

pub struct DescriptionPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: DescriptionProperty,
}

impl<P> DescriptionPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, value: String) -> DescriptionPropertyBuilder<P> {
        DescriptionPropertyBuilder {
            owner,
            inner: DescriptionProperty {
                value,
                params: Vec::new(),
            },
        }
    }

    altrep_param!();
    language_param!();

    impl_finish_component_property_build!(ComponentProperty::Description);
}

impl_other_component_params_builder!(DescriptionPropertyBuilder<P>);

#[derive(Debug)]
pub struct GeographicPositionProperty {
    pub(crate) latitude: f64,
    pub(crate) longitude: f64,
    pub(crate) params: Vec<Param>,
}

pub struct GeographicPositionPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: GeographicPositionProperty,
}

impl<P> GeographicPositionPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(
        owner: P,
        latitude: f64,
        longitude: f64,
    ) -> GeographicPositionPropertyBuilder<P> {
        GeographicPositionPropertyBuilder {
            owner,
            inner: GeographicPositionProperty {
                latitude,
                longitude,
                params: Vec::new(),
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::GeographicPosition);
}

impl_other_component_params_builder!(GeographicPositionPropertyBuilder<P>);

#[derive(Debug)]
pub struct LastModifiedProperty {
    pub(crate) date: time::Date,
    pub(crate) time: time::Time,
    pub(crate) is_utc: bool,
    pub(crate) params: Vec<Param>,
}

pub struct LastModifiedPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: LastModifiedProperty,
}

impl<P> LastModifiedPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(
        owner: P,
        date: time::Date,
        time: time::Time,
    ) -> LastModifiedPropertyBuilder<P> {
        LastModifiedPropertyBuilder {
            owner,
            inner: LastModifiedProperty {
                date,
                time,
                is_utc: false,
                params: Vec::new(),
            },
        }
    }

    add_is_utc!();

    impl_finish_component_property_build!(ComponentProperty::LastModified);
}

impl_other_component_params_builder!(LastModifiedPropertyBuilder<P>);

#[derive(Debug)]
pub struct LocationProperty {
    pub(crate) value: String,
    pub(crate) params: Vec<Param>,
}

pub struct LocationPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: LocationProperty,
}

impl<P> LocationPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, value: String) -> LocationPropertyBuilder<P> {
        LocationPropertyBuilder {
            owner,
            inner: LocationProperty {
                value,
                params: Vec::new(),
            },
        }
    }

    altrep_param!();
    language_param!();

    impl_finish_component_property_build!(ComponentProperty::Location);
}

impl_other_component_params_builder!(LocationPropertyBuilder<P>);

#[derive(Debug)]
pub struct OrganizerProperty {
    pub(crate) value: String,
    pub(crate) params: Vec<Param>,
}

pub struct OrganizerPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: OrganizerProperty,
}

impl<P> OrganizerPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, value: String) -> OrganizerPropertyBuilder<P> {
        OrganizerPropertyBuilder {
            owner,
            inner: OrganizerProperty {
                value,
                params: Vec::new(),
            },
        }
    }

    common_name_param!();

    directory_entry_reference_param!();

    sent_by_param!();

    language_param!();

    impl_finish_component_property_build!(ComponentProperty::Organizer);
}

impl_other_component_params_builder!(OrganizerPropertyBuilder<P>);

#[derive(Debug)]
pub struct PriorityProperty {
    pub(crate) value: u8,
    pub(crate) params: Vec<Param>,
}

pub struct PriorityPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: PriorityProperty,
}

impl<P> PriorityPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, value: u8) -> PriorityPropertyBuilder<P> {
        PriorityPropertyBuilder {
            owner,
            inner: PriorityProperty {
                value,
                params: Vec::new(),
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::Priority);
}

impl_other_component_params_builder!(PriorityPropertyBuilder<P>);

#[derive(Debug)]
pub struct SequenceProperty {
    pub(crate) value: u32,
    pub(crate) params: Vec<Param>,
}

pub struct SequencePropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: SequenceProperty,
}

impl<P> SequencePropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, value: u32) -> SequencePropertyBuilder<P> {
        SequencePropertyBuilder {
            owner,
            inner: SequenceProperty {
                value,
                params: Vec::new(),
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::Sequence);
}

impl_other_component_params_builder!(SequencePropertyBuilder<P>);

#[derive(Debug)]
pub struct RequestStatusProperty {
    pub(crate) status_code: Vec<u32>,
    pub(crate) description: String,
    pub(crate) exception_data: Option<String>,
    pub(crate) params: Vec<Param>,
}

pub struct RequestStatusPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: RequestStatusProperty,
}

impl<P> RequestStatusPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(
        owner: P,
        status_code: Vec<u32>,
        description: String,
        exception_data: Option<String>,
    ) -> RequestStatusPropertyBuilder<P> {
        RequestStatusPropertyBuilder {
            owner,
            inner: RequestStatusProperty {
                status_code,
                description,
                exception_data,
                params: Vec::new(),
            },
        }
    }

    language_param!();

    impl_finish_component_property_build!(ComponentProperty::RequestStatus);
}

impl_other_component_params_builder!(RequestStatusPropertyBuilder<P>);

#[derive(Debug)]
pub struct SummaryProperty {
    pub(crate) value: String,
    pub(crate) params: Vec<Param>,
}

pub struct SummaryPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: SummaryProperty,
}

impl<P> SummaryPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, value: String) -> SummaryPropertyBuilder<P> {
        SummaryPropertyBuilder {
            owner,
            inner: SummaryProperty {
                value,
                params: Vec::new(),
            },
        }
    }

    altrep_param!();
    language_param!();

    impl_finish_component_property_build!(ComponentProperty::Summary);
}

impl_other_component_params_builder!(SummaryPropertyBuilder<P>);

#[derive(Debug)]
pub struct TimeTransparencyProperty {
    pub(crate) value: TimeTransparency,
    pub(crate) params: Vec<Param>,
}

pub struct TimeTransparencyPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: TimeTransparencyProperty,
}

impl<P> TimeTransparencyPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, value: TimeTransparency) -> TimeTransparencyPropertyBuilder<P> {
        TimeTransparencyPropertyBuilder {
            owner,
            inner: TimeTransparencyProperty {
                value,
                params: Vec::new(),
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::TimeTransparency);
}

impl_other_component_params_builder!(TimeTransparencyPropertyBuilder<P>);

#[derive(Debug)]
pub struct UrlProperty {
    // TODO should be a URI
    pub(crate) value: String,
    pub(crate) params: Vec<Param>,
}

pub struct UrlPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: UrlProperty,
}

impl<P> UrlPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, value: String) -> UrlPropertyBuilder<P> {
        UrlPropertyBuilder {
            owner,
            inner: UrlProperty {
                value,
                params: Vec::new(),
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::Url);
}

impl_other_component_params_builder!(UrlPropertyBuilder<P>);

#[derive(Debug)]
pub struct RecurrenceIdProperty {
    pub(crate) date: time::Date,
    pub(crate) time: Option<time::Time>,
    pub(crate) is_utc: bool,
    pub(crate) params: Vec<Param>,
}

pub struct RecurrenceIdPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: RecurrenceIdProperty,
}

impl<P> RecurrenceIdPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(
        owner: P,
        date: time::Date,
        time: Option<time::Time>,
    ) -> RecurrenceIdPropertyBuilder<P> {
        let mut params = Vec::new();

        // The default is DATE-TIME. If the time is None, then it is a DATE and although it's
        // optional, this will default to setting the value here.
        if time.is_none() {
            params.push(Param::ValueType { value: Value::Date })
        }

        RecurrenceIdPropertyBuilder {
            owner,
            inner: RecurrenceIdProperty {
                date,
                time,
                is_utc: false,
                params,
            },
        }
    }

    tz_id_param!();

    add_is_utc!();

    pub fn add_range(mut self, range: Range) -> Self {
        self.inner.params.push(Param::Range { range });
        self
    }

    impl_finish_component_property_build!(ComponentProperty::RecurrenceId);
}

impl_other_component_params_builder!(RecurrenceIdPropertyBuilder<P>);

#[derive(Debug)]
pub struct RecurrenceRuleProperty {
    pub(crate) rule: RecurrenceRule,
    pub(crate) params: Vec<Param>,
}

pub struct RecurrenceRulePropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: RecurrenceRuleProperty,
}

impl<P> RecurrenceRulePropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, rule: RecurrenceRule) -> RecurrenceRulePropertyBuilder<P> {
        RecurrenceRulePropertyBuilder {
            owner,
            inner: RecurrenceRuleProperty {
                rule,
                params: Vec::new(),
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::RecurrenceRule);
}

impl_other_component_params_builder!(RecurrenceRulePropertyBuilder<P>);

#[derive(Debug, Clone)]
pub struct DateTimeEndProperty {
    pub(crate) date: time::Date,
    pub(crate) time: Option<time::Time>,
    pub(crate) is_utc: bool,
    pub(crate) params: Vec<Param>,
}

pub struct DateTimeEndPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: DateTimeEndProperty,
}

impl<P> DateTimeEndPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(
        owner: P,
        date: time::Date,
        time: Option<time::Time>,
    ) -> DateTimeEndPropertyBuilder<P> {
        let mut params = Vec::new();

        // The default is DATE-TIME. If the time is None, then it is a DATE and although it's
        // optional, this will default to setting the value here.
        if time.is_none() {
            params.push(Param::ValueType { value: Value::Date })
        }

        DateTimeEndPropertyBuilder {
            owner,
            inner: DateTimeEndProperty {
                date,
                time,
                is_utc: false,
                params,
            },
        }
    }

    tz_id_param!();

    impl_finish_component_property_build!(ComponentProperty::DateTimeEnd);
}

impl_other_component_params_builder!(DateTimeEndPropertyBuilder<P>);

#[derive(Debug)]
pub struct DurationProperty {
    pub(crate) duration: Duration,
    pub(crate) params: Vec<Param>,
}

pub struct DurationPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: DurationProperty,
}

impl<P> DurationPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, duration: Duration) -> DurationPropertyBuilder<P> {
        DurationPropertyBuilder {
            owner,
            inner: DurationProperty {
                duration,
                params: Vec::new(),
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::Duration);
}

impl_other_component_params_builder!(DurationPropertyBuilder<P>);

#[derive(Debug)]
pub struct AttachProperty {
    pub(crate) value: String,
    pub(crate) params: Vec<Param>,
}

pub struct AttachPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: AttachProperty,
}

impl<P> AttachPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new_with_uri(owner: P, uri: String) -> AttachPropertyBuilder<P> {
        AttachPropertyBuilder {
            owner,
            inner: AttachProperty {
                value: uri,
                params: Vec::new(),
            },
        }
    }

    pub(crate) fn new_with_binary(owner: P, binary: String) -> AttachPropertyBuilder<P> {
        AttachPropertyBuilder {
            owner,
            inner: AttachProperty {
                value: binary,
                params: vec![
                    Param::Encoding {
                        encoding: Encoding::Base64,
                    },
                    Param::ValueType {
                        value: Value::Binary,
                    },
                ],
            },
        }
    }

    pub fn add_fmt_type<U: ToString, V: ToString>(
        mut self,
        type_name: U,
        sub_type_name: V,
    ) -> Self {
        self.inner.params.push(Param::FormatType {
            type_name: type_name.to_string(),
            sub_type_name: sub_type_name.to_string(),
        });
        self
    }

    impl_finish_component_property_build!(ComponentProperty::Attach);
}

impl_other_component_params_builder!(AttachPropertyBuilder<P>);

#[derive(Debug)]
pub struct AttendeeProperty {
    pub(crate) value: String,
    pub(crate) params: Vec<Param>,
}

pub struct AttendeePropertyBuilder<P: AddComponentProperty, PS> {
    owner: P,
    inner: AttendeeProperty,
    _phantom: PhantomData<PS>,
}

impl<P, PS> AttendeePropertyBuilder<P, PS>
where
    P: AddComponentProperty,
    PS: Into<ParticipationStatusUnknown>,
{
    pub(crate) fn new(owner: P, value: String) -> AttendeePropertyBuilder<P, PS> {
        AttendeePropertyBuilder {
            owner,
            inner: AttendeeProperty {
                value,
                params: Vec::new(),
            },
            _phantom: PhantomData,
        }
    }

    pub fn add_calendar_user_type(mut self, cu_type: CalendarUserType) -> Self {
        self.inner.params.push(Param::CalendarUserType { cu_type });
        self
    }

    pub fn add_members(mut self, members: Vec<String>) -> Self {
        self.inner.params.push(Param::Members { members });
        self
    }

    pub fn add_role(mut self, role: Role) -> Self {
        self.inner.params.push(Param::Role { role });
        self
    }

    pub fn add_participation_status(mut self, status: PS) -> Self {
        self.inner.params.push(Param::ParticipationStatus {
            status: status.into(),
        });
        self
    }

    pub fn add_rsvp(mut self) -> Self {
        // Default is false, add to set true
        self.inner.params.push(Param::Rsvp { rsvp: true });
        self
    }

    pub fn add_delegated_to(mut self, delegates: Vec<String>) -> Self {
        self.inner.params.push(Param::DelegatedTo { delegates });
        self
    }

    pub fn add_delegated_from(mut self, delegators: Vec<String>) -> Self {
        self.inner.params.push(Param::DelegatedFrom { delegators });
        self
    }

    sent_by_param!();
    common_name_param!();
    directory_entry_reference_param!();
    language_param!();

    impl_finish_component_property_build!(ComponentProperty::Attendee);
}

impl_other_component_params_builder!(AttendeePropertyBuilder<P, PS>);

#[derive(Debug)]
pub struct CategoriesProperty {
    pub(crate) value: Vec<String>,
    pub(crate) params: Vec<Param>,
}

pub struct CategoriesPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: CategoriesProperty,
}

impl<P> CategoriesPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, value: Vec<String>) -> CategoriesPropertyBuilder<P> {
        CategoriesPropertyBuilder {
            owner,
            inner: CategoriesProperty {
                value,
                params: Vec::new(),
            },
        }
    }

    language_param!();

    impl_finish_component_property_build!(ComponentProperty::Categories);
}

impl_other_component_params_builder!(CategoriesPropertyBuilder<P>);

#[derive(Debug)]
pub struct CommentProperty {
    pub(crate) value: String,
    pub(crate) params: Vec<Param>,
}

pub struct CommentPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: CommentProperty,
}

impl<P> CommentPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, value: String) -> CommentPropertyBuilder<P> {
        CommentPropertyBuilder {
            owner,
            inner: CommentProperty {
                value,
                params: Vec::new(),
            },
        }
    }

    altrep_param!();
    language_param!();

    impl_finish_component_property_build!(ComponentProperty::Comment);
}

impl_other_component_params_builder!(CommentPropertyBuilder<P>);

#[derive(Debug)]
pub struct ContactProperty {
    pub(crate) value: String,
    pub(crate) params: Vec<Param>,
}

pub struct ContactPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: ContactProperty,
}

impl<P> ContactPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, value: String) -> ContactPropertyBuilder<P> {
        ContactPropertyBuilder {
            owner,
            inner: ContactProperty {
                value,
                params: Vec::new(),
            },
        }
    }

    altrep_param!();
    language_param!();

    impl_finish_component_property_build!(ComponentProperty::Contact);
}

impl_other_component_params_builder!(ContactPropertyBuilder<P>);

#[derive(Debug)]
pub struct ExceptionDateTimesProperty {
    pub(crate) date_times: Vec<(time::Date, Option<time::Time>, bool)>,
    pub(crate) params: Vec<Param>,
}

pub struct ExceptionDateTimesPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: ExceptionDateTimesProperty,
}

impl<P> ExceptionDateTimesPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, date_times: Vec<(time::Date, Option<time::Time>, bool)>) -> Self {
        let mut params = Vec::new();
        if let Some((_, None, _)) = date_times.first() {
            params.push(Param::ValueType { value: Value::Date });
        }

        ExceptionDateTimesPropertyBuilder {
            owner,
            inner: ExceptionDateTimesProperty { date_times, params },
        }
    }

    tz_id_param!();

    impl_finish_component_property_build!(ComponentProperty::ExceptionDateTimes);
}

impl_other_component_params_builder!(ExceptionDateTimesPropertyBuilder<P>);

#[derive(Debug)]
pub struct StatusProperty {
    pub(crate) value: Status,
    pub(crate) params: Vec<Param>,
}

pub struct StatusPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: StatusProperty,
}

impl<P> StatusPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, value: Status) -> StatusPropertyBuilder<P> {
        StatusPropertyBuilder {
            owner,
            inner: StatusProperty {
                value,
                params: Vec::new(),
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::Status);
}

impl_other_component_params_builder!(StatusPropertyBuilder<P>);

#[derive(Debug)]
pub struct RelatedToProperty {
    pub(crate) value: String,
    pub(crate) params: Vec<Param>,
}

pub struct RelatedToPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: RelatedToProperty,
}

impl<P> RelatedToPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, value: String) -> RelatedToPropertyBuilder<P> {
        RelatedToPropertyBuilder {
            owner,
            inner: RelatedToProperty {
                value,
                params: Vec::new(),
            },
        }
    }

    pub fn add_relationship_type(mut self, relationship_type: RelationshipType) -> Self {
        self.inner.params.push(Param::RelationshipType {
            relationship: relationship_type,
        });
        self
    }

    impl_finish_component_property_build!(ComponentProperty::RelatedTo);
}

impl_other_component_params_builder!(RelatedToPropertyBuilder<P>);

#[derive(Debug)]
pub struct ResourcesProperty {
    pub(crate) value: Vec<String>,
    pub(crate) params: Vec<Param>,
}

pub struct ResourcesPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: ResourcesProperty,
}

impl<P> ResourcesPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, value: Vec<String>) -> ResourcesPropertyBuilder<P> {
        ResourcesPropertyBuilder {
            owner,
            inner: ResourcesProperty {
                value,
                params: Vec::new(),
            },
        }
    }

    altrep_param!();
    language_param!();

    impl_finish_component_property_build!(ComponentProperty::Resources);
}

impl_other_component_params_builder!(ResourcesPropertyBuilder<P>);

#[derive(Clone, Debug)]
pub struct Period {
    pub start: (time::Date, time::Time, bool),
    pub end: PeriodEnd,
}

impl Period {
    pub fn new_explicit(
        start_date: time::Date,
        start_time: time::Time,
        end_date: time::Date,
        end_time: time::Time,
        is_utc: bool,
    ) -> Self {
        Period {
            start: (start_date, start_time, is_utc),
            end: PeriodEnd::DateTime((end_date, end_time, is_utc)),
        }
    }

    pub fn new_start(
        start_date: time::Date,
        start_time: time::Time,
        is_utc: bool,
        duration: Duration,
    ) -> Self {
        Period {
            start: (start_date, start_time, is_utc),
            end: PeriodEnd::Duration(duration),
        }
    }

    pub fn expand(&self) -> Option<(CalendarDateTime, CalendarDateTime)> {
        if self.start.2 {
            Some((
                self.start.into(),
                match &self.end {
                    PeriodEnd::DateTime(end) => (*end).into(),
                    PeriodEnd::Duration(duration) => {
                        let cdt: CalendarDateTime = self.start.into();
                        cdt.add(duration)
                    }
                },
            ))
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub enum PeriodEnd {
    DateTime((time::Date, time::Time, bool)),
    Duration(Duration),
}

#[derive(Debug)]
pub struct RecurrenceDateTimesProperty {
    pub(crate) date_times: Vec<(time::Date, Option<time::Time>, bool)>,
    pub(crate) periods: Vec<Period>,
    pub(crate) params: Vec<Param>,
}

pub struct RecurrenceDateTimesPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: RecurrenceDateTimesProperty,
}

impl<P> RecurrenceDateTimesPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub fn new_date_times(
        owner: P,
        date_times: Vec<(time::Date, Option<time::Time>, bool)>,
    ) -> Self {
        let mut params = Vec::new();
        if let Some((_, None, _)) = date_times.first() {
            params.push(Param::ValueType { value: Value::Date });
        }

        RecurrenceDateTimesPropertyBuilder {
            owner,
            inner: RecurrenceDateTimesProperty {
                date_times,
                periods: Vec::with_capacity(0),
                params,
            },
        }
    }

    pub fn new_periods(owner: P, periods: Vec<Period>) -> Self {
        RecurrenceDateTimesPropertyBuilder {
            owner,
            inner: RecurrenceDateTimesProperty {
                date_times: Vec::with_capacity(0),
                periods,
                params: vec![Param::ValueType {
                    value: Value::Period,
                }],
            },
        }
    }

    tz_id_param!();

    impl_finish_component_property_build!(ComponentProperty::RecurrenceDateTimes);
}

impl_other_component_params_builder!(RecurrenceDateTimesPropertyBuilder<P>);

#[derive(Debug)]
pub struct DateTimeCompletedProperty {
    pub(crate) date: time::Date,
    pub(crate) time: time::Time,
    pub(crate) is_utc: bool,
    pub(crate) params: Vec<Param>,
}

pub struct CompletedPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: DateTimeCompletedProperty,
}

impl<P> CompletedPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, date: time::Date, time: time::Time) -> CompletedPropertyBuilder<P> {
        CompletedPropertyBuilder {
            owner,
            inner: DateTimeCompletedProperty {
                date,
                time,
                is_utc: false,
                params: Vec::new(),
            },
        }
    }

    add_is_utc!();

    impl_finish_component_property_build!(ComponentProperty::DateTimeCompleted);
}

impl_other_component_params_builder!(CompletedPropertyBuilder<P>);

#[derive(Debug)]
pub struct PercentCompleteProperty {
    pub(crate) value: u8,
    pub(crate) params: Vec<Param>,
}

pub struct PercentCompletePropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: PercentCompleteProperty,
}

impl<P> PercentCompletePropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, value: u8) -> PercentCompletePropertyBuilder<P> {
        PercentCompletePropertyBuilder {
            owner,
            inner: PercentCompleteProperty {
                value,
                params: Vec::new(),
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::PercentComplete);
}

impl_other_component_params_builder!(PercentCompletePropertyBuilder<P>);

#[derive(Debug, Clone)]
pub struct DateTimeDueProperty {
    pub(crate) date: time::Date,
    pub(crate) time: Option<time::Time>,
    pub(crate) is_utc: bool,
    pub(crate) params: Vec<Param>,
}

pub struct DateTimeDuePropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: DateTimeDueProperty,
}

impl<P> DateTimeDuePropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(
        owner: P,
        date: time::Date,
        time: Option<time::Time>,
    ) -> DateTimeDuePropertyBuilder<P> {
        let mut params = Vec::new();

        // The default is DATE-TIME. If the time is None, then it is a DATE and although it's
        // optional, this will default to setting the value here.
        if time.is_none() {
            params.push(Param::ValueType { value: Value::Date })
        }

        DateTimeDuePropertyBuilder {
            owner,
            inner: DateTimeDueProperty {
                date,
                time,
                is_utc: false,
                params,
            },
        }
    }

    tz_id_param!();

    add_is_utc!();

    impl_finish_component_property_build!(ComponentProperty::DateTimeDue);
}

impl_other_component_params_builder!(DateTimeDuePropertyBuilder<P>);

#[derive(Debug)]
pub struct FreeBusyTimeProperty {
    pub(crate) value: Vec<Period>,
    pub(crate) params: Vec<Param>,
}

pub struct FreeBusyTimePropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: FreeBusyTimeProperty,
}

impl<P> FreeBusyTimePropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(
        owner: P,
        free_busy_time_type: FreeBusyTimeType,
        value: Vec<Period>,
    ) -> FreeBusyTimePropertyBuilder<P> {
        FreeBusyTimePropertyBuilder {
            owner,
            inner: FreeBusyTimeProperty {
                value,
                params: vec![Param::FreeBusyTimeType {
                    fb_type: free_busy_time_type,
                }],
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::FreeBusyTime);
}

impl_other_component_params_builder!(FreeBusyTimePropertyBuilder<P>);

#[derive(Debug)]
pub struct TimeZoneIdProperty {
    pub(crate) value: String,
    pub(crate) unique_registry_id: bool,
    pub(crate) params: Vec<Param>,
}

pub struct TimeZoneIdPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: TimeZoneIdProperty,
}

impl<P> TimeZoneIdPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(
        owner: P,
        value: String,
        unique_registry_id: bool,
    ) -> TimeZoneIdPropertyBuilder<P> {
        TimeZoneIdPropertyBuilder {
            owner,
            inner: TimeZoneIdProperty {
                value,
                unique_registry_id,
                params: Vec::new(),
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::TimeZoneId);
}

impl_other_component_params_builder!(TimeZoneIdPropertyBuilder<P>);

#[derive(Debug)]
pub struct TimeZoneUrlProperty {
    pub(crate) value: String,
    pub(crate) params: Vec<Param>,
}

pub struct TimeZoneUrlPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: TimeZoneUrlProperty,
}

impl<P> TimeZoneUrlPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, value: String) -> TimeZoneUrlPropertyBuilder<P> {
        TimeZoneUrlPropertyBuilder {
            owner,
            inner: TimeZoneUrlProperty {
                value,
                params: Vec::new(),
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::TimeZoneUrl);
}

impl_other_component_params_builder!(TimeZoneUrlPropertyBuilder<P>);

#[derive(Debug)]
pub struct TimeZoneOffset {
    pub(crate) sign: i8,
    pub(crate) hours: u8,
    pub(crate) minutes: u8,
    pub(crate) seconds: Option<u8>,
}

impl TimeZoneOffset {
    pub fn new(sign: i8, hours: u8, minutes: u8, seconds: Option<u8>) -> Self {
        TimeZoneOffset {
            sign,
            hours,
            minutes,
            seconds,
        }
    }
}

#[derive(Debug)]
pub struct TimeZoneOffsetToProperty {
    pub(crate) offset: TimeZoneOffset,
    pub(crate) params: Vec<Param>,
}

pub struct TimeZoneOffsetToPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: TimeZoneOffsetToProperty,
}

impl<P> TimeZoneOffsetToPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, offset: TimeZoneOffset) -> TimeZoneOffsetToPropertyBuilder<P> {
        TimeZoneOffsetToPropertyBuilder {
            owner,
            inner: TimeZoneOffsetToProperty {
                offset,
                params: Vec::new(),
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::TimeZoneOffsetTo);
}

impl_other_component_params_builder!(TimeZoneOffsetToPropertyBuilder<P>);

#[derive(Debug)]
pub struct TimeZoneOffsetFromProperty {
    pub(crate) offset: TimeZoneOffset,
    pub(crate) params: Vec<Param>,
}

pub struct TimeZoneOffsetFromPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: TimeZoneOffsetFromProperty,
}

impl<P> TimeZoneOffsetFromPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, offset: TimeZoneOffset) -> TimeZoneOffsetFromPropertyBuilder<P> {
        TimeZoneOffsetFromPropertyBuilder {
            owner,
            inner: TimeZoneOffsetFromProperty {
                offset,
                params: Vec::new(),
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::TimeZoneOffsetFrom);
}

impl_other_component_params_builder!(TimeZoneOffsetFromPropertyBuilder<P>);

#[derive(Debug)]
pub struct TimeZoneNameProperty {
    pub(crate) value: String,
    pub(crate) params: Vec<Param>,
}

pub struct TimeZoneNamePropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: TimeZoneNameProperty,
}

impl<P> TimeZoneNamePropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, value: String) -> TimeZoneNamePropertyBuilder<P> {
        TimeZoneNamePropertyBuilder {
            owner,
            inner: TimeZoneNameProperty {
                value,
                params: Vec::new(),
            },
        }
    }

    language_param!();

    impl_finish_component_property_build!(ComponentProperty::TimeZoneName);
}

impl_other_component_params_builder!(TimeZoneNamePropertyBuilder<P>);

#[derive(Debug, Clone)]
pub enum Action {
    Audio,
    Display,
    Email,
    XName(String),
    IanaToken(String),
}

#[derive(Debug)]
pub struct ActionProperty {
    pub(crate) value: Action,
    pub(crate) params: Vec<Param>,
}

pub struct ActionPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: ActionProperty,
}

impl<P> ActionPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, value: Action) -> ActionPropertyBuilder<P> {
        ActionPropertyBuilder {
            owner,
            inner: ActionProperty {
                value,
                params: Vec::new(),
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::Action);
}

impl_other_component_params_builder!(ActionPropertyBuilder<P>);

#[derive(Debug)]
pub struct RelativeTriggerProperty {
    pub(crate) value: Duration,
    pub(crate) params: Vec<Param>,
}

pub struct RelativeTriggerPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: RelativeTriggerProperty,
}

impl<P> RelativeTriggerPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, value: Duration) -> RelativeTriggerPropertyBuilder<P> {
        RelativeTriggerPropertyBuilder {
            owner,
            inner: RelativeTriggerProperty {
                value,
                params: vec![Param::ValueType {
                    value: Value::Duration,
                }],
            },
        }
    }

    pub fn add_related(mut self, related: Related) -> Self {
        self.inner.params.push(Param::Related { related });
        self
    }

    pub fn finish_property(mut self) -> P {
        self.owner
            .add_property(ComponentProperty::Trigger(Trigger::Relative(self.inner)));
        self.owner
    }
}

impl_other_component_params_builder!(RelativeTriggerPropertyBuilder<P>);

#[derive(Debug)]
pub struct AbsoluteTriggerProperty {
    pub(crate) date: time::Date,
    pub(crate) time: time::Time,
    pub(crate) is_utc: bool,
    pub(crate) params: Vec<Param>,
}

pub struct AbsoluteTriggerPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: AbsoluteTriggerProperty,
}

impl<P> AbsoluteTriggerPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(
        owner: P,
        date: time::Date,
        time: time::Time,
    ) -> AbsoluteTriggerPropertyBuilder<P> {
        AbsoluteTriggerPropertyBuilder {
            owner,
            inner: AbsoluteTriggerProperty {
                date,
                time,
                is_utc: false,
                params: vec![Param::ValueType {
                    value: Value::DateTime,
                }],
            },
        }
    }

    add_is_utc!();

    pub fn finish_property(mut self) -> P {
        self.owner
            .add_property(ComponentProperty::Trigger(Trigger::Absolute(self.inner)));
        self.owner
    }
}

impl_other_component_params_builder!(AbsoluteTriggerPropertyBuilder<P>);

#[derive(Debug)]
pub struct RepeatProperty {
    pub(crate) value: u32,
    pub(crate) params: Vec<Param>,
}

pub struct RepeatPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: RepeatProperty,
}

impl<P> RepeatPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, value: u32) -> RepeatPropertyBuilder<P> {
        RepeatPropertyBuilder {
            owner,
            inner: RepeatProperty {
                value,
                params: Vec::new(),
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::Repeat);
}

impl_other_component_params_builder!(RepeatPropertyBuilder<P>);
