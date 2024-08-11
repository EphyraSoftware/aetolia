use crate::model::component::time_zone::TimeZoneComponentBuilder;
use crate::model::component::{
    add_comment, add_date_time_start, add_recurrence_date, add_recurrence_rule,
    impl_other_component_properties, CalendarComponent,
};
use crate::model::impl_component_access;
use crate::model::property::{
    AddComponentProperty, ComponentProperty, IanaComponentPropertyBuilder,
    TimeZoneNamePropertyBuilder, TimeZoneOffset, TimeZoneOffsetFromPropertyBuilder,
    TimeZoneOffsetToPropertyBuilder, XComponentPropertyBuilder,
};

#[derive(Debug, PartialEq)]
pub struct DaylightComponent {
    pub(crate) properties: Vec<ComponentProperty>,
}

impl_component_access!(DaylightComponent);

impl DaylightComponent {
    pub(crate) fn new() -> Self {
        DaylightComponent {
            properties: Vec::new(),
        }
    }
}

impl Default for DaylightComponent {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DaylightComponentBuilder {
    owner: TimeZoneComponentBuilder,
    inner: DaylightComponent,
}

impl DaylightComponentBuilder {
    pub(crate) fn new(owner: TimeZoneComponentBuilder) -> Self {
        DaylightComponentBuilder {
            owner,
            inner: DaylightComponent {
                properties: Vec::new(),
            },
        }
    }

    add_date_time_start!();

    pub fn add_time_zone_offset_to(
        self,
        offset: TimeZoneOffset,
    ) -> TimeZoneOffsetToPropertyBuilder<Self> {
        TimeZoneOffsetToPropertyBuilder::new(self, offset)
    }

    pub fn add_time_zone_offset_from(
        self,
        offset: TimeZoneOffset,
    ) -> TimeZoneOffsetFromPropertyBuilder<Self> {
        TimeZoneOffsetFromPropertyBuilder::new(self, offset)
    }

    add_recurrence_rule!();

    add_comment!();

    add_recurrence_date!();

    pub fn add_time_zone_name(self, value: &str) -> TimeZoneNamePropertyBuilder<Self> {
        TimeZoneNamePropertyBuilder::new(self, value.to_string())
    }

    impl_other_component_properties!(
        XComponentPropertyBuilder,
        IanaComponentPropertyBuilder,
        DaylightComponentBuilder
    );

    pub(crate) fn build(mut self) -> TimeZoneComponentBuilder {
        self.owner
            .inner
            .components
            .push(CalendarComponent::Daylight(self.inner));
        self.owner
    }
}

impl AddComponentProperty for DaylightComponentBuilder {
    fn add_property(&mut self, property: ComponentProperty) {
        self.inner.properties.push(property);
    }
}
