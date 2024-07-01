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
        let (date, maybe_time) = match &self.value {
            DateOrDateTime::Date(date) => (date.try_into()?, None),
            DateOrDateTime::DateTime(datetime) => {
                let (date, time) = datetime.try_into()?;
                (date, Some(time))
            }
        };

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

impl TryFrom<&crate::parser::Date> for time::Date {
    type Error = anyhow::Error;

    fn try_from(value: &crate::parser::Date) -> Result<Self, Self::Error> {
        Ok(time::Date::from_calendar_date(
            value.year as i32,
            time::Month::try_from(value.month).context("Invalid month")?,
            value.day,
        )
        .context("Invalid date")?)
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
