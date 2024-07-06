use crate::serialize::WriteModel;
use std::io::Write;

impl WriteModel for crate::model::ComponentProperty {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        use crate::model::ComponentProperty;

        match self {
            ComponentProperty::DateTimeStamp(property) => {
                writer.write_all(b"DTSTAMP")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                (property.date, property.time, property.is_utc).write_model(writer)?;
            }
            ComponentProperty::UniqueIdentifier(property) => {
                writer.write_all(b"UID")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                writer.write_all(property.value.as_bytes())?;
            }
            ComponentProperty::DateTimeStart(property) => {
                writer.write_all(b"DTSTART")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                (property.date, property.time, property.is_utc).write_model(writer)?;
            }
            ComponentProperty::Classification(property) => {
                writer.write_all(b"CLASS")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                property.value.write_model(writer)?;
            }
            ComponentProperty::DateTimeCreated(property) => {
                writer.write_all(b"CREATED")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                (property.date, property.time, property.is_utc).write_model(writer)?;
            }
            ComponentProperty::Description(property) => {
                writer.write_all(b"DESCRIPTION")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                writer.write_all(property.value.as_bytes())?;
            }
            ComponentProperty::GeographicPosition(property) => {
                writer.write_all(b"GEO")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                write!(writer, "{};", property.latitude)?;
                write!(writer, "{}", property.longitude)?;
            }
            ComponentProperty::LastModified(property) => {
                writer.write_all(b"LAST-MODIFIED")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                (property.date, property.time, property.is_utc).write_model(writer)?;
            }
            ComponentProperty::Location(property) => {
                writer.write_all(b"LOCATION")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                writer.write_all(property.value.as_bytes())?;
            }
            ComponentProperty::Organizer(property) => {
                writer.write_all(b"ORGANIZER")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                writer.write_all(property.value.as_bytes())?;
            }
            ComponentProperty::Priority(property) => {
                writer.write_all(b"PRIORITY")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                writer.write_all(&[property.value])?;
            }
            ComponentProperty::Sequence(property) => {
                writer.write_all(b"SEQUENCE")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                write!(writer, "{}", property.value)?;
            }
            ComponentProperty::Summary(property) => {
                writer.write_all(b"SUMMARY")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                writer.write_all(property.value.as_bytes())?;
            }
            ComponentProperty::TimeTransparency(property) => {
                writer.write_all(b"TRANSP")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                property.value.write_model(writer)?;
            }
            ComponentProperty::RequestStatus(property) => {
                writer.write_all(b"REQUEST-STATUS")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                if let Some(code) = property.status_code.first() {
                    write!(writer, "{}", code)?;
                }
                for code in property.status_code.iter().skip(1) {
                    write!(writer, ".{}", code)?;
                }
                writer.write_all(b";")?;
                writer.write_all(property.description.as_bytes())?;
                if let Some(exception_data) = &property.exception_data {
                    writer.write_all(b";")?;
                    writer.write_all(exception_data.as_bytes())?;
                }
            }
            ComponentProperty::Url(property) => {
                writer.write_all(b"URL")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                writer.write_all(property.value.as_bytes())?;
            }
            ComponentProperty::RecurrenceId(property) => {
                writer.write_all(b"RECURRENCE-ID")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                (property.date, property.time, property.is_utc).write_model(writer)?;
            }
            ComponentProperty::RecurrenceRule(property) => {
                writer.write_all(b"RRULE")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                property.rule.write_model(writer)?;
            }
            ComponentProperty::DateTimeEnd(property) => {
                writer.write_all(b"DTEND")?;
                property.params.as_slice().write_model(writer)?;
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

impl WriteModel for &[crate::model::Param] {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        if let Some(param) = self.first() {
            param.write_model(writer)?;
        }
        for param in self.iter().skip(1) {
            writer.write_all(b";")?;
            param.write_model(writer)?;
        }

        Ok(())
    }
}
