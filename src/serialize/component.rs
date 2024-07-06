use std::io::Write;
use crate::serialize::WriteModel;

impl WriteModel for crate::model::CalendarComponent {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        match self {
            crate::model::CalendarComponent::Event(component) => {
                writer.write(b"BEGIN:VEVENT")?;
                for property in &component.properties {
                    writer.write(b"\r\n")?;
                    property.write_model(writer)?;
                }
                for alarm in &component.alarms {
                    writer.write(b"\r\n")?;
                    alarm.write_model(writer)?;
                }
                writer.write(b"\r\nEND:VEVENT")?;
            }
            _ => {
                unimplemented!()
            }
        }

        Ok(())
    }
}
