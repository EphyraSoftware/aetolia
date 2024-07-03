mod recur;

use crate::convert::{convert_string, ToModel};
use crate::parser::DateOrDateTime;
use anyhow::Context;

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
