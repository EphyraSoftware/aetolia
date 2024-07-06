use crate::common::LanguageTag;
use crate::serialize::WriteModel;
use std::io::Write;
use std::ops::Add;

impl WriteModel for (time::Date, time::Time, bool) {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        self.0.write_model(writer)?;
        writer.write_all(b"T")?;
        self.1.write_model(writer)?;
        if self.2 {
            writer.write_all(b"Z")?;
        }

        Ok(())
    }
}

impl WriteModel for (time::Date, Option<time::Time>, bool) {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        match self.1 {
            Some(time) => {
                (self.0, time, self.2).write_model(writer)?;
            }
            None => {
                self.0.write_model(writer)?;
            }
        }

        Ok(())
    }
}

impl WriteModel for time::Date {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        let year = self.year();
        if (0..10).contains(&year) {
            writer.write_all(&[0, 0, 0, year as u8])?;
        } else if (10..100).contains(&year) {
            writer.write_all(&[0, 0, year as u8 / 10, year as u8 % 10])?;
        } else if (100..1000).contains(&year) {
            writer.write_all(&[0, year as u8 / 100, year as u8 / 10 % 10, year as u8 % 10])?;
        } else if (1000..10000).contains(&year) {
            write!(writer, "{}", year)?;
        } else {
            return Err(anyhow::anyhow!("Year [{year}] out of range"));
        }

        match self.month() {
            m @ time::Month::October | m @ time::Month::November | m @ time::Month::December => {
                write!(writer, "{}", m as u8)?;
            }
            m => {
                write!(writer, "0{}", m as u8)?;
            }
        }

        match self.day() {
            d @ 10..=31 => {
                write!(writer, "{}", d)?;
            }
            d => {
                write!(writer, "0{}", d)?;
            }
        }

        Ok(())
    }
}

impl WriteModel for time::Time {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        match self.hour() {
            h @ 10..=23 => {
                write!(writer, "{}", h)?;
            }
            h => {
                write!(writer, "0{}", h)?;
            }
        }

        match self.minute() {
            m @ 10..=59 => {
                write!(writer, "{}", m)?;
            }
            m => {
                write!(writer, "0{}", m)?;
            }
        }

        match self.second() {
            s @ 10..=60 => {
                write!(writer, "{}", s)?;
            }
            s => {
                write!(writer, "0{}", s)?;
            }
        }

        Ok(())
    }
}

impl WriteModel for crate::common::Value {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        use crate::common::Value;

        match self {
            Value::Binary => {
                writer.write_all(b"BINARY")?;
            }
            Value::Boolean => {
                writer.write_all(b"BOOLEAN")?;
            }
            Value::CalendarAddress => {
                writer.write_all(b"CALENDAR-ADDRESS")?;
            }
            Value::Date => {
                writer.write_all(b"DATE")?;
            }
            Value::DateTime => {
                writer.write_all(b"DATE-TIME")?;
            }
            Value::Duration => {
                writer.write_all(b"DURATION")?;
            }
            Value::Float => {
                writer.write_all(b"FLOAT")?;
            }
            Value::Integer => {
                writer.write_all(b"INTEGER")?;
            }
            Value::Period => {
                writer.write_all(b"PERIOD")?;
            }
            Value::Recurrence => {
                writer.write_all(b"RECUR")?;
            }
            Value::Text => {
                writer.write_all(b"TEXT")?;
            }
            Value::Time => {
                writer.write_all(b"TIME")?;
            }
            Value::Uri => {
                writer.write_all(b"URI")?;
            }
            Value::UtcOffset => {
                writer.write_all(b"UTC-OFFSET")?;
            }
            Value::XName(name) => {
                write!(writer, "{}", name)?;
            }
            Value::IanaToken(token) => {
                write!(writer, "{}", token)?;
            }
        }

        Ok(())
    }
}

impl WriteModel for LanguageTag {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_all(self.language.as_bytes())?;
        if let Some(ext_lang) = &self.ext_lang {
            writer.write_all(b"-")?;
            writer.write_all(ext_lang.as_bytes())?;
        }
        if let Some(script) = &self.script {
            writer.write_all(b"-")?;
            writer.write_all(script.as_bytes())?;
        }
        if let Some(region) = &self.region {
            writer.write_all(b"-")?;
            writer.write_all(region.as_bytes())?;
        }
        for variant in &self.variants {
            writer.write_all(b"-")?;
            writer.write_all(variant.as_bytes())?;
        }
        for extension in &self.extensions {
            writer.write_all(b"-")?;
            writer.write_all(extension.as_bytes())?;
        }
        if let Some(private_use) = &self.private_use {
            writer.write_all(b"-")?;
            writer.write_all(private_use.as_bytes())?;
        }
        Ok(())
    }
}

impl WriteModel for crate::common::Range {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        use crate::common::Range;

        match self {
            Range::ThisAndFuture => {
                writer.write_all(b"THISANDFUTURE")?;
            }
        }

        Ok(())
    }
}

impl WriteModel for crate::common::Encoding {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        use crate::common::Encoding;

