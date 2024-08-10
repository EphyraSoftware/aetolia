pub mod alarm;
mod daylight;
pub mod event;
mod free_busy;
pub mod iana_component;
mod journal;
mod standard;
mod time_zone;
mod todo;
pub mod x_component;

pub use crate::model::component::daylight::DaylightComponent;
pub use crate::model::component::standard::StandardComponent;
pub use alarm::AlarmComponent;
pub use free_busy::{FreeBusyComponent, FreeBusyComponentBuilder};
pub use iana_component::{IanaComponent, IanaComponentBuilder};
pub use journal::{JournalComponent, JournalComponentBuilder};
pub use time_zone::{TimeZoneComponent, TimeZoneComponentBuilder};
pub use todo::{ToDoComponent, ToDoComponentBuilder};
pub use x_component::{XComponent, XComponentBuilder};

#[derive(Debug, PartialEq)]
pub enum CalendarComponent {
    Event(EventComponent),
    ToDo(ToDoComponent),
    Journal(JournalComponent),
    FreeBusy(FreeBusyComponent),
    TimeZone(TimeZoneComponent),
    Standard(StandardComponent),
    Daylight(DaylightComponent),
    Alarm(AlarmComponent),
    IanaComponent(IanaComponent),
    XComponent(XComponent),
}

impl ComponentAccess for CalendarComponent {
    fn properties(&self) -> &[ComponentProperty] {
        match self {
            CalendarComponent::Event(e) => &e.properties,
            CalendarComponent::ToDo(t) => &t.properties,
            CalendarComponent::Journal(j) => &j.properties,
            CalendarComponent::FreeBusy(f) => &f.properties,
            CalendarComponent::TimeZone(tz) => &tz.properties,
            CalendarComponent::Standard(s) => &s.properties,
            CalendarComponent::Daylight(d) => &d.properties,
            CalendarComponent::Alarm(a) => &a.properties,
            CalendarComponent::IanaComponent(i) => &i.properties,
            CalendarComponent::XComponent(x) => &x.properties,
        }
    }
}

macro_rules! impl_finish_component_build {
    ($ev:expr) => {
        pub fn finish_component(mut self) -> $crate::model::object::ICalObjectBuilder {
            self.owner.inner.components.push($ev(self.inner));
            self.owner
        }
    };
}

pub(crate) use impl_finish_component_build;

macro_rules! impl_other_component_properties {
    ($x_builder:ident, $iana_builder:ident, $inner:ty) => {
        pub fn add_x_property<N: ToString, V: ToString>(
            self,
            name: N,
            value: V,
        ) -> $x_builder<$inner> {
            $x_builder::new(self, name.to_string(), value.to_string())
        }

        pub fn add_iana_property<N: ToString, V: ToString>(
            self,
            name: N,
            value: V,
        ) -> $iana_builder<$inner> {
            $iana_builder::new(self, name.to_string(), value.to_string())
        }
    };
}

pub(crate) use impl_other_component_properties;

use crate::model::component::event::EventComponent;

macro_rules! add_date_time_stamp {
    () => {
        pub fn add_date_time_stamp(
            self,
            date: time::Date,
            time: time::Time,
        ) -> $crate::model::property::DateTimeStampPropertyBuilder<Self> {
            $crate::model::property::DateTimeStampPropertyBuilder::new(self, date, time)
        }
    };
}

pub(crate) use add_date_time_stamp;

macro_rules! add_unique_identifier {
    () => {
        pub fn add_unique_identifier<V: ToString>(
            self,
            value: V,
        ) -> $crate::model::property::UniqueIdentifierPropertyBuilder<Self> {
            $crate::model::property::UniqueIdentifierPropertyBuilder::new(self, value.to_string())
        }
    };
}

pub(crate) use add_unique_identifier;

