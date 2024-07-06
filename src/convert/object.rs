use crate::convert::ToModel;

impl ToModel for crate::parser::ICalendar<'_> {
    type Model = crate::model::ICalObject;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let mut calendar = crate::model::ICalObject::new();

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