        match self {
            Encoding::EightBit => {
                writer.write_all(b"8BIT")?;
            }
            Encoding::Base64 => {
                writer.write_all(b"BASE64")?;
            }
        }

        Ok(())
    }
}

impl WriteModel for crate::common::CalendarUserType {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        use crate::common::CalendarUserType;

        match self {
            CalendarUserType::Individual => {
                writer.write_all(b"INDIVIDUAL")?;
            }
            CalendarUserType::Group => {
                writer.write_all(b"GROUP")?;
            }
            CalendarUserType::Resource => {
                writer.write_all(b"RESOURCE")?;
            }
            CalendarUserType::Room => {
                writer.write_all(b"ROOM")?;
            }
            CalendarUserType::Unknown => {
                writer.write_all(b"UNKNOWN")?;
            }
            CalendarUserType::XName(name) => {
                writer.write_all(name.as_bytes())?;
            }
            CalendarUserType::IanaToken(token) => {
                writer.write_all(token.as_bytes())?;
            }
        }

        Ok(())
    }
}

impl WriteModel for crate::common::Role {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        use crate::common::Role;

        match self {
            Role::Chair => {
                writer.write_all(b"CHAIR")?;
            }
            Role::RequiredParticipant => {
                writer.write_all(b"REQ-PARTICIPANT")?;
            }
            Role::OptionalParticipant => {
                writer.write_all(b"OPT-PARTICIPANT")?;
            }
            Role::NonParticipant => {
                writer.write_all(b"NON-PARTICIPANT")?;
            }
            Role::XName(name) => {
                writer.write_all(name.as_bytes())?;
            }
            Role::IanaToken(token) => {
                writer.write_all(token.as_bytes())?;
            }
        }

        Ok(())
    }
}

impl WriteModel for crate::common::ParticipationStatusUnknown {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        use crate::common::ParticipationStatusUnknown;

        match self {
            ParticipationStatusUnknown::NeedsAction => {
                writer.write_all(b"NEEDS-ACTION")?;
            }
            ParticipationStatusUnknown::Accepted => {
                writer.write_all(b"ACCEPTED")?;
            }
            ParticipationStatusUnknown::Declined => {
                writer.write_all(b"DECLINED")?;
            }
            ParticipationStatusUnknown::Tentative => {
                writer.write_all(b"TENTATIVE")?;
            }
            ParticipationStatusUnknown::Delegated => {
                writer.write_all(b"DELEGATED")?;
            }
            ParticipationStatusUnknown::Completed => {
                writer.write_all(b"COMPLETED")?;
            }
            ParticipationStatusUnknown::InProcess => {
                writer.write_all(b"IN-PROCESS")?;
            }
            ParticipationStatusUnknown::XName(name) => {
                writer.write_all(name.as_bytes())?;
            }
            ParticipationStatusUnknown::IanaToken(token) => {
                writer.write_all(token.as_bytes())?;
            }
        }

        Ok(())
    }
}

impl WriteModel for bool {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        if *self {
            writer.write_all(b"TRUE")?;
        } else {
            writer.write_all(b"FALSE")?;
        }

        Ok(())
    }
}

impl WriteModel for crate::common::RelationshipType {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        use crate::common::RelationshipType;

        match self {
            RelationshipType::Parent => {
                writer.write_all(b"PARENT")?;
            }
            RelationshipType::Child => {
                writer.write_all(b"CHILD")?;
            }
            RelationshipType::Sibling => {
                writer.write_all(b"SIBLING")?;
            }
            RelationshipType::XName(name) => {
                writer.write_all(name.as_bytes())?;
            }
            RelationshipType::IanaToken(token) => {
                writer.write_all(token.as_bytes())?;
            }
        }

        Ok(())
    }
}

impl WriteModel for crate::common::FreeBusyTimeType {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        use crate::common::FreeBusyTimeType;

        match self {
            FreeBusyTimeType::Free => {
                writer.write_all(b"FREE")?;
            }
            FreeBusyTimeType::Busy => {
                writer.write_all(b"BUSY")?;
            }
            FreeBusyTimeType::BusyUnavailable => {
                writer.write_all(b"BUSY-UNAVAILABLE")?;
            }
            FreeBusyTimeType::BusyTentative => {
                writer.write_all(b"BUSY-TENTATIVE")?;
            }
            FreeBusyTimeType::XName(name) => {
                writer.write_all(name.as_bytes())?;
            }
            FreeBusyTimeType::IanaToken(token) => {
                writer.write_all(token.as_bytes())?;
            }
        }

        Ok(())
    }
}

impl WriteModel for crate::common::Related {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        use crate::common::Related;

        match self {
            Related::Start => {
                writer.write_all(b"START")?;
            }
            Related::End => {
                writer.write_all(b"END")?;
            }
        }

        Ok(())
    }
}

impl WriteModel for crate::model::Classification {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        use crate::model::Classification;

        match self {
            Classification::Public => {
                writer.write_all(b"PUBLIC")?;
            }
            Classification::Private => {
                writer.write_all(b"PRIVATE")?;
            }
            Classification::Confidential => {
                writer.write_all(b"CONFIDENTIAL")?;
            }
            Classification::XName(name) => {
                writer.write_all(name.as_bytes())?;
            }
            Classification::IanaToken(token) => {
                writer.write_all(token.as_bytes())?;
            }
        }

