use crate::convert::ToModel;
use crate::model::event::EventComponent;
use crate::model::ICalObjectBuilder;

impl ToModel for crate::parser::CalendarComponent<'_> {
    type Model = crate::model::CalendarComponent;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        match self {
            crate::parser::CalendarComponent::Event { properties, .. } => {
                let mut component = EventComponent::new();
                component.properties.reserve(properties.len());

                for property in properties {
                    component.properties.push(property.to_model()?);
                }

                Ok(crate::model::CalendarComponent::Event(component))
            }
            _ => {
                unimplemented!()
            }
        }
    }
}
