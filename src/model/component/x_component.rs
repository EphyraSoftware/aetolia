use crate::model::component::{
    impl_finish_component_build, impl_other_component_properties, CalendarComponent,
};
use crate::model::object::ICalObjectBuilder;
use crate::model::property::{AddComponentProperty, ComponentProperty, XComponentPropertyBuilder};

pub struct XComponent {
    name: String,
    pub(crate) properties: Vec<ComponentProperty>,
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

    impl_other_component_properties!(XComponentPropertyBuilder, XComponentBuilder);

    impl_finish_component_build!(CalendarComponent::XComponent);
}
