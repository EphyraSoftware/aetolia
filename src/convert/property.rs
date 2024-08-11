mod recur;

use crate::common::CalendarDateTime;
use crate::convert::{convert_string, ToModel};
use crate::model::{
    GeographicPositionPropertyValue, Period, RecurrenceDateTimesPropertyValue,
    TimeZoneIdPropertyValue,
};
use crate::parser::ContentLine;
use crate::prelude::{RequestStatusPropertyValue, TriggerValue};
use anyhow::Context;

impl ToModel for crate::parser::types::DateTimeStampProperty<'_> {
    type Model = crate::model::DateTimeStampProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::DateTimeStampProperty {
            value: (
                time::Date::from_calendar_date(
                    self.value.date.year as i32,
                    time::Month::try_from(self.value.date.month).context("Invalid month")?,
                    self.value.date.day,
                )
                .context("Invalid date")?,
                time::Time::from_hms(
                    self.value.time.hour,
                    self.value.time.minute,
                    self.value.time.second,
                )
                .context("Invalid time")?,
                self.value.time.is_utc,
            )
                .into(),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::UniqueIdentifierProperty<'_> {
    type Model = crate::model::UniqueIdentifierProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::UniqueIdentifierProperty {
            value: convert_string(&self.value),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::DateTimeStartProperty<'_> {
    type Model = crate::model::DateTimeStartProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let dt = self.value.to_model()?;

        Ok(crate::model::DateTimeStartProperty {
            value: dt,
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::ClassificationProperty<'_> {
    type Model = crate::model::ClassificationProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::ClassificationProperty {
            value: self.value.to_model()?,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::Classification<'_> {
    type Model = crate::model::Classification;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(match self {
            crate::parser::types::Classification::Public => crate::model::Classification::Public,
            crate::parser::types::Classification::Private => crate::model::Classification::Private,
            crate::parser::types::Classification::Confidential => {
                crate::model::Classification::Confidential
            }
            crate::parser::types::Classification::XName(name) => {
                crate::model::Classification::XName(convert_string(name))
            }
            crate::parser::types::Classification::IanaToken(token) => {
                crate::model::Classification::IanaToken(convert_string(token))
            }
        })
    }
}

impl ToModel for crate::parser::types::CreatedProperty<'_> {
    type Model = crate::model::CreatedProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let (date, time, is_utc) = (&self.value).try_into()?;

        Ok(crate::model::CreatedProperty {
            value: (date, time, is_utc).into(),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::DescriptionProperty<'_> {
    type Model = crate::model::DescriptionProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::DescriptionProperty {
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::GeographicPositionProperty<'_> {
    type Model = crate::model::GeographicPositionProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::GeographicPositionProperty {
            value: GeographicPositionPropertyValue {
                latitude: self.latitude,
                longitude: self.longitude,
            },
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::LastModifiedProperty<'_> {
    type Model = crate::model::LastModifiedProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let (date, time, is_utc) = (&self.value).try_into()?;

        Ok(crate::model::LastModifiedProperty {
            value: (date, time, is_utc).into(),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::LocationProperty<'_> {
    type Model = crate::model::LocationProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::LocationProperty {
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::OrganizerProperty<'_> {
    type Model = crate::model::OrganizerProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::OrganizerProperty {
            value: convert_string(self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::PriorityProperty<'_> {
    type Model = crate::model::PriorityProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::PriorityProperty {
            value: self.value,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::SequenceProperty<'_> {
    type Model = crate::model::SequenceProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::SequenceProperty {
            value: self.value,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::StatusProperty<'_> {
    type Model = crate::model::StatusProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::StatusProperty {
            value: self.value.clone(),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::SummaryProperty<'_> {
    type Model = crate::model::SummaryProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::SummaryProperty {
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::TimeTransparencyProperty<'_> {
    type Model = crate::model::TimeTransparencyProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::TimeTransparencyProperty {
            value: self.value.clone(),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::UrlProperty<'_> {
    type Model = crate::model::UrlProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::UrlProperty {
            value: self.value.to_string(),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::RecurrenceIdProperty<'_> {
    type Model = crate::model::RecurrenceIdProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let dt = self.value.to_model()?;

        Ok(crate::model::RecurrenceIdProperty {
            value: dt,
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::RecurrenceRuleProperty<'_> {
    type Model = crate::model::RecurrenceRuleProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::RecurrenceRuleProperty {
            value: self.value.to_model()?,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::DateTimeEndProperty<'_> {
    type Model = crate::model::DateTimeEndProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let dt = self.value.to_model()?;

        Ok(crate::model::DateTimeEndProperty {
            value: dt,
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::DurationProperty<'_> {
    type Model = crate::model::DurationProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::DurationProperty {
            value: self.value.to_model()?,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::AttachProperty<'_> {
    type Model = crate::model::AttachProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let value = match self.value {
            crate::parser::types::AttachValue::Uri(uri) => convert_string(uri),
            crate::parser::types::AttachValue::Binary(binary) => convert_string(binary),
        };

        Ok(crate::model::AttachProperty {
            value,
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::AttendeeProperty<'_> {
    type Model = crate::model::AttendeeProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::AttendeeProperty {
            value: convert_string(self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::CategoriesProperty<'_> {
    type Model = crate::model::CategoriesProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::CategoriesProperty {
            value: self.value.iter().map(|v| convert_string(v)).collect(),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::CommentProperty<'_> {
    type Model = crate::model::CommentProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::CommentProperty {
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::ContactProperty<'_> {
    type Model = crate::model::ContactProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::ContactProperty {
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::ExceptionDateTimesProperty<'_> {
    type Model = crate::model::ExceptionDateTimesProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::ExceptionDateTimesProperty {
            value: self
                .value
                .iter()
                .map(|v| v.to_model())
                .collect::<Result<_, _>>()?,
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::RequestStatusProperty<'_> {
    type Model = crate::model::RequestStatusProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::RequestStatusProperty {
            value: RequestStatusPropertyValue {
                status_code: self.status_code.clone(),
                description: convert_string(&self.status_description),
                exception_data: self.exception_data.as_ref().map(|v| convert_string(v)),
            },
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::RelatedToProperty<'_> {
    type Model = crate::model::RelatedToProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::RelatedToProperty {
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::ResourcesProperty<'_> {
    type Model = crate::model::ResourcesProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::ResourcesProperty {
            value: self.value.iter().map(|v| convert_string(v)).collect(),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::XProperty<'_> {
    type Model = crate::model::XProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::XProperty {
            name: convert_string(self.name),
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::IanaProperty<'_> {
    type Model = crate::model::IanaProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::IanaProperty {
            name: convert_string(self.name),
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::RecurrenceDateTimesProperty<'_> {
    type Model = crate::model::RecurrenceDateTimesProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let date_times = self.value.to_model()?;

        let (date_times, periods) = if date_times.iter().all(|dt| dt.0.is_some()) {
            (
                date_times.iter().map(|dt| dt.0.unwrap().into()).collect(),
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
            value: if !periods.is_empty() {
                RecurrenceDateTimesPropertyValue::Periods(periods)
            } else {
                RecurrenceDateTimesPropertyValue::DateTimes(date_times)
            },
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

impl ToModel for crate::parser::types::DateOrDateTimeOrPeriod {
    type Model = (
        Option<(time::Date, Option<time::Time>, bool)>,
        Option<Period>,
    );

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        match self {
            crate::parser::types::DateOrDateTimeOrPeriod::Date(date) => {
                Ok((Some((date.try_into()?, None, false)), None))
            }
            crate::parser::types::DateOrDateTimeOrPeriod::DateTime(date_time) => {
                let (date, time, is_utc) = date_time.try_into()?;
                Ok((Some((date, Some(time), is_utc)), None))
            }
            crate::parser::types::DateOrDateTimeOrPeriod::Period(period) => {
                Ok((None, Some(period.to_model()?)))
            }
        }
    }
}

impl ToModel for crate::parser::types::ProductIdProperty<'_> {
    type Model = crate::model::ProductIdProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::ProductIdProperty {
            value: convert_string(&self.value),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::VersionProperty<'_> {
    type Model = crate::model::VersionProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::VersionProperty {
            min_version: self.min_version.map(convert_string),
            max_version: convert_string(self.max_version),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::CalendarScaleProperty<'_> {
    type Model = crate::model::CalendarScaleProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::CalendarScaleProperty {
            value: convert_string(&self.value),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::MethodProperty<'_> {
    type Model = crate::model::MethodProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::MethodProperty {
            value: convert_string(self.value),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::CalendarProperty<'_> {
    type Model = crate::model::CalendarProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        match self {
            crate::parser::CalendarProperty::ProductId(product_id) => Ok(
                crate::model::CalendarProperty::ProductId(product_id.to_model()?),
            ),
            crate::parser::CalendarProperty::Version(version) => {
                Ok(crate::model::CalendarProperty::Version(version.to_model()?))
            }
            crate::parser::CalendarProperty::CalendarScale(cal_scale) => Ok(
                crate::model::CalendarProperty::CalendarScale(cal_scale.to_model()?),
            ),
            crate::parser::CalendarProperty::Method(method) => {
                Ok(crate::model::CalendarProperty::Method(method.to_model()?))
            }
            crate::parser::CalendarProperty::XProperty(x_prop) => Ok(
                crate::model::CalendarProperty::XProperty(x_prop.to_model()?),
            ),
            crate::parser::CalendarProperty::IanaProperty(iana_prop) => Ok(
                crate::model::CalendarProperty::IanaProperty(iana_prop.to_model()?),
            ),
        }
    }
}

impl ToModel for crate::parser::types::DateTimeCompletedProperty<'_> {
    type Model = crate::model::DateTimeCompletedProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let (date, time, is_utc) = (&self.value).try_into()?;

        Ok(crate::model::DateTimeCompletedProperty {
            value: (date, time, is_utc).into(),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::PercentCompleteProperty<'_> {
    type Model = crate::model::PercentCompleteProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::PercentCompleteProperty {
            value: self.value,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::DateTimeDueProperty<'_> {
    type Model = crate::model::DateTimeDueProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let dt = self.value.to_model()?;

        Ok(crate::model::DateTimeDueProperty {
            value: dt,
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::FreeBusyTimeProperty<'_> {
    type Model = crate::model::FreeBusyTimeProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::FreeBusyTimeProperty {
            value: self.value.to_model()?,
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::TimeZoneIdProperty<'_> {
    type Model = crate::model::TimeZoneIdProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::TimeZoneIdProperty {
            value: TimeZoneIdPropertyValue {
                id: convert_string(&self.value),
                unique_registry_id: self.unique_registry_id,
            },
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::TimeZoneUrlProperty<'_> {
    type Model = crate::model::TimeZoneUrlProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::TimeZoneUrlProperty {
            value: convert_string(self.value),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::TimeZoneOffsetProperty<'_> {
    type Model = crate::model::TimeZoneOffsetToProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::TimeZoneOffsetToProperty {
            value: self.value.to_model()?,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::UtcOffset {
    type Model = crate::model::TimeZoneOffset;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::TimeZoneOffset {
            sign: self.sign,
            hours: self.hours as u8,
            minutes: self.minutes as u8,
            seconds: self.seconds.map(|s| s as u8),
        })
    }
}

impl ToModel for crate::parser::types::TimeZoneNameProperty<'_> {
    type Model = crate::model::TimeZoneNameProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::TimeZoneNameProperty {
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::ActionProperty<'_> {
    type Model = crate::model::ActionProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::ActionProperty {
            value: self.value.to_model()?,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::Action<'_> {
    type Model = crate::model::Action;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(match self {
            crate::parser::types::Action::Audio => crate::model::Action::Audio,
            crate::parser::types::Action::Display => crate::model::Action::Display,
            crate::parser::types::Action::Email => crate::model::Action::Email,
            crate::parser::types::Action::XName(name) => {
                crate::model::Action::XName(convert_string(name))
            }
            crate::parser::types::Action::IanaToken(token) => {
                crate::model::Action::IanaToken(convert_string(token))
            }
        })
    }
}

impl ToModel for crate::parser::types::TriggerProperty<'_> {
    type Model = crate::model::TriggerProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        match &self.value {
            crate::parser::types::DurationOrDateTime::DateTime(date_time) => {
                let (date, time, is_utc) = date_time.try_into()?;
                Ok(crate::model::TriggerProperty {
                    value: TriggerValue::Absolute((date, time, is_utc).into()),
                    params: self.params.to_model()?,
                })
            }
            crate::parser::types::DurationOrDateTime::Duration(duration) => {
                Ok(crate::model::TriggerProperty {
                    value: TriggerValue::Relative(duration.to_model()?),
                    params: self.params.to_model()?,
                })
            }
        }
    }
}

impl ToModel for crate::parser::types::RepeatProperty<'_> {
    type Model = crate::model::RepeatProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::RepeatProperty {
            value: self.value,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::ComponentProperty<'_> {
    type Model = crate::model::ComponentProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        match self {
            crate::parser::types::ComponentProperty::DateTimeStamp(date_time_stamp) => Ok(
                crate::model::ComponentProperty::DateTimeStamp(date_time_stamp.to_model()?),
            ),
            crate::parser::types::ComponentProperty::UniqueIdentifier(unique_identifier) => Ok(
                crate::model::ComponentProperty::UniqueIdentifier(unique_identifier.to_model()?),
            ),
            crate::parser::types::ComponentProperty::DateTimeStart(date_time_start) => Ok(
                crate::model::ComponentProperty::DateTimeStart(date_time_start.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Classification(classification) => Ok(
                crate::model::ComponentProperty::Classification(classification.to_model()?),
            ),
            crate::parser::types::ComponentProperty::DateTimeCreated(created) => Ok(
                crate::model::ComponentProperty::DateTimeCreated(created.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Description(description) => Ok(
                crate::model::ComponentProperty::Description(description.to_model()?),
            ),
            crate::parser::types::ComponentProperty::GeographicPosition(geo_pos) => Ok(
                crate::model::ComponentProperty::GeographicPosition(geo_pos.to_model()?),
            ),
            crate::parser::types::ComponentProperty::LastModified(last_modified) => Ok(
                crate::model::ComponentProperty::LastModified(last_modified.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Location(location) => Ok(
                crate::model::ComponentProperty::Location(location.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Organizer(organizer) => Ok(
                crate::model::ComponentProperty::Organizer(organizer.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Priority(priority) => Ok(
                crate::model::ComponentProperty::Priority(priority.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Sequence(sequence) => Ok(
                crate::model::ComponentProperty::Sequence(sequence.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Status(status) => {
                Ok(crate::model::ComponentProperty::Status(status.to_model()?))
            }
            crate::parser::types::ComponentProperty::Summary(summary) => Ok(
                crate::model::ComponentProperty::Summary(summary.to_model()?),
            ),
            crate::parser::types::ComponentProperty::TimeTransparency(time_transparency) => Ok(
                crate::model::ComponentProperty::TimeTransparency(time_transparency.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Url(url) => {
                Ok(crate::model::ComponentProperty::Url(url.to_model()?))
            }
            crate::parser::types::ComponentProperty::RecurrenceId(recurrence_id) => Ok(
                crate::model::ComponentProperty::RecurrenceId(recurrence_id.to_model()?),
            ),
            crate::parser::types::ComponentProperty::RecurrenceRule(recurrence_rule) => Ok(
                crate::model::ComponentProperty::RecurrenceRule(recurrence_rule.to_model()?),
            ),
            crate::parser::types::ComponentProperty::DateTimeEnd(date_time_end) => Ok(
                crate::model::ComponentProperty::DateTimeEnd(date_time_end.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Duration(duration) => Ok(
                crate::model::ComponentProperty::Duration(duration.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Attach(attach) => {
                Ok(crate::model::ComponentProperty::Attach(attach.to_model()?))
            }
            crate::parser::types::ComponentProperty::Attendee(attendee) => Ok(
                crate::model::ComponentProperty::Attendee(attendee.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Categories(categories) => Ok(
                crate::model::ComponentProperty::Categories(categories.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Comment(comment) => Ok(
                crate::model::ComponentProperty::Comment(comment.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Contact(contact) => Ok(
                crate::model::ComponentProperty::Contact(contact.to_model()?),
            ),
            crate::parser::types::ComponentProperty::ExceptionDateTimes(exception_date_times) => {
                Ok(crate::model::ComponentProperty::ExceptionDateTimes(
                    exception_date_times.to_model()?,
                ))
            }
            crate::parser::types::ComponentProperty::RequestStatus(request_status) => Ok(
                crate::model::ComponentProperty::RequestStatus(request_status.to_model()?),
            ),
            crate::parser::types::ComponentProperty::RelatedTo(related_to) => Ok(
                crate::model::ComponentProperty::RelatedTo(related_to.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Resources(resources) => Ok(
                crate::model::ComponentProperty::Resources(resources.to_model()?),
            ),
            crate::parser::types::ComponentProperty::RecurrenceDateTimes(recurrence_date_times) => {
                Ok(crate::model::ComponentProperty::RecurrenceDateTimes(
                    recurrence_date_times.to_model()?,
                ))
            }
            crate::parser::types::ComponentProperty::DateTimeCompleted(date_time_completed) => Ok(
                crate::model::ComponentProperty::DateTimeCompleted(date_time_completed.to_model()?),
            ),
            crate::parser::types::ComponentProperty::PercentComplete(percent_complete) => Ok(
                crate::model::ComponentProperty::PercentComplete(percent_complete.to_model()?),
            ),
            crate::parser::types::ComponentProperty::DateTimeDue(date_time_due) => Ok(
                crate::model::ComponentProperty::DateTimeDue(date_time_due.to_model()?),
            ),
            crate::parser::types::ComponentProperty::FreeBusyTime(free_busy_time) => Ok(
                crate::model::ComponentProperty::FreeBusyTime(free_busy_time.to_model()?),
            ),
            crate::parser::types::ComponentProperty::TimeZoneId(time_zone_id) => Ok(
                crate::model::ComponentProperty::TimeZoneId(time_zone_id.to_model()?),
            ),
            crate::parser::types::ComponentProperty::TimeZoneUrl(time_zone_url) => Ok(
                crate::model::ComponentProperty::TimeZoneUrl(time_zone_url.to_model()?),
            ),
            crate::parser::types::ComponentProperty::TimeZoneOffsetTo(time_zone_offset_to) => Ok(
                crate::model::ComponentProperty::TimeZoneOffsetTo(time_zone_offset_to.to_model()?),
            ),
            crate::parser::types::ComponentProperty::TimeZoneOffsetFrom(time_zone_offset_from) => {
                let to = time_zone_offset_from.to_model()?;
                Ok(crate::model::ComponentProperty::TimeZoneOffsetFrom(
                    crate::model::TimeZoneOffsetFromProperty {
                        value: to.value,
                        params: to.params,
                    },
                ))
            }
            crate::parser::types::ComponentProperty::TimeZoneName(time_zone_name) => Ok(
                crate::model::ComponentProperty::TimeZoneName(time_zone_name.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Action(action) => {
                Ok(crate::model::ComponentProperty::Action(action.to_model()?))
            }
            crate::parser::types::ComponentProperty::Trigger(trigger) => Ok(
                crate::model::ComponentProperty::Trigger(trigger.to_model()?),
            ),
            crate::parser::types::ComponentProperty::RepeatCount(repeat_count) => Ok(
                crate::model::ComponentProperty::Repeat(repeat_count.to_model()?),
            ),
            crate::parser::types::ComponentProperty::XProperty(x_prop) => Ok(
                crate::model::ComponentProperty::XProperty(x_prop.to_model()?),
            ),
            crate::parser::types::ComponentProperty::IanaProperty(iana_prop) => Ok(
                crate::model::ComponentProperty::IanaProperty(iana_prop.to_model()?),
            ),
        }
    }
}

impl ToModel for ContentLine<'_> {
    type Model = crate::model::IanaProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::IanaProperty {
            name: convert_string(self.property_name),
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::Period {
    type Model = Period;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(Period {
            start: (&self.start).try_into()?,
            end: match &self.end {
                crate::parser::PeriodEnd::DateTime(date_time) => {
                    crate::model::PeriodEnd::DateTime(date_time.try_into()?)
                }
                crate::parser::PeriodEnd::Duration(duration) => {
                    crate::model::PeriodEnd::Duration(duration.to_model()?)
                }
            },
        })
    }
}

impl ToModel for crate::parser::types::DateOrDateTime {
    type Model = CalendarDateTime;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(match self {
            crate::parser::types::DateOrDateTime::Date(date) => {
                (date.try_into()?, None, false).into()
            }
            crate::parser::types::DateOrDateTime::DateTime(datetime) => {
                let (date, time, is_utc) = datetime.try_into()?;
                (date, Some(time), is_utc).into()
            }
        })
    }
}

impl TryFrom<&crate::parser::types::Date> for time::Date {
    type Error = anyhow::Error;

    fn try_from(value: &crate::parser::types::Date) -> Result<Self, Self::Error> {
        time::Date::from_calendar_date(
            value.year as i32,
            time::Month::try_from(value.month).context("Invalid month")?,
            value.day,
        )
        .context("Invalid date")
    }
}

impl TryFrom<&crate::parser::types::DateTime> for (time::Date, time::Time, bool) {
    type Error = anyhow::Error;

    fn try_from(value: &crate::parser::types::DateTime) -> Result<Self, Self::Error> {
        Ok((
            time::Date::try_from(&value.date)?,
            time::Time::from_hms(value.time.hour, value.time.minute, value.time.second)
                .context("Invalid time")?,
            value.time.is_utc,
        ))
    }
}
