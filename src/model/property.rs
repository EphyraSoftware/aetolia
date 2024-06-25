mod duration;
mod recur;

use crate::model::object::ICalObjectBuilder;
use crate::model::param::Param;
use crate::model::param::{impl_other_component_params_builder, impl_other_params_builder};
use crate::model::{
    altrep_param, impl_other_component_properties, language_param, tz_id_param, Encoding, Range,
    Value,
};
use std::fmt::Display;
use std::ops::Deref;

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
    params: Vec<Param>,
    value: String,
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
                params: Vec::new(),
                value,
            },
        }
    }

    impl_finish_property_build!(CalendarProperty::ProductId);
}

impl_other_params_builder!(ProductIdPropertyBuilder);

pub struct VersionProperty {
    params: Vec<Param>,
    min_version: Option<String>,
    max_version: String,
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
                params: Vec::new(),
                min_version,
                max_version,
            },
        }
    }

    impl_finish_property_build!(CalendarProperty::Version);
}

impl_other_params_builder!(VersionPropertyBuilder);

pub struct CalendarScaleProperty {
    params: Vec<Param>,
    value: String,
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
                params: Vec::new(),
                value,
            },
        }
    }

    impl_finish_property_build!(CalendarProperty::CalendarScale);
}

impl_other_params_builder!(CalendarScalePropertyBuilder);

pub struct MethodProperty {
    params: Vec<Param>,
    value: String,
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

pub enum ComponentProperty {
    DateTimeStamp(DateTimeStampProperty),
    UniqueIdentifier(UniqueIdentifierProperty),
    DateTimeStart(DateTimeStartProperty),
    Class(ClassProperty),
    Created(CreatedProperty),
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
    IanaProperty(IanaProperty),
    XProperty(XProperty),
}

pub struct XProperty {
    pub(crate) params: Vec<Param>,
    name: String,
    value: String,
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
                params: Vec::new(),
                name,
                value,
            },
        }
    }

    impl_finish_property_build!(CalendarProperty::XProperty);
}

impl_other_params_builder!(XPropertyBuilder);

pub struct IanaProperty {
    pub(crate) params: Vec<Param>,
    name: String,
    value: String,
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
                params: Vec::new(),
                name,
                value,
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
                params: Vec::new(),
                name,
                value,
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::IanaProperty);
}

impl_other_component_params_builder!(IanaComponentPropertyBuilder<P>);

pub struct DateTimeStampProperty {
    date: time::Date,
    time: time::Time,
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
                params: Vec::new(),
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::DateTimeStamp);
}

impl_other_component_params_builder!(DateTimeStampPropertyBuilder<P>);

pub struct UniqueIdentifierProperty {
    value: String,
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

pub struct DateTimeStartProperty {
    date: time::Date,
    time: Option<time::Time>,
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
            params.push(Param::Value { value: Value::Date })
        }

        DateTimeStartPropertyBuilder {
            owner,
            inner: DateTimeStartProperty { date, time, params },
        }
    }

    tz_id_param!();

    impl_finish_component_property_build!(ComponentProperty::DateTimeStart);
}

impl_other_component_params_builder!(DateTimeStartPropertyBuilder<P>);

pub struct ClassProperty {
    value: String,
    pub(crate) params: Vec<Param>,
}

pub struct ClassPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: ClassProperty,
}

impl<P> ClassPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, value: String) -> ClassPropertyBuilder<P> {
        ClassPropertyBuilder {
            owner,
            inner: ClassProperty {
                value,
                params: Vec::new(),
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::Class);
}

impl_other_component_params_builder!(ClassPropertyBuilder<P>);

pub struct CreatedProperty {
    date: time::Date,
    time: time::Time,
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
                params: Vec::new(),
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::Created);
}

impl_other_component_params_builder!(CreatedPropertyBuilder<P>);

