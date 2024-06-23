use crate::model::component::{
    impl_finish_component_build, impl_other_component_properties, CalendarComponent,
};
use crate::model::object::ICalObjectBuilder;
use crate::model::property::{
    AddComponentProperty, ComponentProperty, DateTimeStampPropertyBuilder,
    XComponentPropertyBuilder,
};
use crate::model::{
    CreatedPropertyBuilder, DateTimeStartPropertyBuilder, GeographicPositionPropertyBuilder,
    LocationPropertyBuilder, OrganizerPropertyBuilder, PriorityPropertyBuilder,
    SequencePropertyBuilder,
};
use crate::prelude::{
    ClassPropertyBuilder, DescriptionPropertyBuilder, UniqueIdentifierPropertyBuilder,
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

    impl_other_component_properties!(XComponentPropertyBuilder, EventComponentBuilder);

    impl_finish_component_build!(CalendarComponent::Event);
}

impl AddComponentProperty for EventComponentBuilder {
    fn add_property(&mut self, property: ComponentProperty) {
        self.inner.properties.push(property);
    }
}