macro_rules! add_class {
    () => {
        pub fn add_classification(
            self,
            value: $crate::model::property::Classification,
        ) -> $crate::model::property::ClassificationPropertyBuilder<Self> {
            $crate::model::property::ClassificationPropertyBuilder::new(self, value)
        }
    };
}

pub(crate) use add_class;

macro_rules! add_created {
    () => {
        pub fn add_date_time_created(
            self,
            date: time::Date,
            time: time::Time,
        ) -> $crate::model::property::CreatedPropertyBuilder<Self> {
            $crate::model::property::CreatedPropertyBuilder::new(self, date, time)
        }
    };
}

pub(crate) use add_created;

macro_rules! add_description {
    () => {
        pub fn add_description<V: ToString>(
            self,
            value: V,
        ) -> $crate::model::property::DescriptionPropertyBuilder<Self> {
            $crate::model::property::DescriptionPropertyBuilder::new(self, value.to_string())
        }
    };
}

pub(crate) use add_description;

macro_rules! add_date_time_start {
    () => {
        pub fn add_date_time_start(
            self,
            date: time::Date,
            time: Option<time::Time>,
        ) -> $crate::model::property::DateTimeStartPropertyBuilder<Self> {
            $crate::model::property::DateTimeStartPropertyBuilder::new(self, date, time)
        }
    };
}

pub(crate) use add_date_time_start;

macro_rules! add_geographic_position {
    () => {
        pub fn add_geographic_position(
            self,
            latitude: f64,
            longitude: f64,
        ) -> $crate::model::property::GeographicPositionPropertyBuilder<Self> {
            $crate::model::property::GeographicPositionPropertyBuilder::new(
                self, latitude, longitude,
            )
        }
    };
}

pub(crate) use add_geographic_position;

macro_rules! add_last_modified {
    () => {
        pub fn add_last_modified(
            self,
            date: time::Date,
            time: time::Time,
        ) -> $crate::model::property::LastModifiedPropertyBuilder<Self> {
            $crate::model::property::LastModifiedPropertyBuilder::new(self, date, time)
        }
    };
}

pub(crate) use add_last_modified;

macro_rules! add_location {
    () => {
        pub fn add_location(
            self,
            value: &str,
        ) -> $crate::model::property::LocationPropertyBuilder<Self> {
            $crate::model::property::LocationPropertyBuilder::new(self, value.to_string())
        }
    };
}

pub(crate) use add_location;

macro_rules! add_organizer {
    () => {
        pub fn add_organizer(
            self,
            value: &str,
        ) -> $crate::model::property::OrganizerPropertyBuilder<Self> {
            $crate::model::property::OrganizerPropertyBuilder::new(self, value.to_string())
        }
    };
}

pub(crate) use add_organizer;

macro_rules! add_priority {
    () => {
        pub fn add_priority(
            self,
            value: u8,
        ) -> $crate::model::property::PriorityPropertyBuilder<Self> {
            $crate::model::property::PriorityPropertyBuilder::new(self, value)
        }
    };
}

pub(crate) use add_priority;

macro_rules! add_recurrence_id {
    () => {
        pub fn add_recurrence_id(
            self,
            date: time::Date,
            time: Option<time::Time>,
        ) -> $crate::model::property::RecurrenceIdPropertyBuilder<Self> {
            $crate::model::property::RecurrenceIdPropertyBuilder::new(self, date, time)
        }
    };
}

pub(crate) use add_recurrence_id;

macro_rules! add_sequence {
    () => {
        pub fn add_sequence(
            self,
            value: u32,
        ) -> $crate::model::property::SequencePropertyBuilder<Self> {
            $crate::model::property::SequencePropertyBuilder::new(self, value)
        }
    };
}

pub(crate) use add_sequence;

macro_rules! add_summary {
    () => {
        pub fn add_summary<V: ToString>(
            self,
            value: V,
        ) -> $crate::model::property::SummaryPropertyBuilder<Self> {
            $crate::model::property::SummaryPropertyBuilder::new(self, value.to_string())
        }
    };
}

