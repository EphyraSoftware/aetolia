use std::io::Write;
use crate::serialize::WriteModel;

impl WriteModel for crate::model::ComponentProperty {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        match self {
            crate::model::ComponentProperty::DateTimeStamp(property) => {
                writer.write(b"DTSTAMP")?;
                for param in &property.params {
                    writer.write(b";")?;
                    param.write_model(writer)?;
                }
                writer.write(b":")?;
                (property.date, property.time, property.is_utc).write_model(writer)?;
            }
            _ => {
                unimplemented!()
            }
        }

        Ok(())
    }
}
