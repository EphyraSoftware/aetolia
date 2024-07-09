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
                property.value.write_model(writer)?;
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
                property.value.write_model(writer)?;
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
                property.value.write_model(writer)?;
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
                write!(writer, "{}", property.value)?;
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
                property.value.write_model(writer)?;
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
            ComponentProperty::Duration(property) => {
                writer.write_all(b"DURATION")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                property.duration.write_model(writer)?;
            }
            ComponentProperty::Attach(property) => {
                writer.write_all(b"ATTACH")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                writer.write_all(property.value.as_bytes())?;
            }
            ComponentProperty::Attendee(property) => {
                writer.write_all(b"ATTENDEE")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                writer.write_all(property.value.as_bytes())?;
            }
            ComponentProperty::Categories(property) => {
                writer.write_all(b"CATEGORIES")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                if let Some(category) = property.value.first() {
                    category.write_model(writer)?;
                }
                for category in property.value.iter().skip(1) {
                    writer.write_all(b",")?;
                    category.write_model(writer)?;
                }
            }
            ComponentProperty::Comment(property) => {
                writer.write_all(b"COMMENT")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                property.value.write_model(writer)?;
            }
            ComponentProperty::Contact(property) => {
                writer.write_all(b"CONTACT")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                property.value.write_model(writer)?;
            }
            ComponentProperty::ExceptionDateTimes(property) => {
                writer.write_all(b"EXDATE")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                if let Some((date, time, is_utc)) = property.date_times.first() {
                    (*date, *time, *is_utc).write_model(writer)?;
                }
                for (date, time, is_utc) in property.date_times.iter().skip(1) {
                    writer.write_all(b",")?;
                    (*date, *time, *is_utc).write_model(writer)?;
                }
            }
            ComponentProperty::Status(property) => {
                writer.write_all(b"STATUS")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                property.value.write_model(writer)?;
            }
            ComponentProperty::RelatedTo(property) => {
                writer.write_all(b"RELATED-TO")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                property.value.write_model(writer)?;
            }
            ComponentProperty::Resources(property) => {
                writer.write_all(b"RESOURCES")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                if let Some(resource) = property.value.first() {
                    resource.write_model(writer)?;
                }
                for resource in property.value.iter().skip(1) {
                    writer.write_all(b",")?;
                    resource.write_model(writer)?;
                }
            }
            ComponentProperty::RecurrenceDateTimes(property) => {
                writer.write_all(b"RDATE")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                if let Some((date, time, is_utc)) = property.date_times.first() {
                    (*date, *time, *is_utc).write_model(writer)?;
                }
                for (date, time, is_utc) in property.date_times.iter().skip(1) {
                    writer.write_all(b",")?;
                    (*date, *time, *is_utc).write_model(writer)?;
                }
            }
            ComponentProperty::DateTimeCompleted(property) => {
                writer.write_all(b"COMPLETED")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                (property.date, property.time, property.is_utc).write_model(writer)?;
            }
            ComponentProperty::PercentComplete(property) => {
                writer.write_all(b"PERCENT-COMPLETE")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                write!(writer, "{}", property.value)?;
            }
            ComponentProperty::DateTimeDue(property) => {
                writer.write_all(b"DUE")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                (property.date, property.time, property.is_utc).write_model(writer)?;
            }
            ComponentProperty::FreeBusyTime(property) => {
                writer.write_all(b"FREEBUSY")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                if let Some(period) = property.value.first() {
                    period.write_model(writer)?;
                }
                for period in property.value.iter().skip(1) {
                    writer.write_all(b",")?;
                    period.write_model(writer)?;
                }
            }
            ComponentProperty::TimeZoneId(property) => {
                writer.write_all(b"TZID")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                writer.write_all(property.value.as_bytes())?;
            }
            ComponentProperty::TimeZoneUrl(property) => {
                writer.write_all(b"TZURL")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                writer.write_all(property.value.as_bytes())?;
            }
            ComponentProperty::TimeZoneOffsetTo(property) => {
                writer.write_all(b"TZOFFSETTO")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                property.offset.write_model(writer)?;
            }
            ComponentProperty::TimeZoneOffsetFrom(property) => {
                writer.write_all(b"TZOFFSETFROM")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                property.offset.write_model(writer)?;
            }
            ComponentProperty::TimeZoneName(property) => {
                writer.write_all(b"TZNAME")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                property.value.write_model(writer)?;
            }
            ComponentProperty::Action(property) => {
                writer.write_all(b"ACTION")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                property.value.write_model(writer)?;
            }
            ComponentProperty::Trigger(property) => {
                writer.write_all(b"TRIGGER")?;
                match property {
                    crate::model::Trigger::Relative(property) => {
                        property.params.as_slice().write_model(writer)?;
                        writer.write_all(b":")?;
                        property.value.write_model(writer)?;
                    }
                    crate::model::Trigger::Absolute(property) => {
                        property.params.as_slice().write_model(writer)?;
                        writer.write_all(b":")?;
                        (property.date, property.time, property.is_utc).write_model(writer)?;
                    }
                }
            }
            ComponentProperty::Repeat(property) => {
                writer.write_all(b"REPEAT")?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                write!(writer, "{}", property.value)?;
            }
            ComponentProperty::IanaProperty(property) => {
                writer.write_all(property.name.as_bytes())?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                writer.write_all(property.value.as_bytes())?;
            }
            ComponentProperty::XProperty(property) => {
                writer.write_all(property.name.as_bytes())?;
                property.params.as_slice().write_model(writer)?;
                writer.write_all(b":")?;
                writer.write_all(property.value.as_bytes())?;
            }
        }

        Ok(())
    }
}

impl WriteModel for &[crate::model::Param] {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        for param in self.iter() {
            writer.write_all(b";")?;
            param.write_model(writer)?;
        }

        Ok(())
    }
}