pub(crate) use add_summary;

macro_rules! add_url {
    () => {
        pub fn add_url(self, value: &str) -> $crate::model::property::UrlPropertyBuilder<Self> {
            $crate::model::property::UrlPropertyBuilder::new(self, value.to_string())
        }
    };
}

pub(crate) use add_url;

macro_rules! add_recurrence_rule {
    () => {
        pub fn add_recurrence_rule(
            self,
            frequency: $crate::common::RecurFreq,
            builder: fn(
                $crate::model::property::RecurrenceRule,
            ) -> $crate::model::property::RecurrenceRule,
        ) -> $crate::model::property::RecurrenceRulePropertyBuilder<Self> {
            $crate::model::property::RecurrenceRulePropertyBuilder::new(
                self,
                builder($crate::model::property::RecurrenceRule::new(frequency)),
            )
        }
    };
}

pub(crate) use add_recurrence_rule;

macro_rules! add_duration {
    () => {
        pub fn add_duration(
            self,
            builder: fn() -> $crate::model::property::Duration,
        ) -> $crate::model::property::DurationPropertyBuilder<Self> {
            $crate::model::property::DurationPropertyBuilder::new(self, builder())
        }
    };
}

pub(crate) use add_duration;

macro_rules! add_attach {
    () => {
        pub fn add_attach_uri(
            self,
            value: &str,
        ) -> $crate::model::property::AttachPropertyBuilder<Self> {
            $crate::model::property::AttachPropertyBuilder::new_with_uri(self, value.to_string())
        }

        pub fn add_attach_binary(
            self,
            value: &str,
        ) -> $crate::model::property::AttachPropertyBuilder<Self> {
            $crate::model::property::AttachPropertyBuilder::new_with_binary(self, value.to_string())
        }
    };
}

pub(crate) use add_attach;

macro_rules! add_categories {
    () => {
        pub fn add_categories(
            self,
            value: Vec<&str>,
        ) -> $crate::model::property::CategoriesPropertyBuilder<Self> {
            $crate::model::property::CategoriesPropertyBuilder::new(
                self,
                value.into_iter().map(|s| s.to_string()).collect(),
            )
        }
    };
}

pub(crate) use add_categories;

macro_rules! add_comment {
    () => {
        pub fn add_comment<V: ToString>(
            self,
            value: V,
        ) -> $crate::model::property::CommentPropertyBuilder<Self> {
            $crate::model::property::CommentPropertyBuilder::new(self, value.to_string())
        }
    };
}

pub(crate) use add_comment;

macro_rules! add_contact {
    () => {
        pub fn add_contact<V: ToString>(
            self,
            value: V,
        ) -> $crate::model::property::ContactPropertyBuilder<Self> {
            $crate::model::property::ContactPropertyBuilder::new(self, value.to_string())
        }
    };
}

pub(crate) use add_contact;

macro_rules! add_exception_date_times {
    () => {
        pub fn add_exception_date_times(
            self,
            date_times: std::vec::Vec<$crate::common::CalendarDateTime>,
        ) -> $crate::model::property::ExceptionDateTimesPropertyBuilder<Self> {
            $crate::model::property::ExceptionDateTimesPropertyBuilder::new(self, date_times)
        }
    };
}

pub(crate) use add_exception_date_times;

macro_rules! add_request_status {
    () => {
        pub fn add_request_status(
            self,
            status_code: &[u32],
            description: &str,
            exception_data: std::option::Option<&str>,
        ) -> $crate::model::property::RequestStatusPropertyBuilder<Self> {
            $crate::model::property::RequestStatusPropertyBuilder::new(
                self,
                status_code.to_vec(),
                description.to_string(),
                exception_data.map(|s| s.to_string()),
            )
        }
    };
}

pub(crate) use add_request_status;