        Ok(())
    }
}

impl WriteModel for crate::common::TimeTransparency {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        use crate::common::TimeTransparency;

        match self {
            TimeTransparency::Opaque => {
                writer.write_all(b"OPAQUE")?;
            }
            TimeTransparency::Transparent => {
                writer.write_all(b"TRANSPARENT")?;
            }
        }

        Ok(())
    }
}

impl WriteModel for crate::model::RecurrenceRule {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_all(b"FREQ=")?;
        self.freq.write_model(writer)?;

        if let Some(until) = &self.until {
            writer.write_all(b";UNTIL=")?;
            until.write_model(writer)?;
        }

        if let Some(count) = &self.count {
            write!(writer, ";COUNT={}", count)?;
        }

        if let Some(interval) = &self.interval {
            write!(writer, ";INTERVAL={}", interval)?;
        }

        if let Some(by_second) = &self.by_second {
            write!(writer, ";BYSECOND=")?;
            by_second.write_model(writer)?;
        }

        if let Some(by_minute) = &self.by_minute {
            write!(writer, ";BYMINUTE=")?;
            by_minute.write_model(writer)?;
        }

        if let Some(by_hour) = &self.by_hour {
            write!(writer, ";BYHOUR=")?;
            by_hour.write_model(writer)?;
        }

        if let Some(by_day) = &self.by_day {
            write!(writer, ";BYDAY=")?;
            if let Some(day) = by_day.first() {
                day.write_model(writer)?;
            }
            for day in by_day.iter().skip(1) {
                write!(writer, ",")?;
                day.write_model(writer)?;
            }
        }

        if let Some(by_month_day) = &self.by_month_day {
            write!(writer, ";BYMONTHDAY=")?;
            by_month_day.write_model(writer)?;
        }

        if let Some(by_year_day) = &self.by_year_day {
            write!(writer, ";BYYEARDAY=")?;
            by_year_day.write_model(writer)?;
        }

        if let Some(by_week_number) = &self.by_week_number {
            write!(writer, ";BYWEEKNO=")?;
            by_week_number.write_model(writer)?;
        }

        if let Some(by_month) = &self.by_month {
            write!(writer, ";BYMONTH=")?;
            if let Some(month) = by_month.first() {
                month.write_model(writer)?;
            }
            for month in by_month.iter().skip(1) {
                write!(writer, ",")?;
                month.write_model(writer)?;
            }
        }

        if let Some(by_set_pos) = &self.by_set_pos {
            write!(writer, ";BYSETPOS=")?;
            by_set_pos.write_model(writer)?;
        }

        if let Some(week_start) = &self.week_start {
            write!(writer, ";WKST=")?;
            (*week_start).write_model(writer)?;
        }

        Ok(())
    }
}

impl WriteModel for crate::common::RecurFreq {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        use crate::common::RecurFreq;

        match self {
            RecurFreq::Secondly => {
                writer.write_all(b"SECONDLY")?;
            }
            RecurFreq::Minutely => {
                writer.write_all(b"MINUTELY")?;
            }
            RecurFreq::Hourly => {
                writer.write_all(b"HOURLY")?;
            }
            RecurFreq::Daily => {
                writer.write_all(b"DAILY")?;
            }
            RecurFreq::Weekly => {
                writer.write_all(b"WEEKLY")?;
            }
            RecurFreq::Monthly => {
                writer.write_all(b"MONTHLY")?;
            }
            RecurFreq::Yearly => {
                writer.write_all(b"YEARLY")?;
            }
        }

        Ok(())
    }
}

impl<T: Add<Output = T> + std::fmt::Display> WriteModel for Vec<T> {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        if let Some(value) = self.first() {
            write!(writer, "{}", value)?;
        }
        for value in self.iter().skip(1) {
            write!(writer, ",{}", value)?;
        }

        Ok(())
    }
}

impl WriteModel for crate::common::OffsetWeekday {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        use crate::common::OffsetWeekday;

        if let Some(offset_weeks) = &self.offset_weeks {
            write!(writer, "{}", offset_weeks)?;
        }

        self.weekday.write_model(writer)?;

        Ok(())
    }
}

impl WriteModel for crate::common::Weekday {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        use crate::common::Weekday;

        match self {
            Weekday::Sunday => {
                writer.write_all(b"SU")?;
            }
            Weekday::Monday => {
                writer.write_all(b"MO")?;
            }
            Weekday::Tuesday => {
                writer.write_all(b"TU")?;
            }
            Weekday::Wednesday => {
                writer.write_all(b"WE")?;
            }
            Weekday::Thursday => {
                writer.write_all(b"TH")?;
            }
            Weekday::Friday => {
                writer.write_all(b"FR")?;
            }
            Weekday::Saturday => {
                writer.write_all(b"SA")?;
            }
        }

        Ok(())
    }
}

impl WriteModel for time::Month {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        write!(writer, "{}", (*self) as u8)?;

        Ok(())
    }
}