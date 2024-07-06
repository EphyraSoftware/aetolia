use std::io::Write;
use crate::serialize::WriteModel;

impl WriteModel for crate::model::ICalObject {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write(b"BEGIN:VCALENDAR")?;
        for property in &self.properties {
            writer.write(b"\r\n")?;
            property.write_model(writer)?;
        }
        for component in &self.components {
            writer.write(b"\r\n")?;
            component.write_model(writer)?;
        }
        writer.write(b"\r\nEND:VCALENDAR\r\n")?;

        Ok(())
    }
}

impl WriteModel for crate::model::CalendarProperty {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        match self {
            crate::model::CalendarProperty::ProductId(property) => {
                writer.write(b"PRODID")?;
                for param in &property.params {
                    writer.write(b";")?;
                    param.write_model(writer)?;
                }
                writer.write(b":")?;
                writer.write(property.value.as_bytes())?;
            }
            _ => {
                unimplemented!()
            }
        }

        Ok(())
    }
}
