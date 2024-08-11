use crate::serialize::WriteModel;
use std::io::Write;

impl WriteModel for crate::model::object::ICalObject {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_all(b"BEGIN:VCALENDAR")?;
        for property in &self.properties {
            writer.write_all(b"\r\n")?;
            property.write_model(writer)?;
        }
        for component in &self.components {
            writer.write_all(b"\r\n")?;
            component.write_model(writer)?;
        }
        writer.write_all(b"\r\nEND:VCALENDAR\r\n")?;

        Ok(())
    }
}

impl WriteModel for crate::model::property::CalendarProperty {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        use crate::model::property::CalendarProperty;

        match self {
            CalendarProperty::ProductId(property) => {
                writer.write_all(b"PRODID")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                writer.write_all(property.value.as_bytes())?;
            }
            CalendarProperty::Version(property) => {
                writer.write_all(b"VERSION")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;

                if let Some(min_version) = &property.min_version {
                    write!(writer, "{};", min_version)?;
                }

                writer.write_all(property.max_version.as_bytes())?;
            }
            CalendarProperty::CalendarScale(property) => {
                writer.write_all(b"CALSCALE")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                writer.write_all(property.value.as_bytes())?;
            }
            CalendarProperty::Method(property) => {
                writer.write_all(b"METHOD")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                writer.write_all(property.value.as_bytes())?;
            }
            CalendarProperty::XProperty(property) => {
                writer.write_all(b"X-")?;
                writer.write_all(property.name.as_bytes())?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                writer.write_all(property.value.as_bytes())?;
            }
            CalendarProperty::IanaProperty(property) => {
                writer.write_all(property.name.as_bytes())?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                writer.write_all(property.value.as_bytes())?;
            }
        }

        Ok(())
    }
}
