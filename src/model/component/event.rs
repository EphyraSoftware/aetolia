use crate::common::TimeTransparency;
use crate::model::alarm::{AddAlarmComponent, AlarmComponent};
use crate::model::component::{
    impl_finish_component_build, impl_other_component_properties, CalendarComponent,
};
use crate::model::object::ICalObjectBuilder;
use crate::model::property::{AddComponentProperty, ComponentProperty, XComponentPropertyBuilder};
use crate::model::{
    add_alarms, add_attach, add_categories, add_class, add_comment, add_contact, add_created,
    add_date_time_end, add_date_time_stamp, add_date_time_start, add_description, add_duration,
    add_exception_date_times, add_geographic_position, add_last_modified, add_location,
    add_organizer, add_priority, add_recurrence_date, add_recurrence_id, add_recurrence_rule,
    add_related, add_request_status, add_resources, add_sequence, add_summary,
    add_unique_identifier, add_url, IanaComponentPropertyBuilder, ParticipationStatusEvent,
    StatusEvent, StatusPropertyBuilder, TimeTransparencyPropertyBuilder,
};
use crate::prelude::{impl_component_access, AttendeePropertyBuilder};

#[derive(Debug, PartialEq)]
pub struct EventComponent {
    pub(crate) properties: Vec<ComponentProperty>,
    pub(crate) alarms: Vec<CalendarComponent>,
}

impl_component_access!(EventComponent);

impl EventComponent {
    pub fn new() -> Self {
        EventComponent {
            properties: Vec::new(),
            alarms: Vec::new(),
        }
    }

    pub fn alarms(&self) -> &[CalendarComponent] {
        &self.alarms
    }
}

impl Default for EventComponent {
    fn default() -> Self {
        Self::new()
    }
}

pub struct EventComponentBuilder {
    owner: ICalObjectBuilder,
    inner: EventComponent,
}

impl EventComponentBuilder {
    pub(crate) fn new(owner: ICalObjectBuilder) -> Self {
        EventComponentBuilder {
            owner,
            inner: EventComponent::new(),
        }
    }

    add_date_time_stamp!();

    add_unique_identifier!();

    add_date_time_start!();

    add_class!();

    add_created!();

    add_description!();

    add_geographic_position!();

    add_last_modified!();

    add_location!();

    add_organizer!();

    add_priority!();

    add_sequence!();

    pub fn add_status(self, value: StatusEvent) -> StatusPropertyBuilder<Self> {
        StatusPropertyBuilder::new(self, value.into())
    }

    add_summary!();

    pub fn add_time_transparency(
        self,
        value: TimeTransparency,
    ) -> TimeTransparencyPropertyBuilder<Self> {
        TimeTransparencyPropertyBuilder::new(self, value)
    }

    add_url!();

    add_recurrence_id!();

    add_recurrence_rule!();

    add_date_time_end!();

    add_duration!();

    add_attach!();

    pub fn add_attendee(
        self,
        value: String,
    ) -> AttendeePropertyBuilder<Self, ParticipationStatusEvent> {
        AttendeePropertyBuilder::new(self, value)
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
        EventComponentBuilder
    );

    add_alarms!();

    impl_finish_component_build!(CalendarComponent::Event);
}

impl AddComponentProperty for EventComponentBuilder {
    fn add_property(&mut self, property: ComponentProperty) {
        self.inner.properties.push(property);
    }
}

impl AddAlarmComponent for EventComponentBuilder {
    fn add_alarm(mut self, alarm: AlarmComponent) -> Self {
        self.inner.alarms.push(CalendarComponent::Alarm(alarm));
        self
    }
}
