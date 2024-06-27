use crate::model::component::{
    impl_finish_component_build, impl_other_component_properties, CalendarComponent,
};
use crate::model::object::ICalObjectBuilder;
use crate::model::property::{
    AddComponentProperty, ComponentProperty, DateTimeStampPropertyBuilder,
    XComponentPropertyBuilder,
};
use crate::model::{
    CategoriesParamBuilder, CreatedPropertyBuilder, DateTimeEndPropertyBuilder,
    DateTimeStartPropertyBuilder, Duration, DurationPropertyBuilder, Frequency,
    GeographicPositionPropertyBuilder, IanaComponentPropertyBuilder, LocationPropertyBuilder,
    OrganizerPropertyBuilder, ParticipationStatusEvent, PriorityPropertyBuilder,
    RecurrenceDateTimesPropertyBuilder, RecurrenceIdPropertyBuilder, RecurrenceRule,
    RecurrenceRulePropertyBuilder, RelatedToPropertyBuilder, RequestStatusPropertyBuilder,
    ResourcesPropertyBuilder, SequencePropertyBuilder, StatusEvent, StatusProperty,
    StatusPropertyBuilder, TimeTransparency, TimeTransparencyPropertyBuilder, UrlPropertyBuilder,
};
use crate::prelude::{
    AttachPropertyBuilder, AttendeeParamBuilder, ClassPropertyBuilder, CommentParamBuilder,
    ContactParamBuilder, DescriptionPropertyBuilder, ExceptionDateTimesPropertyBuilder,
    LastModifiedPropertyBuilder, Period, SummaryPropertyBuilder, UniqueIdentifierPropertyBuilder,
};

pub struct EventComponent {
    pub(crate) properties: Vec<ComponentProperty>,
    alarms: Vec<CalendarComponent>,
}

pub struct EventComponentBuilder {
    owner: ICalObjectBuilder,
    inner: EventComponent,
}

impl EventComponentBuilder {
    pub(crate) fn new(owner: ICalObjectBuilder) -> Self {
        EventComponentBuilder {
            owner,
            inner: EventComponent {
                properties: Vec::new(),
                alarms: Vec::new(),
            },
        }
    }

    pub fn add_date_time_stamp(
        self,
        date: time::Date,
        time: time::Time,
    ) -> DateTimeStampPropertyBuilder<Self> {
        DateTimeStampPropertyBuilder::new(self, date, time)
    }

    pub fn add_uid<V: ToString>(self, value: V) -> UniqueIdentifierPropertyBuilder<Self> {
        UniqueIdentifierPropertyBuilder::new(self, value.to_string())
    }

    pub fn add_date_time_start(
        self,
        date: time::Date,
        time: Option<time::Time>,
    ) -> DateTimeStartPropertyBuilder<Self> {
        DateTimeStartPropertyBuilder::new(self, date, time)
    }

    pub fn add_class<V: ToString>(self, value: V) -> ClassPropertyBuilder<Self> {
        ClassPropertyBuilder::new(self, value.to_string())
    }

    pub fn add_created(self, date: time::Date, time: time::Time) -> CreatedPropertyBuilder<Self> {
        CreatedPropertyBuilder::new(self, date, time)
    }

    pub fn add_description<V: ToString>(self, value: V) -> DescriptionPropertyBuilder<Self> {
        DescriptionPropertyBuilder::new(self, value.to_string())
    }

    pub fn add_geographic_position(
        self,
        latitude: f64,
        longitude: f64,
    ) -> GeographicPositionPropertyBuilder<Self> {
        GeographicPositionPropertyBuilder::new(self, latitude, longitude)
    }

    pub fn add_last_modified(
        self,
        date: time::Date,
        time: time::Time,
    ) -> LastModifiedPropertyBuilder<Self> {
        LastModifiedPropertyBuilder::new(self, date, time)
    }

    pub fn add_location(self, value: String) -> LocationPropertyBuilder<Self> {
        LocationPropertyBuilder::new(self, value)
    }

    pub fn add_organizer(self, value: String) -> OrganizerPropertyBuilder<Self> {
        OrganizerPropertyBuilder::new(self, value)
    }

    pub fn add_priority(self, value: u8) -> PriorityPropertyBuilder<Self> {
        PriorityPropertyBuilder::new(self, value)
    }

    pub fn add_sequence(self, value: u32) -> SequencePropertyBuilder<Self> {
        SequencePropertyBuilder::new(self, value)
    }

