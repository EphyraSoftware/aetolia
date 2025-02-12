use crate::convert::ToModel;
use crate::error::AetoliaResult;

impl ToModel for crate::parser::types::ICalendar<'_> {
    type Model = crate::model::object::ICalObject;

    fn to_model(&self) -> AetoliaResult<Self::Model> {
        let mut calendar = crate::model::object::ICalObject::new();

        calendar.properties.reserve(self.properties.len());
        for property in &self.properties {
            calendar.properties.push(property.to_model()?);
        }

        calendar.components.reserve(self.components.len());
        for component in &self.components {
            calendar.components.push(component.to_model()?);
        }

        Ok(calendar)
    }
}
