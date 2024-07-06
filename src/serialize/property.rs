use crate::serialize::WriteModel;
use std::io::Write;

impl WriteModel for crate::model::ComponentProperty {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        match self {
            crate::model::ComponentProperty::DateTimeStamp(property) => {
                writer.write_all(b"DTSTAMP")?;
                for param in &property.params {
                    writer.write_all(b";")?;
                    param.write_model(writer)?;
                }
                writer.write_all(b":")?;
                (property.date, property.time, property.is_utc).write_model(writer)?;
            }
            _ => {
                unimplemented!()
            }
        }

        Ok(())
    }
}
