use crate::model::component::daylight::DaylightComponentBuilder;
use crate::model::component::standard::StandardComponentBuilder;
use crate::model::component::{
    add_last_modified, impl_finish_component_build, impl_other_component_properties,
    CalendarComponent, ComponentProperty,
};
use crate::model::impl_component_access;
use crate::model::object::ICalObjectBuilder;
use crate::model::property::{
    AddComponentProperty, IanaComponentPropertyBuilder, TimeZoneIdPropertyBuilder,
    TimeZoneUrlPropertyBuilder, XComponentPropertyBuilder,
};

#[derive(Debug, PartialEq)]
pub struct TimeZoneComponent {
    pub(crate) properties: Vec<ComponentProperty>,
    pub(crate) components: Vec<CalendarComponent>,
}

impl_component_access!(TimeZoneComponent);

impl TimeZoneComponent {
    pub(crate) fn new() -> Self {
        TimeZoneComponent {
            properties: Vec::new(),
            components: Vec::new(),
        }
    }

    pub fn nested_components(&self) -> &[CalendarComponent] {
        &self.components
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
        value: &str,
        unique_registry_id: bool,
    ) -> TimeZoneIdPropertyBuilder<Self> {
        TimeZoneIdPropertyBuilder::new(self, value.to_string(), unique_registry_id)
    }

    add_last_modified!();

    pub fn add_time_zone_url(self, value: &str) -> TimeZoneUrlPropertyBuilder<Self> {
        TimeZoneUrlPropertyBuilder::new(self, value.to_string())
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