macro_rules! add_related {
    () => {
        pub fn add_related_to(
            self,
            value: &str,
        ) -> $crate::model::property::RelatedToPropertyBuilder<Self> {
            $crate::model::property::RelatedToPropertyBuilder::new(self, value.to_string())
        }
    };
}

pub(crate) use add_related;

macro_rules! add_resources {
    () => {
        pub fn add_resources(
            self,
            value: std::vec::Vec<&str>,
        ) -> $crate::model::property::ResourcesPropertyBuilder<Self> {
            $crate::model::property::ResourcesPropertyBuilder::new(
                self,
                value.into_iter().map(|s| s.to_string()).collect(),
            )
        }
    };
}

pub(crate) use add_resources;

macro_rules! add_recurrence_date {
    () => {
        pub fn add_recurrence_date_date_times(
            self,
            date_times: std::vec::Vec<$crate::common::CalendarDateTime>,
        ) -> $crate::model::property::RecurrenceDateTimesPropertyBuilder<Self> {
            $crate::model::property::RecurrenceDateTimesPropertyBuilder::new_date_times(
                self, date_times,
            )
        }

        pub fn add_recurrence_date_periods(
            self,
            periods: std::vec::Vec<$crate::model::property::Period>,
        ) -> $crate::model::property::RecurrenceDateTimesPropertyBuilder<Self> {
            $crate::model::property::RecurrenceDateTimesPropertyBuilder::new_periods(self, periods)
        }
    };
}

pub(crate) use add_recurrence_date;

macro_rules! add_date_time_end {
    () => {
        pub fn add_date_time_end(
            self,
            date: time::Date,
            time: std::option::Option<time::Time>,
        ) -> $crate::model::property::DateTimeEndPropertyBuilder<Self> {
            $crate::model::property::DateTimeEndPropertyBuilder::new(self, date, time)
        }
    };
}

pub(crate) use add_date_time_end;

macro_rules! add_action {
    ($typ:expr) => {
        pub fn add_action(self) -> $crate::model::property::ActionPropertyBuilder<Self> {
            $crate::model::property::ActionPropertyBuilder::new(self, $typ)
        }
    };
}

pub(crate) use add_action;

macro_rules! add_trigger {
    () => {
        pub fn add_relative_trigger(
            self,
            value: $crate::model::Duration,
        ) -> $crate::model::property::RelativeTriggerPropertyBuilder<Self> {
            $crate::model::property::RelativeTriggerPropertyBuilder::new(self, value)
        }

        pub fn add_absolute_trigger(
            self,
            date: time::Date,
            time: time::Time,
        ) -> $crate::model::property::AbsoluteTriggerPropertyBuilder<Self> {
            $crate::model::property::AbsoluteTriggerPropertyBuilder::new(self, date, time)
        }
    };
}

pub(crate) use add_trigger;

macro_rules! add_repeat {
    () => {
        pub fn add_repeat(
            self,
            value: u32,
        ) -> $crate::model::property::RepeatPropertyBuilder<Self> {
            $crate::model::property::RepeatPropertyBuilder::new(self, value)
        }
    };
}

pub(crate) use add_repeat;

macro_rules! add_alarms {
    () => {
        pub fn add_audio_alarm(
            self,
        ) -> $crate::model::component::alarm::AudioAlarmComponentBuilder<Self> {
            $crate::model::component::alarm::AudioAlarmComponentBuilder::new(self)
        }

        pub fn add_display_alarm(
            self,
        ) -> $crate::model::component::alarm::DisplayAlarmComponentBuilder<Self> {
            $crate::model::component::alarm::DisplayAlarmComponentBuilder::new(self)
        }

        pub fn add_email_alarm(
            self,
        ) -> $crate::model::component::alarm::EmailAlarmComponentBuilder<Self> {
            $crate::model::component::alarm::EmailAlarmComponentBuilder::new(self)
        }
    };
}

use crate::model::ComponentProperty;
use crate::prelude::ComponentAccess;
pub(crate) use add_alarms;
