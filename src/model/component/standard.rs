use crate::model::component::time_zone::TimeZoneComponentBuilder;
use crate::model::component::{
    add_comment, add_date_time_start, add_recurrence_date, add_recurrence_rule,
    impl_other_component_properties, CalendarComponent, ComponentProperty,
};
use crate::model::impl_component_access;
use crate::model::property::{
    AddComponentProperty, IanaComponentPropertyBuilder, TimeZoneNamePropertyBuilder,
    TimeZoneOffset, TimeZoneOffsetFromPropertyBuilder, TimeZoneOffsetToPropertyBuilder,
    XComponentPropertyBuilder,
};

#[derive(Debug, PartialEq)]
pub struct StandardComponent {
    pub(crate) properties: Vec<ComponentProperty>,
}

impl_component_access!(StandardComponent);

impl StandardComponent {
    pub(crate) fn new() -> Self {
        StandardComponent {
            properties: Vec::new(),
        }
    }
}

impl Default for StandardComponent {
    fn default() -> Self {
        Self::new()
    }
}

pub struct StandardComponentBuilder {
    owner: TimeZoneComponentBuilder,
    inner: StandardComponent,
}

impl StandardComponentBuilder {
    pub(crate) fn new(owner: TimeZoneComponentBuilder) -> Self {
        StandardComponentBuilder {
            owner,
            inner: StandardComponent {
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
        StandardComponentBuilder
    );

    pub(crate) fn build(mut self) -> TimeZoneComponentBuilder {
        self.owner
            .inner
            .components
            .push(CalendarComponent::Standard(self.inner));
        self.owner
    }
}

impl AddComponentProperty for StandardComponentBuilder {
    fn add_property(&mut self, property: ComponentProperty) {
        self.inner.properties.push(property);
    }
}
