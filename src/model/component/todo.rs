use crate::model::component::{
    add_alarms, add_attach, add_categories, add_class, add_comment, add_contact, add_created,
    add_date_time_stamp, add_date_time_start, add_description, add_duration,
    add_exception_date_times, add_geographic_position, add_last_modified, add_location,
    add_organizer, add_priority, add_recurrence_date, add_recurrence_id, add_recurrence_rule,
    add_related, add_request_status, add_resources, add_sequence, add_summary,
    add_unique_identifier, add_url, impl_finish_component_build, impl_other_component_properties,
    AddAlarmComponent, AlarmComponent, CalendarComponent, ComponentProperty,
};
use crate::model::impl_component_access;
use crate::model::object::ICalObjectBuilder;
use crate::model::param::ParticipationStatusToDo;
use crate::model::property::{
    AddComponentProperty, AttendeePropertyBuilder, CompletedPropertyBuilder,
    DateTimeDuePropertyBuilder, IanaComponentPropertyBuilder, PercentCompletePropertyBuilder,
    StatusPropertyBuilder, StatusToDo, XComponentPropertyBuilder,
};

#[derive(Debug, PartialEq)]
pub struct ToDoComponent {
    pub(crate) properties: Vec<ComponentProperty>,
    pub(crate) alarms: Vec<CalendarComponent>,
}

impl_component_access!(ToDoComponent);

impl ToDoComponent {
    pub(crate) fn new() -> Self {
        ToDoComponent {
            properties: Vec::new(),
            alarms: Vec::new(),
        }
    }

    pub fn alarms(&self) -> &[CalendarComponent] {
        &self.alarms
    }
}

impl Default for ToDoComponent {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ToDoComponentBuilder {
    owner: ICalObjectBuilder,
    inner: ToDoComponent,
}

impl ToDoComponentBuilder {
    pub(crate) fn new(owner: ICalObjectBuilder) -> Self {
        ToDoComponentBuilder {
            owner,
            inner: ToDoComponent {
                properties: Vec::new(),
                alarms: Vec::new(),
            },
        }
    }

    add_date_time_stamp!();

    add_unique_identifier!();

    add_class!();

    pub fn add_date_time_completed(
        self,
        date: time::Date,
        time: time::Time,
    ) -> CompletedPropertyBuilder<Self> {
        CompletedPropertyBuilder::new(self, date, time)
    }

    add_created!();

    add_description!();

    add_date_time_start!();

    add_geographic_position!();

    add_last_modified!();

    add_location!();

    add_organizer!();

    pub fn add_percent_complete(self, value: u8) -> PercentCompletePropertyBuilder<Self> {
        PercentCompletePropertyBuilder::new(self, value)
    }

    add_priority!();

    add_recurrence_id!();

    add_sequence!();

    pub fn add_status(self, value: StatusToDo) -> StatusPropertyBuilder<Self> {
        StatusPropertyBuilder::new(self, value.into())
    }

    add_summary!();

    add_url!();

    add_recurrence_rule!();

    pub fn add_due_date_time(
        self,
        date: time::Date,
        time: Option<time::Time>,
    ) -> DateTimeDuePropertyBuilder<Self> {
        DateTimeDuePropertyBuilder::new(self, date, time)
    }

    add_duration!();

    add_attach!();

    pub fn add_attendee(
        self,
        value: &str,
    ) -> AttendeePropertyBuilder<Self, ParticipationStatusToDo> {
        AttendeePropertyBuilder::new(self, value.to_string())
    }

    add_categories!();

    add_comment!();

    add_contact!();

    add_exception_date_times!();

    add_request_status!();

    add_related!();

    add_resources!();

    add_recurrence_date!();

    impl_other_component_properties!(
        XComponentPropertyBuilder,
        IanaComponentPropertyBuilder,
        ToDoComponentBuilder
    );

    add_alarms!();

    impl_finish_component_build!(CalendarComponent::ToDo);
}

impl AddComponentProperty for ToDoComponentBuilder {
    fn add_property(&mut self, property: ComponentProperty) {
        self.inner.properties.push(property);
    }
}

impl AddAlarmComponent for ToDoComponentBuilder {
    fn add_alarm(mut self, alarm: AlarmComponent) -> Self {
        self.inner.alarms.push(CalendarComponent::Alarm(alarm));
        self
    }
}
