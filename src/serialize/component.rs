use crate::serialize::WriteModel;
use std::io::Write;

impl WriteModel for crate::model::CalendarComponent {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        match self {
            crate::model::CalendarComponent::Event(component) => {
                writer.write_all(b"BEGIN:VEVENT")?;
                for property in &component.properties {
                    writer.write_all(b"\r\n")?;
                    property.write_model(writer)?;
                }
                for alarm in &component.alarms {
                    writer.write_all(b"\r\n")?;
                    alarm.write_model(writer)?;
                }
                writer.write_all(b"\r\nEND:VEVENT")?;
            }
            _ => {
                unimplemented!()
            }
        }

        Ok(())
    }
}
