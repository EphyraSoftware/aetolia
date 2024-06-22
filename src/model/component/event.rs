use crate::model::component::{
    impl_finish_component_build, impl_other_component_properties, CalendarComponent,
};
use crate::model::object::ICalObjectBuilder;
use crate::model::property::{
    AddComponentProperty, ComponentProperty, DateTimeStampPropertyBuilder,
    XComponentPropertyBuilder,
};
use crate::model::DateTimeStartPropertyBuilder;
use crate::prelude::UniqueIdentifierPropertyBuilder;

pub struct EventComponent {
    pub(crate) properties: Vec<ComponentProperty>,
    alarms: Vec<CalendarComponent>,
}

pub struct EventComponentBuilder {
    owner: ICalObjectBuilder,
    inner: EventComponent,
}

impl EventComponentBuilder {
    pub(crate) fn new(owner: ICalObjectBuilder) -> EventComponentBuilder {
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
    ) -> DateTimeStampPropertyBuilder<EventComponentBuilder> {
        DateTimeStampPropertyBuilder::new(self, date, time)
    }

    pub fn add_uid<V: ToString>(
        self,
        value: V,
    ) -> UniqueIdentifierPropertyBuilder<EventComponentBuilder> {
        UniqueIdentifierPropertyBuilder::new(self, value.to_string())
    }

    pub fn add_date_time_start(
        self,
        date: time::Date,
        time: Option<time::Time>,
    ) -> DateTimeStartPropertyBuilder<EventComponentBuilder> {
        DateTimeStartPropertyBuilder::new(self, date, time)
    }

    impl_other_component_properties!(XComponentPropertyBuilder, EventComponentBuilder);

    impl_finish_component_build!(CalendarComponent::Event);
}

impl AddComponentProperty for EventComponentBuilder {
    fn add_property(&mut self, property: ComponentProperty) {
        self.inner.properties.push(property);
    }
}
