mod recur;

use crate::convert::{convert_string, ToModel};
use crate::model::Period;
use crate::parser::{DateOrDateTime, DateOrDateTimeOrPeriod};
use anyhow::Context;
use chrono::{Datelike, Timelike};

impl ToModel for crate::parser::DateTimeStampProperty<'_> {
    type Model = crate::model::DateTimeStampProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::DateTimeStampProperty {
            date: time::Date::from_calendar_date(
                self.value.date.year as i32,
                time::Month::try_from(self.value.date.month).context("Invalid month")?,
                self.value.date.day,
            )
            .context("Invalid date")?,
            time: time::Time::from_hms(
                self.value.time.hour,
                self.value.time.minute,
                self.value.time.second,
            )
            .context("Invalid time")?,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::UniqueIdentifierProperty<'_> {
    type Model = crate::model::UniqueIdentifierProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::UniqueIdentifierProperty {
            value: convert_string(&self.value),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::DateTimeStartProperty<'_> {
    type Model = crate::model::DateTimeStartProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let (date, maybe_time) = self.value.to_model()?;

        Ok(crate::model::DateTimeStartProperty {
            date,
            time: maybe_time,
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::ClassificationProperty<'_> {
    type Model = crate::model::ClassificationProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::ClassificationProperty {
            value: self.value.to_model()?,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::Classification<'_> {
    type Model = crate::model::Classification;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(match self {
            crate::parser::Classification::Public => crate::model::Classification::Public,
            crate::parser::Classification::Private => crate::model::Classification::Private,
            crate::parser::Classification::Confidential => {
                crate::model::Classification::Confidential
            }
            crate::parser::Classification::XName(name) => {
                crate::model::Classification::XName(convert_string(name))
            }
            crate::parser::Classification::IanaToken(token) => {
                crate::model::Classification::IanaToken(convert_string(token))
            }
        })
    }
}

impl ToModel for crate::parser::CreatedProperty<'_> {
    type Model = crate::model::CreatedProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let (date, time) = (&self.value).try_into()?;

        Ok(crate::model::CreatedProperty {
            date,
            time,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::DescriptionProperty<'_> {
    type Model = crate::model::DescriptionProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::DescriptionProperty {
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::GeographicPositionProperty<'_> {
    type Model = crate::model::GeographicPositionProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::GeographicPositionProperty {
            latitude: self.latitude,
            longitude: self.longitude,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::LastModifiedProperty<'_> {
    type Model = crate::model::LastModifiedProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let (date, time) = (&self.value).try_into()?;

        Ok(crate::model::LastModifiedProperty {
            date,
            time,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::LocationProperty<'_> {
    type Model = crate::model::LocationProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::LocationProperty {
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::OrganizerProperty<'_> {
    type Model = crate::model::OrganizerProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::OrganizerProperty {
            value: convert_string(self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::PriorityProperty<'_> {
    type Model = crate::model::PriorityProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::PriorityProperty {
            value: self.value,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::SequenceProperty<'_> {
    type Model = crate::model::SequenceProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::SequenceProperty {
            value: self.value,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::StatusProperty<'_> {
    type Model = crate::model::StatusProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::StatusProperty {
            value: self.value.clone(),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::SummaryProperty<'_> {
    type Model = crate::model::SummaryProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::SummaryProperty {
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::TimeTransparencyProperty<'_> {
    type Model = crate::model::TimeTransparencyProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::TimeTransparencyProperty {
            value: self.value.clone(),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::UrlProperty<'_> {
    type Model = crate::model::UrlProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::UrlProperty {
            value: self.value.to_string(),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::RecurrenceIdProperty<'_> {
    type Model = crate::model::RecurrenceIdProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let (date, maybe_time) = self.value.to_model()?;

        Ok(crate::model::RecurrenceIdProperty {
            date,
            time: maybe_time,
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::RecurrenceRuleProperty<'_> {
    type Model = crate::model::RecurrenceRuleProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::RecurrenceRuleProperty {
            rule: self.value.to_model()?,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::DateTimeEndProperty<'_> {
    type Model = crate::model::DateTimeEndProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let (date, maybe_time) = self.value.to_model()?;

        Ok(crate::model::DateTimeEndProperty {
            date,
            time: maybe_time,
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::DurationProperty<'_> {
    type Model = crate::model::DurationProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::DurationProperty {
            duration: self.value.to_model()?,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::AttachProperty<'_> {
    type Model = crate::model::AttachProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let (uri, binary) = match self.value {
            crate::parser::AttachValue::Uri(uri) => (Some(convert_string(uri)), None),
            crate::parser::AttachValue::Binary(binary) => (None, Some(convert_string(binary))),
        };

        Ok(crate::model::AttachProperty {
            value_uri: uri,
            value_binary: binary,
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::AttendeeProperty<'_> {
    type Model = crate::model::AttendeeProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::AttendeeProperty {
            value: convert_string(self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::CategoriesProperty<'_> {
    type Model = crate::model::CategoriesProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::CategoriesProperty {
            value: self.value.iter().map(|v| convert_string(v)).collect(),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::CommentProperty<'_> {
    type Model = crate::model::CommentProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::CommentProperty {
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::ContactProperty<'_> {
    type Model = crate::model::ContactProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::ContactProperty {
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::ExceptionDateTimesProperty<'_> {
    type Model = crate::model::ExceptionDateTimesProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::ExceptionDateTimesProperty {
            date_times: self
                .value
                .iter()
                .map(|v| v.to_model())
                .collect::<Result<_, _>>()?,
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::RequestStatusProperty<'_> {
    type Model = crate::model::RequestStatusProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::RequestStatusProperty {
            status_code: self.status_code.clone(),
            description: convert_string(&self.status_description),
            exception_data: self.exception_data.as_ref().map(|v| convert_string(v)),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::RelatedToProperty<'_> {
    type Model = crate::model::RelatedToProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::RelatedToProperty {
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::ResourcesProperty<'_> {
    type Model = crate::model::ResourcesProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::ResourcesProperty {
            value: self.value.iter().map(|v| convert_string(v)).collect(),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::XProperty<'_> {
    type Model = crate::model::XProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::XProperty {
            name: convert_string(self.name),
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::IanaProperty<'_> {
    type Model = crate::model::IanaProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::IanaProperty {
            name: convert_string(self.name),
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::RecurrenceDateTimesProperty<'_> {
    type Model = crate::model::RecurrenceDateTimesProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let date_times = self.value.to_model()?;

        let (date_times, periods) = if date_times.iter().all(|dt| dt.0.is_some()) {
            (
                date_times.iter().map(|dt| dt.0.unwrap()).collect(),
                Vec::with_capacity(0),
            )
        } else if date_times.iter().all(|dt| dt.1.is_some()) {
            (
                Vec::with_capacity(0),
                date_times.iter().map(|dt| dt.1.clone().unwrap()).collect(),
            )
        } else {
            return Err(anyhow::anyhow!("Invalid recurrence date-times"));
        };

        Ok(crate::model::RecurrenceDateTimesProperty {
            date_times,
            periods,
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::Duration {
    type Model = crate::model::Duration;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::Duration {
            sign: self.sign,
            weeks: self.weeks,
            days: self.days,
            hours: self.hours,
            minutes: self.minutes,
            seconds: self.seconds,
        })
    }
}

impl ToModel for DateOrDateTimeOrPeriod<'_> {
    type Model = (Option<(time::Date, Option<time::Time>)>, Option<Period>);

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        match self {
            DateOrDateTimeOrPeriod::Date(date) => Ok((Some((date.try_into()?, None)), None)),
            DateOrDateTimeOrPeriod::DateTime(date_time) => {
                let (date, time) = date_time.try_into()?;
                Ok((Some((date, Some(time))), None))
            }
            DateOrDateTimeOrPeriod::Period(period) => Ok((None, Some(period.to_model()?))),
        }
    }
}

impl ToModel for crate::parser::Period<'_> {
    type Model = Period;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(Period {
            start: convert_date_time_iso8601(self.start)?,
            end: match &self.end {
                crate::parser::PeriodEnd::DateTime(date_time) => {
                    crate::model::PeriodEnd::DateTime(convert_date_time_iso8601(date_time)?)
                }
                crate::parser::PeriodEnd::Duration(duration) => {
                    crate::model::PeriodEnd::Duration(duration.to_model()?)
                }
            },
        })
    }
}

impl ToModel for DateOrDateTime {
    type Model = (time::Date, Option<time::Time>);

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(match self {
            DateOrDateTime::Date(date) => (date.try_into()?, None),
            DateOrDateTime::DateTime(datetime) => {
                let (date, time) = datetime.try_into()?;
                (date, Some(time))
            }
        })
    }
}

impl TryFrom<&crate::parser::Date> for time::Date {
    type Error = anyhow::Error;

    fn try_from(value: &crate::parser::Date) -> Result<Self, Self::Error> {
        time::Date::from_calendar_date(
            value.year as i32,
            time::Month::try_from(value.month).context("Invalid month")?,
            value.day,
        )
        .context("Invalid date")
    }
}

impl TryFrom<&crate::parser::DateTime> for (time::Date, time::Time) {
    type Error = anyhow::Error;

    fn try_from(value: &crate::parser::DateTime) -> Result<Self, Self::Error> {
        Ok((
            time::Date::try_from(&value.date)?,
            time::Time::from_hms(value.time.hour, value.time.minute, value.time.second)
                .context("Invalid time")?,
        ))
    }
}

fn convert_date_time_iso8601(raw: &[u8]) -> anyhow::Result<(time::Date, time::Time)> {
    let date_time = chrono::DateTime::parse_from_rfc3339(
        std::str::from_utf8(raw).context("Invalid date string")?,
    )
    .context("Date failed to parse")?;

    Ok((
        time::Date::from_calendar_date(
            date_time.year(),
            time::Month::try_from(date_time.month() as u8).context("Should be a valid month")?,
            date_time.day() as u8,
        )
        .context("Invalid date")?,
        time::Time::from_hms(
            date_time.hour() as u8,
            date_time.minute() as u8,
            date_time.second() as u8,
        )
        .context("Invalid time")?,
    ))
}
