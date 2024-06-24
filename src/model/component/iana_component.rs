use crate::model::component::{
    impl_finish_component_build, impl_other_component_properties, CalendarComponent,
};
use crate::model::object::ICalObjectBuilder;
use crate::model::property::{
    AddComponentProperty, ComponentProperty, IanaComponentPropertyBuilder,
};

pub struct IanaComponent {
    name: String,
    pub(crate) properties: Vec<ComponentProperty>,
}

impl AddComponentProperty for IanaComponentBuilder {
    fn add_property(&mut self, property: ComponentProperty) {
        self.inner.properties.push(property);
    }
}

pub struct IanaComponentBuilder {
    owner: ICalObjectBuilder,
    inner: IanaComponent,
}

impl IanaComponentBuilder {
    pub(crate) fn new(owner: ICalObjectBuilder, name: String) -> IanaComponentBuilder {
        IanaComponentBuilder {
            owner,
            inner: IanaComponent {
                name,
                properties: Vec::new(),
            },
        }
    }

    impl_other_component_properties!(IanaComponentPropertyBuilder, IanaComponentBuilder);

    impl_finish_component_build!(CalendarComponent::IanaComponent);
}