use crate::model::component::{
    impl_finish_component_build, impl_other_component_properties, CalendarComponent,
};
use crate::model::object::ICalObjectBuilder;
use crate::model::property::{
    AddComponentProperty, ComponentProperty, IanaComponentPropertyBuilder,
};
use crate::model::XComponentPropertyBuilder;
use crate::prelude::impl_component_access;

#[derive(Debug)]
pub struct IanaComponent {
    pub(crate) name: String,
    pub(crate) properties: Vec<ComponentProperty>,
}

impl_component_access!(IanaComponent);

impl IanaComponent {
    pub(crate) fn new(name: String) -> Self {
        IanaComponent {
            name,
            properties: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
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

    impl_other_component_properties!(
        XComponentPropertyBuilder,
        IanaComponentPropertyBuilder,
        IanaComponentBuilder
    );

    impl_finish_component_build!(CalendarComponent::IanaComponent);
}
