use crate::model::component::{
    impl_finish_component_build, impl_other_component_properties, CalendarComponent,
};
use crate::model::object::ICalObjectBuilder;
use crate::model::property::{AddComponentProperty, ComponentProperty, XComponentPropertyBuilder};
use crate::model::IanaComponentPropertyBuilder;
use crate::prelude::impl_component_access;

#[derive(Debug)]
pub struct XComponent {
    pub(crate) name: String,
    pub(crate) properties: Vec<ComponentProperty>,
}

impl_component_access!(XComponent);

impl XComponent {
    pub(crate) fn new(name: String) -> Self {
        XComponent {
            name,
            properties: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl AddComponentProperty for XComponentBuilder {
    fn add_property(&mut self, property: ComponentProperty) {
        self.inner.properties.push(property);
    }
}

pub struct XComponentBuilder {
    owner: ICalObjectBuilder,
    inner: XComponent,
}

impl XComponentBuilder {
    pub(crate) fn new(owner: ICalObjectBuilder, name: String) -> XComponentBuilder {
        XComponentBuilder {
            owner,
            inner: XComponent {
                name,
                properties: Vec::new(),
            },
        }
    }

    impl_other_component_properties!(
        XComponentPropertyBuilder,
        IanaComponentPropertyBuilder,
        XComponentBuilder
    );

    impl_finish_component_build!(CalendarComponent::XComponent);
}