    pub fn add_status(self, value: StatusEvent) -> StatusPropertyBuilder<Self> {
        StatusPropertyBuilder::new(self, value.into())
    }

    pub fn add_summary<V: ToString>(self, value: V) -> SummaryPropertyBuilder<Self> {
        SummaryPropertyBuilder::new(self, value.to_string())
    }

    pub fn add_time_transparency(
        self,
        value: TimeTransparency,
    ) -> TimeTransparencyPropertyBuilder<Self> {
        TimeTransparencyPropertyBuilder::new(self, value.to_string())
    }

    pub fn add_url(self, value: String) -> UrlPropertyBuilder<Self> {
        UrlPropertyBuilder::new(self, value)
    }

    pub fn add_recurrence_id(
        self,
        date: time::Date,
        time: Option<time::Time>,
    ) -> RecurrenceIdPropertyBuilder<Self> {
        RecurrenceIdPropertyBuilder::new(self, date, time)
    }

    pub fn add_recurrence_rule(
        self,
        frequency: Frequency,
        builder: fn(RecurrenceRule) -> RecurrenceRule,
    ) -> RecurrenceRulePropertyBuilder<Self> {
        RecurrenceRulePropertyBuilder::new(self, builder(RecurrenceRule::new(frequency)))
    }

    pub fn add_date_time_end(
        self,
        date: time::Date,
        time: Option<time::Time>,
    ) -> DateTimeEndPropertyBuilder<Self> {
        DateTimeEndPropertyBuilder::new(self, date, time)
    }

    pub fn add_duration(self, builder: fn() -> Duration) -> DurationPropertyBuilder<Self> {
        DurationPropertyBuilder::new(self, builder())
    }

    pub fn add_attach_uri(self, value: String) -> AttachPropertyBuilder<Self> {
        AttachPropertyBuilder::new_with_uri(self, value)
    }

    pub fn add_attach_binary(self, value: String) -> AttachPropertyBuilder<Self> {
        AttachPropertyBuilder::new_with_binary(self, value)
    }

    pub fn add_attendee(
        self,
        value: String,
    ) -> AttendeeParamBuilder<Self, ParticipationStatusEvent> {
        AttendeeParamBuilder::new(self, value)
    }

    pub fn add_categories(self, value: Vec<String>) -> CategoriesParamBuilder<Self> {
        CategoriesParamBuilder::new(self, value)
    }

    pub fn add_comment<V: ToString>(self, value: V) -> CommentParamBuilder<Self> {
        CommentParamBuilder::new(self, value.to_string())
    }

    pub fn add_contact<V: ToString>(self, value: V) -> ContactParamBuilder<Self> {
        ContactParamBuilder::new(self, value.to_string())
    }

    pub fn add_exception_date_times(
        self,
        date_times: Vec<(time::Date, Option<time::Time>)>,
    ) -> ExceptionDateTimesPropertyBuilder<Self> {
        ExceptionDateTimesPropertyBuilder::new(self, date_times)
    }

    pub fn add_request_status(
        self,
        status_code: &[u32],
        description: String,
        exception_data: Option<String>,
    ) -> RequestStatusPropertyBuilder<Self> {
        RequestStatusPropertyBuilder::new(self, status_code.to_vec(), description, exception_data)
    }

    pub fn add_related(self, value: String) -> RelatedToPropertyBuilder<Self> {
        RelatedToPropertyBuilder::new(self, value)
    }

    pub fn add_resources(self, value: Vec<String>) -> ResourcesPropertyBuilder<Self> {
        ResourcesPropertyBuilder::new(self, value)
    }

    pub fn add_recurrence_date_date_times(
        self,
        date_times: Vec<(time::Date, Option<time::Time>)>,
    ) -> RecurrenceDateTimesPropertyBuilder<Self> {
        RecurrenceDateTimesPropertyBuilder::new_date_times(self, date_times)
    }

    pub fn add_recurrence_date_periods(
        self,
        periods: Vec<Period>,
    ) -> RecurrenceDateTimesPropertyBuilder<Self> {
        RecurrenceDateTimesPropertyBuilder::new_periods(self, periods)
    }

    impl_other_component_properties!(
        XComponentPropertyBuilder,
        IanaComponentPropertyBuilder,
        EventComponentBuilder
    );

    impl_finish_component_build!(CalendarComponent::Event);
}

impl AddComponentProperty for EventComponentBuilder {
    fn add_property(&mut self, property: ComponentProperty) {
        self.inner.properties.push(property);
    }
}
