use crate::serialize::WriteModel;
use std::io::Write;

impl WriteModel for crate::model::ICalObject {
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

impl WriteModel for crate::model::CalendarProperty {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        match self {
            crate::model::CalendarProperty::ProductId(property) => {
                writer.write_all(b"PRODID")?;
                for param in &property.params {
                    writer.write_all(b";")?;
                    param.write_model(writer)?;
                }
                writer.write_all(b":")?;
                writer.write_all(property.value.as_bytes())?;
            }
            _ => {
                unimplemented!()
            }
        }

        Ok(())
    }
}
