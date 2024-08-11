use crate::common::CalendarDateTime;
use crate::convert::{convert_string, ToModel};
use crate::model::property::{
    GeographicPositionPropertyValue, Period, RecurrenceDateTimesPropertyValue,
    RequestStatusPropertyValue, TimeZoneIdPropertyValue, TriggerValue,
};
use crate::parser::types::ContentLine;
use anyhow::Context;

mod recur;

impl ToModel for crate::parser::types::DateTimeStampProperty<'_> {
    type Model = crate::model::property::DateTimeStampProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::DateTimeStampProperty {
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
    type Model = crate::model::property::UniqueIdentifierProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::UniqueIdentifierProperty {
            value: convert_string(&self.value),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::DateTimeStartProperty<'_> {
    type Model = crate::model::property::DateTimeStartProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let dt = self.value.to_model()?;

        Ok(crate::model::property::DateTimeStartProperty {
            value: dt,
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::ClassificationProperty<'_> {
    type Model = crate::model::property::ClassificationProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::ClassificationProperty {
            value: self.value.to_model()?,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::Classification<'_> {
    type Model = crate::model::property::Classification;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(match self {
            crate::parser::types::Classification::Public => {
                crate::model::property::Classification::Public
            }
            crate::parser::types::Classification::Private => {
                crate::model::property::Classification::Private
            }
            crate::parser::types::Classification::Confidential => {
                crate::model::property::Classification::Confidential
            }
            crate::parser::types::Classification::XName(name) => {
                crate::model::property::Classification::XName(convert_string(name))
            }
            crate::parser::types::Classification::IanaToken(token) => {
                crate::model::property::Classification::IanaToken(convert_string(token))
            }
        })
    }
}

impl ToModel for crate::parser::types::DateTimeCreatedProperty<'_> {
    type Model = crate::model::property::CreatedProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let (date, time, is_utc) = (&self.value).try_into()?;

        Ok(crate::model::property::CreatedProperty {
            value: (date, time, is_utc).into(),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::DescriptionProperty<'_> {
    type Model = crate::model::property::DescriptionProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::DescriptionProperty {
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::GeographicPositionProperty<'_> {
    type Model = crate::model::property::GeographicPositionProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::GeographicPositionProperty {
            value: GeographicPositionPropertyValue {
                latitude: self.latitude,
                longitude: self.longitude,
            },
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::LastModifiedProperty<'_> {
    type Model = crate::model::property::LastModifiedProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let (date, time, is_utc) = (&self.value).try_into()?;

        Ok(crate::model::property::LastModifiedProperty {
            value: (date, time, is_utc).into(),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::LocationProperty<'_> {
    type Model = crate::model::property::LocationProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::LocationProperty {
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::OrganizerProperty<'_> {
    type Model = crate::model::property::OrganizerProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::OrganizerProperty {
            value: convert_string(self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::PriorityProperty<'_> {
    type Model = crate::model::property::PriorityProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::PriorityProperty {
            value: self.value,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::SequenceProperty<'_> {
    type Model = crate::model::property::SequenceProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::SequenceProperty {
            value: self.value,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::StatusProperty<'_> {
    type Model = crate::model::property::StatusProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::StatusProperty {
            value: self.value.clone(),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::SummaryProperty<'_> {
    type Model = crate::model::property::SummaryProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::SummaryProperty {
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::TimeTransparencyProperty<'_> {
    type Model = crate::model::property::TimeTransparencyProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::TimeTransparencyProperty {
            value: self.value.clone(),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::UrlProperty<'_> {
    type Model = crate::model::property::UrlProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::UrlProperty {
            value: self.value.to_string(),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::RecurrenceIdProperty<'_> {
    type Model = crate::model::property::RecurrenceIdProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let dt = self.value.to_model()?;

        Ok(crate::model::property::RecurrenceIdProperty {
            value: dt,
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::RecurrenceRuleProperty<'_> {
    type Model = crate::model::property::RecurrenceRuleProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::RecurrenceRuleProperty {
            value: self.value.to_model()?,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::DateTimeEndProperty<'_> {
    type Model = crate::model::property::DateTimeEndProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let dt = self.value.to_model()?;

        Ok(crate::model::property::DateTimeEndProperty {
            value: dt,
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::DurationProperty<'_> {
    type Model = crate::model::property::DurationProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::DurationProperty {
            value: self.value.to_model()?,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::AttachProperty<'_> {
    type Model = crate::model::property::AttachProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let value = match self.value {
            crate::parser::types::AttachValue::Uri(uri) => convert_string(uri),
            crate::parser::types::AttachValue::Binary(binary) => convert_string(binary),
        };

        Ok(crate::model::property::AttachProperty {
            value,
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::AttendeeProperty<'_> {
    type Model = crate::model::property::AttendeeProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::AttendeeProperty {
            value: convert_string(self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::CategoriesProperty<'_> {
    type Model = crate::model::property::CategoriesProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::CategoriesProperty {
            value: self.value.iter().map(|v| convert_string(v)).collect(),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::CommentProperty<'_> {
    type Model = crate::model::property::CommentProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::CommentProperty {
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::ContactProperty<'_> {
    type Model = crate::model::property::ContactProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::ContactProperty {
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::ExceptionDateTimesProperty<'_> {
    type Model = crate::model::property::ExceptionDateTimesProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::ExceptionDateTimesProperty {
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
    type Model = crate::model::property::RequestStatusProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::RequestStatusProperty {
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
    type Model = crate::model::property::RelatedToProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::RelatedToProperty {
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::ResourcesProperty<'_> {
    type Model = crate::model::property::ResourcesProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::ResourcesProperty {
            value: self.value.iter().map(|v| convert_string(v)).collect(),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::XProperty<'_> {
    type Model = crate::model::property::XProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::XProperty {
            name: convert_string(self.name),
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::IanaProperty<'_> {
    type Model = crate::model::property::IanaProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::IanaProperty {
            name: convert_string(self.name),
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::RecurrenceDateTimesProperty<'_> {
    type Model = crate::model::property::RecurrenceDateTimesProperty;

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

        Ok(crate::model::property::RecurrenceDateTimesProperty {
            value: if !periods.is_empty() {
                RecurrenceDateTimesPropertyValue::Periods(periods)
            } else {
                RecurrenceDateTimesPropertyValue::DateTimes(date_times)
            },
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::Duration {
    type Model = crate::model::property::Duration;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::Duration {
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
    type Model = crate::model::property::ProductIdProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::ProductIdProperty {
            value: convert_string(&self.value),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::VersionProperty<'_> {
    type Model = crate::model::property::VersionProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::VersionProperty {
            min_version: self.min_version.map(convert_string),
            max_version: convert_string(self.max_version),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::CalendarScaleProperty<'_> {
    type Model = crate::model::property::CalendarScaleProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::CalendarScaleProperty {
            value: convert_string(&self.value),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::MethodProperty<'_> {
    type Model = crate::model::property::MethodProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::MethodProperty {
            value: convert_string(self.value),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::CalendarProperty<'_> {
    type Model = crate::model::property::CalendarProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        match self {
            crate::parser::types::CalendarProperty::ProductId(product_id) => Ok(
                crate::model::property::CalendarProperty::ProductId(product_id.to_model()?),
            ),
            crate::parser::types::CalendarProperty::Version(version) => Ok(
                crate::model::property::CalendarProperty::Version(version.to_model()?),
            ),
            crate::parser::types::CalendarProperty::CalendarScale(cal_scale) => Ok(
                crate::model::property::CalendarProperty::CalendarScale(cal_scale.to_model()?),
            ),
            crate::parser::types::CalendarProperty::Method(method) => Ok(
                crate::model::property::CalendarProperty::Method(method.to_model()?),
            ),
            crate::parser::types::CalendarProperty::XProperty(x_prop) => Ok(
                crate::model::property::CalendarProperty::XProperty(x_prop.to_model()?),
            ),
            crate::parser::types::CalendarProperty::IanaProperty(iana_prop) => Ok(
                crate::model::property::CalendarProperty::IanaProperty(iana_prop.to_model()?),
            ),
        }
    }
}

impl ToModel for crate::parser::types::DateTimeCompletedProperty<'_> {
    type Model = crate::model::property::DateTimeCompletedProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let (date, time, is_utc) = (&self.value).try_into()?;

        Ok(crate::model::property::DateTimeCompletedProperty {
            value: (date, time, is_utc).into(),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::PercentCompleteProperty<'_> {
    type Model = crate::model::property::PercentCompleteProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::PercentCompleteProperty {
            value: self.value,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::DateTimeDueProperty<'_> {
    type Model = crate::model::property::DateTimeDueProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let dt = self.value.to_model()?;

        Ok(crate::model::property::DateTimeDueProperty {
            value: dt,
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::FreeBusyTimeProperty<'_> {
    type Model = crate::model::property::FreeBusyTimeProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::FreeBusyTimeProperty {
            value: self.value.to_model()?,
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::TimeZoneIdProperty<'_> {
    type Model = crate::model::property::TimeZoneIdProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::TimeZoneIdProperty {
            value: TimeZoneIdPropertyValue {
                id: convert_string(&self.value),
                unique_registry_id: self.unique_registry_id,
            },
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::TimeZoneUrlProperty<'_> {
    type Model = crate::model::property::TimeZoneUrlProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::TimeZoneUrlProperty {
            value: convert_string(self.value),
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::TimeZoneOffsetProperty<'_> {
    type Model = crate::model::property::TimeZoneOffsetToProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::TimeZoneOffsetToProperty {
            value: self.value.to_model()?,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::UtcOffset {
    type Model = crate::model::property::TimeZoneOffset;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::TimeZoneOffset {
            sign: self.sign,
            hours: self.hours as u8,
            minutes: self.minutes as u8,
            seconds: self.seconds.map(|s| s as u8),
        })
    }
}

impl ToModel for crate::parser::types::TimeZoneNameProperty<'_> {
    type Model = crate::model::property::TimeZoneNameProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::TimeZoneNameProperty {
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::ActionProperty<'_> {
    type Model = crate::model::property::ActionProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::ActionProperty {
            value: self.value.to_model()?,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::Action<'_> {
    type Model = crate::model::property::Action;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(match self {
            crate::parser::types::Action::Audio => crate::model::property::Action::Audio,
            crate::parser::types::Action::Display => crate::model::property::Action::Display,
            crate::parser::types::Action::Email => crate::model::property::Action::Email,
            crate::parser::types::Action::XName(name) => {
                crate::model::property::Action::XName(convert_string(name))
            }
            crate::parser::types::Action::IanaToken(token) => {
                crate::model::property::Action::IanaToken(convert_string(token))
            }
        })
    }
}

impl ToModel for crate::parser::types::TriggerProperty<'_> {
    type Model = crate::model::property::TriggerProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        match &self.value {
            crate::parser::types::DurationOrDateTime::DateTime(date_time) => {
                let (date, time, is_utc) = date_time.try_into()?;
                Ok(crate::model::property::TriggerProperty {
                    value: TriggerValue::Absolute((date, time, is_utc).into()),
                    params: self.params.to_model()?,
                })
            }
            crate::parser::types::DurationOrDateTime::Duration(duration) => {
                Ok(crate::model::property::TriggerProperty {
                    value: TriggerValue::Relative(duration.to_model()?),
                    params: self.params.to_model()?,
                })
            }
        }
    }
}

impl ToModel for crate::parser::types::RepeatProperty<'_> {
    type Model = crate::model::property::RepeatProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::RepeatProperty {
            value: self.value,
            params: self.other_params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::ComponentProperty<'_> {
    type Model = crate::model::property::ComponentProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        match self {
            crate::parser::types::ComponentProperty::DateTimeStamp(date_time_stamp) => {
                Ok(crate::model::property::ComponentProperty::DateTimeStamp(
                    date_time_stamp.to_model()?,
                ))
            }
            crate::parser::types::ComponentProperty::UniqueIdentifier(unique_identifier) => {
                Ok(crate::model::property::ComponentProperty::UniqueIdentifier(
                    unique_identifier.to_model()?,
                ))
            }
            crate::parser::types::ComponentProperty::DateTimeStart(date_time_start) => {
                Ok(crate::model::property::ComponentProperty::DateTimeStart(
                    date_time_start.to_model()?,
                ))
            }
            crate::parser::types::ComponentProperty::Classification(classification) => {
                Ok(crate::model::property::ComponentProperty::Classification(
                    classification.to_model()?,
                ))
            }
            crate::parser::types::ComponentProperty::DateTimeCreated(created) => Ok(
                crate::model::property::ComponentProperty::DateTimeCreated(created.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Description(description) => Ok(
                crate::model::property::ComponentProperty::Description(description.to_model()?),
            ),
            crate::parser::types::ComponentProperty::GeographicPosition(geo_pos) => Ok(
                crate::model::property::ComponentProperty::GeographicPosition(geo_pos.to_model()?),
            ),
            crate::parser::types::ComponentProperty::LastModified(last_modified) => Ok(
                crate::model::property::ComponentProperty::LastModified(last_modified.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Location(location) => Ok(
                crate::model::property::ComponentProperty::Location(location.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Organizer(organizer) => Ok(
                crate::model::property::ComponentProperty::Organizer(organizer.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Priority(priority) => Ok(
                crate::model::property::ComponentProperty::Priority(priority.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Sequence(sequence) => Ok(
                crate::model::property::ComponentProperty::Sequence(sequence.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Status(status) => Ok(
                crate::model::property::ComponentProperty::Status(status.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Summary(summary) => Ok(
                crate::model::property::ComponentProperty::Summary(summary.to_model()?),
            ),
            crate::parser::types::ComponentProperty::TimeTransparency(time_transparency) => {
                Ok(crate::model::property::ComponentProperty::TimeTransparency(
                    time_transparency.to_model()?,
                ))
            }
            crate::parser::types::ComponentProperty::Url(url) => Ok(
                crate::model::property::ComponentProperty::Url(url.to_model()?),
            ),
            crate::parser::types::ComponentProperty::RecurrenceId(recurrence_id) => Ok(
                crate::model::property::ComponentProperty::RecurrenceId(recurrence_id.to_model()?),
            ),
            crate::parser::types::ComponentProperty::RecurrenceRule(recurrence_rule) => {
                Ok(crate::model::property::ComponentProperty::RecurrenceRule(
                    recurrence_rule.to_model()?,
                ))
            }
            crate::parser::types::ComponentProperty::DateTimeEnd(date_time_end) => Ok(
                crate::model::property::ComponentProperty::DateTimeEnd(date_time_end.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Duration(duration) => Ok(
                crate::model::property::ComponentProperty::Duration(duration.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Attach(attach) => Ok(
                crate::model::property::ComponentProperty::Attach(attach.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Attendee(attendee) => Ok(
                crate::model::property::ComponentProperty::Attendee(attendee.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Categories(categories) => Ok(
                crate::model::property::ComponentProperty::Categories(categories.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Comment(comment) => Ok(
                crate::model::property::ComponentProperty::Comment(comment.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Contact(contact) => Ok(
                crate::model::property::ComponentProperty::Contact(contact.to_model()?),
            ),
            crate::parser::types::ComponentProperty::ExceptionDateTimes(exception_date_times) => {
                Ok(
                    crate::model::property::ComponentProperty::ExceptionDateTimes(
                        exception_date_times.to_model()?,
                    ),
                )
            }
            crate::parser::types::ComponentProperty::RequestStatus(request_status) => {
                Ok(crate::model::property::ComponentProperty::RequestStatus(
                    request_status.to_model()?,
                ))
            }
            crate::parser::types::ComponentProperty::RelatedTo(related_to) => Ok(
                crate::model::property::ComponentProperty::RelatedTo(related_to.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Resources(resources) => Ok(
                crate::model::property::ComponentProperty::Resources(resources.to_model()?),
            ),
            crate::parser::types::ComponentProperty::RecurrenceDateTimes(recurrence_date_times) => {
                Ok(
                    crate::model::property::ComponentProperty::RecurrenceDateTimes(
                        recurrence_date_times.to_model()?,
                    ),
                )
            }
            crate::parser::types::ComponentProperty::DateTimeCompleted(date_time_completed) => Ok(
                crate::model::property::ComponentProperty::DateTimeCompleted(
                    date_time_completed.to_model()?,
                ),
            ),
            crate::parser::types::ComponentProperty::PercentComplete(percent_complete) => {
                Ok(crate::model::property::ComponentProperty::PercentComplete(
                    percent_complete.to_model()?,
                ))
            }
            crate::parser::types::ComponentProperty::DateTimeDue(date_time_due) => Ok(
                crate::model::property::ComponentProperty::DateTimeDue(date_time_due.to_model()?),
            ),
            crate::parser::types::ComponentProperty::FreeBusyTime(free_busy_time) => Ok(
                crate::model::property::ComponentProperty::FreeBusyTime(free_busy_time.to_model()?),
            ),
            crate::parser::types::ComponentProperty::TimeZoneId(time_zone_id) => Ok(
                crate::model::property::ComponentProperty::TimeZoneId(time_zone_id.to_model()?),
            ),
            crate::parser::types::ComponentProperty::TimeZoneUrl(time_zone_url) => Ok(
                crate::model::property::ComponentProperty::TimeZoneUrl(time_zone_url.to_model()?),
            ),
            crate::parser::types::ComponentProperty::TimeZoneOffsetTo(time_zone_offset_to) => {
                Ok(crate::model::property::ComponentProperty::TimeZoneOffsetTo(
                    time_zone_offset_to.to_model()?,
                ))
            }
            crate::parser::types::ComponentProperty::TimeZoneOffsetFrom(time_zone_offset_from) => {
                let to = time_zone_offset_from.to_model()?;
                Ok(
                    crate::model::property::ComponentProperty::TimeZoneOffsetFrom(
                        crate::model::property::TimeZoneOffsetFromProperty {
                            value: to.value,
                            params: to.params,
                        },
                    ),
                )
            }
            crate::parser::types::ComponentProperty::TimeZoneName(time_zone_name) => Ok(
                crate::model::property::ComponentProperty::TimeZoneName(time_zone_name.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Action(action) => Ok(
                crate::model::property::ComponentProperty::Action(action.to_model()?),
            ),
            crate::parser::types::ComponentProperty::Trigger(trigger) => Ok(
                crate::model::property::ComponentProperty::Trigger(trigger.to_model()?),
            ),
            crate::parser::types::ComponentProperty::RepeatCount(repeat_count) => Ok(
                crate::model::property::ComponentProperty::Repeat(repeat_count.to_model()?),
            ),
            crate::parser::types::ComponentProperty::XProperty(x_prop) => Ok(
                crate::model::property::ComponentProperty::XProperty(x_prop.to_model()?),
            ),
            crate::parser::types::ComponentProperty::IanaProperty(iana_prop) => Ok(
                crate::model::property::ComponentProperty::IanaProperty(iana_prop.to_model()?),
            ),
        }
    }
}

impl ToModel for ContentLine<'_> {
    type Model = crate::model::property::IanaProperty;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(crate::model::property::IanaProperty {
            name: convert_string(self.property_name),
            value: convert_string(&self.value),
            params: self.params.to_model()?,
        })
    }
}

impl ToModel for crate::parser::types::Period {
    type Model = Period;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(Period {
            start: (&self.start).try_into()?,
            end: match &self.end {
                crate::parser::types::PeriodEnd::DateTime(date_time) => {
                    crate::model::property::PeriodEnd::DateTime(date_time.try_into()?)
                }
                crate::parser::types::PeriodEnd::Duration(duration) => {
                    crate::model::property::PeriodEnd::Duration(duration.to_model()?)
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