pub struct DescriptionProperty {
    value: String,
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

pub struct GeographicPositionProperty {
    latitude: f64,
    longitude: f64,
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

pub struct LastModifiedProperty {
    date: time::Date,
    time: time::Time,
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
                params: Vec::new(),
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::LastModified);
}

impl_other_component_params_builder!(LastModifiedPropertyBuilder<P>);

pub struct LocationProperty {
    value: String,
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

pub struct OrganizerProperty {
    pub(crate) params: Vec<Param>,
    pub(crate) value: String,
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

    pub fn add_common_name<V: ToString>(mut self, value: V) -> Self {
        self.inner.params.push(Param::CommonName {
            name: value.to_string(),
        });
        self
    }

    // TODO should be a URI
    pub fn add_directory_entry_reference(mut self, value: String) -> Self {
        self.inner
            .params
            .push(Param::DirectoryEntryReference { value });
        self
    }

    // TODO should be a URI
    pub fn add_sent_by(mut self, value: String) -> Self {
        self.inner.params.push(Param::SentBy { value });
        self
    }

    language_param!();

    impl_finish_component_property_build!(ComponentProperty::Organizer);
}

impl_other_component_params_builder!(OrganizerPropertyBuilder<P>);

pub struct PriorityProperty {
    value: u8,
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

pub struct SequenceProperty {
    value: u32,
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

pub struct RequestStatusProperty {
    status_code: Vec<u32>,
    description: String,
    exception_data: Option<String>,
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

pub struct SummaryProperty {
    value: String,
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

pub struct TimeTransparencyProperty {
    value: String,
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
    pub(crate) fn new(owner: P, value: String) -> TimeTransparencyPropertyBuilder<P> {
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

pub struct UrlProperty {
    // TODO should be a URI
    value: String,
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

pub struct RecurrenceIdProperty {
    date: time::Date,
    time: Option<time::Time>,
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
            params.push(Param::Value { value: Value::Date })
        }

        RecurrenceIdPropertyBuilder {
            owner,
            inner: RecurrenceIdProperty { date, time, params },
        }
    }

    tz_id_param!();

    pub fn add_range(mut self, range: Range) -> Self {
        self.inner.params.push(Param::Range { range });
        self
    }

    impl_finish_component_property_build!(ComponentProperty::RecurrenceId);
}

impl_other_component_params_builder!(RecurrenceIdPropertyBuilder<P>);

pub struct RecurrenceRuleProperty {
    rule: RecurrenceRule,
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

pub struct DateTimeEndProperty {
    date: time::Date,
    time: Option<time::Time>,
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
            params.push(Param::Value { value: Value::Date })
        }

        DateTimeEndPropertyBuilder {
            owner,
            inner: DateTimeEndProperty { date, time, params },
        }
    }

    tz_id_param!();

    impl_finish_component_property_build!(ComponentProperty::DateTimeEnd);
}

impl_other_component_params_builder!(DateTimeEndPropertyBuilder<P>);

pub struct DurationProperty {
    duration: duration::Duration,
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
    pub(crate) fn new(owner: P, duration: duration::Duration) -> DurationPropertyBuilder<P> {
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

pub struct AttachProperty {
    value_uri: Option<String>,
    value_binary: Option<String>,
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
                value_uri: Some(uri),
                value_binary: None,
                params: Vec::new(),
            },
        }
    }

    pub(crate) fn new_with_binary(owner: P, binary: String) -> AttachPropertyBuilder<P> {
        AttachPropertyBuilder {
            owner,
            inner: AttachProperty {
                value_uri: None,
                value_binary: Some(binary),
                params: vec![
                    Param::Encoding {
                        encoding: Encoding::Base64,
                    },
                    Param::Value {
                        value: Value::Binary,
                    },
                ],
            },
        }
    }

    pub fn add_fmt_type<U: ToString, V: ToString>(mut self, type_name: U, sub_type_name: V) -> Self {
        self.inner.params.push(Param::FormatType {
            type_name: type_name.to_string(),
            sub_type_name: sub_type_name.to_string(),
        });
        self
    }

    impl_finish_component_property_build!(ComponentProperty::Attach);
}

impl_other_component_params_builder!(AttachPropertyBuilder<P>);
