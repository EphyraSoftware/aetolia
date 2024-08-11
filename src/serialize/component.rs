use crate::serialize::WriteModel;
use std::io::Write;

impl WriteModel for crate::model::component::CalendarComponent {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        use crate::model::component::CalendarComponent;

        match self {
            CalendarComponent::Event(component) => {
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
            CalendarComponent::ToDo(component) => {
                writer.write_all(b"BEGIN:VTODO")?;
                for property in &component.properties {
                    writer.write_all(b"\r\n")?;
                    property.write_model(writer)?;
                }
                for alarm in &component.alarms {
                    writer.write_all(b"\r\n")?;
                    alarm.write_model(writer)?;
                }
                writer.write_all(b"\r\nEND:VTODO")?;
            }
            CalendarComponent::Journal(component) => {
                writer.write_all(b"BEGIN:VJOURNAL")?;
                for property in &component.properties {
                    writer.write_all(b"\r\n")?;
                    property.write_model(writer)?;
                }
                writer.write_all(b"\r\nEND:VJOURNAL")?;
            }
            CalendarComponent::FreeBusy(component) => {
                writer.write_all(b"BEGIN:VFREEBUSY")?;
                for property in &component.properties {
                    writer.write_all(b"\r\n")?;
                    property.write_model(writer)?;
                }
                writer.write_all(b"\r\nEND:VFREEBUSY")?;
            }
            CalendarComponent::TimeZone(component) => {
                writer.write_all(b"BEGIN:VTIMEZONE")?;
                for property in &component.properties {
                    writer.write_all(b"\r\n")?;
                    property.write_model(writer)?;
                }
                for component in &component.components {
                    writer.write_all(b"\r\n")?;
                    component.write_model(writer)?;
                }
                writer.write_all(b"\r\nEND:VTIMEZONE")?;
            }
            CalendarComponent::Standard(component) => {
                writer.write_all(b"BEGIN:STANDARD")?;
                for property in &component.properties {
                    writer.write_all(b"\r\n")?;
                    property.write_model(writer)?;
                }
                writer.write_all(b"\r\nEND:STANDARD")?;
            }
            CalendarComponent::Daylight(component) => {
                writer.write_all(b"BEGIN:DAYLIGHT")?;
                for property in &component.properties {
                    writer.write_all(b"\r\n")?;
                    property.write_model(writer)?;
                }
                writer.write_all(b"\r\nEND:DAYLIGHT")?;
            }
            CalendarComponent::Alarm(component) => {
                writer.write_all(b"BEGIN:VALARM")?;
                for property in &component.properties {
                    writer.write_all(b"\r\n")?;
                    property.write_model(writer)?;
                }
                writer.write_all(b"\r\nEND:VALARM")?;
            }
            CalendarComponent::IanaComponent(component) => {
                writer.write_all(b"BEGIN:")?;
                writer.write_all(component.name.as_bytes())?;
                for property in &component.properties {
                    writer.write_all(b"\r\n")?;
                    property.write_model(writer)?;
                }
                writer.write_all(b"\r\nEND:")?;
                writer.write_all(component.name.as_bytes())?;
            }
            CalendarComponent::XComponent(component) => {
                writer.write_all(b"BEGIN:")?;
                writer.write_all(component.name.as_bytes())?;
                for property in &component.properties {
                    writer.write_all(b"\r\n")?;
                    property.write_model(writer)?;
                }
                writer.write_all(b"\r\nEND:")?;
                writer.write_all(component.name.as_bytes())?;
            }
        }

        Ok(())
    }
}
