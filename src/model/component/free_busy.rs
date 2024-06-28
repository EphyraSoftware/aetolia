use crate::model::component::journal::{JournalComponent, JournalComponentBuilder};
use crate::model::{
    add_comment, add_contact, add_date_time_end, add_date_time_stamp, add_date_time_start,
    add_organizer, add_request_status, add_unique_identifier, add_url, impl_finish_component_build,
    impl_other_component_properties, AddComponentProperty, AttendeeParamBuilder, CalendarComponent,
    ComponentProperty, FreeBusyPropertyBuilder, FreeBusyTimeType, ICalObjectBuilder,
    IanaComponentPropertyBuilder, ParticipationStatusEvent, XComponentPropertyBuilder,
};
use crate::prelude::Period;

pub struct FreeBusyComponent {
    pub(crate) properties: Vec<ComponentProperty>,
}

pub struct FreeBusyComponentBuilder {
    owner: ICalObjectBuilder,
    inner: FreeBusyComponent,
}

impl FreeBusyComponentBuilder {
    pub(crate) fn new(owner: ICalObjectBuilder) -> Self {
        FreeBusyComponentBuilder {
            owner,
            inner: FreeBusyComponent {
                properties: Vec::new(),
            },
        }
    }

    add_date_time_stamp!();

    add_unique_identifier!();

    add_contact!();

    add_date_time_start!();

    add_date_time_end!();

    add_organizer!();

    add_url!();

    pub fn add_attendee(
        self,
        value: String,
    ) -> AttendeeParamBuilder<Self, ParticipationStatusEvent> {
        AttendeeParamBuilder::new(self, value)
    }

    add_comment!();

    pub fn add_free_busy_time(
        self,
        free_busy_time_type: FreeBusyTimeType,
        periods: Vec<Period>,
    ) -> FreeBusyPropertyBuilder<Self> {
        FreeBusyPropertyBuilder::new(self, free_busy_time_type, periods)
    }

    add_request_status!();

    impl_other_component_properties!(
        XComponentPropertyBuilder,
        IanaComponentPropertyBuilder,
        FreeBusyComponentBuilder
    );

    impl_finish_component_build!(CalendarComponent::FreeBusy);
}

impl AddComponentProperty for FreeBusyComponentBuilder {
    fn add_property(&mut self, property: ComponentProperty) {
        self.inner.properties.push(property);
    }
}
