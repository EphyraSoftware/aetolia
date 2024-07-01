use crate::convert::ToModel;
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
