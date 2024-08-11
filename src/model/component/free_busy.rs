use crate::common::FreeBusyTimeType;
use crate::model::component::{
    add_comment, add_contact, add_date_time_end, add_date_time_stamp, add_date_time_start,
    add_organizer, add_request_status, add_unique_identifier, add_url, impl_finish_component_build,
    impl_other_component_properties, CalendarComponent, ComponentProperty,
};
use crate::model::impl_component_access;
use crate::model::object::ICalObjectBuilder;
use crate::model::param::ParticipationStatusEvent;
use crate::model::property::{
    AddComponentProperty, AttendeePropertyBuilder, FreeBusyTimePropertyBuilder,
    IanaComponentPropertyBuilder, Period, XComponentPropertyBuilder,
};

#[derive(Debug, PartialEq)]
pub struct FreeBusyComponent {
    pub(crate) properties: Vec<ComponentProperty>,
}

impl_component_access!(FreeBusyComponent);

impl FreeBusyComponent {
    pub(crate) fn new() -> Self {
        FreeBusyComponent {
            properties: Vec::new(),
        }
    }
}

impl Default for FreeBusyComponent {
    fn default() -> Self {
        Self::new()
    }
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
        value: &str,
    ) -> AttendeePropertyBuilder<Self, ParticipationStatusEvent> {
        AttendeePropertyBuilder::new(self, value.to_string())
    }

    add_comment!();

    pub fn add_free_busy_time(
        self,
        free_busy_time_type: FreeBusyTimeType,
        periods: Vec<Period>,
    ) -> FreeBusyTimePropertyBuilder<Self> {
        FreeBusyTimePropertyBuilder::new(self, free_busy_time_type, periods)
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
