use crate::model::component::daylight::DaylightComponentBuilder;
use crate::model::component::standard::StandardComponentBuilder;
use crate::model::event::{EventComponent, EventComponentBuilder};
use crate::model::{
    add_last_modified, impl_finish_component_build, impl_other_component_properties,
    AddComponentProperty, CalendarComponent, ComponentProperty, ICalObjectBuilder,
    IanaComponentPropertyBuilder, TimeZoneUrlPropertyBuilder, XComponentPropertyBuilder,
};
use crate::prelude::TimeZoneIdPropertyBuilder;

pub struct TimeZoneComponent {
    pub(crate) properties: Vec<ComponentProperty>,
    pub(crate) components: Vec<CalendarComponent>,
}

impl TimeZoneComponent {
    pub(crate) fn new() -> Self {
        TimeZoneComponent {
            properties: Vec::new(),
            components: Vec::new(),
        }
    }
}

impl Default for TimeZoneComponent {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TimeZoneComponentBuilder {
    owner: ICalObjectBuilder,
    pub(crate) inner: TimeZoneComponent,
}

impl TimeZoneComponentBuilder {
    pub(crate) fn new(owner: ICalObjectBuilder) -> Self {
        TimeZoneComponentBuilder {
            owner,
            inner: TimeZoneComponent {
                properties: Vec::new(),
                components: Vec::new(),
            },
        }
    }

    pub fn add_time_zone_id(
        self,
        value: String,
        unique_registry_id: bool,
    ) -> TimeZoneIdPropertyBuilder<Self> {
        TimeZoneIdPropertyBuilder::new(self, value, unique_registry_id)
    }

    add_last_modified!();

    pub fn add_time_zone_url(self, value: String) -> TimeZoneUrlPropertyBuilder<Self> {
        TimeZoneUrlPropertyBuilder::new(self, value)
    }

    pub fn add_standard_time(
        self,
        builder: fn(StandardComponentBuilder) -> StandardComponentBuilder,
    ) -> Self {
        builder(StandardComponentBuilder::new(self)).build()
    }

    pub fn add_daylight_time(
        self,
        builder: fn(DaylightComponentBuilder) -> DaylightComponentBuilder,
    ) -> Self {
        builder(DaylightComponentBuilder::new(self)).build()
    }

    impl_other_component_properties!(
        XComponentPropertyBuilder,
        IanaComponentPropertyBuilder,
        TimeZoneComponentBuilder
    );

    impl_finish_component_build!(CalendarComponent::TimeZone);
}

impl AddComponentProperty for TimeZoneComponentBuilder {
    fn add_property(&mut self, property: ComponentProperty) {
        self.inner.properties.push(property);
    }
}
