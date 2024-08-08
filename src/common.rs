use crate::model::Duration;
use std::cmp::Ordering;
use std::ops::{Add, Sub};

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum CalendarUserType {
    #[default]
    Individual,
    Group,
    Resource,
    Room,
    Unknown,
    XName(String),
    IanaToken(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum Encoding {
    #[default]
    EightBit,
    Base64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FreeBusyTimeType {
    Free,
    Busy,
    BusyUnavailable,
    BusyTentative,
    XName(String),
    IanaToken(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageTag {
    pub language: String,
    pub ext_lang: Option<String>,
    pub script: Option<String>,
    pub region: Option<String>,
    pub variants: Vec<String>,
    pub extensions: Vec<String>,
    pub private_use: Option<String>,
}

impl Default for LanguageTag {
    fn default() -> Self {
        Self {
            language: String::new(),
            ext_lang: None,
            script: None,
            region: None,
            variants: Vec::with_capacity(0),
            extensions: Vec::with_capacity(0),
            private_use: None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Range {
    ThisAndFuture,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum Related {
    #[default]
    Start,
    End,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum RelationshipType {
    #[default]
    Parent,
    Child,
    Sibling,
    XName(String),
    IanaToken(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum Role {
    Chair,
    #[default]
    RequiredParticipant,
    OptionalParticipant,
    NonParticipant,
    XName(String),
    IanaToken(String),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Value {
    Binary,
    Boolean,
    CalendarAddress,
    Date,
    DateTime,
    Duration,
    Float,
    Integer,
    Period,
    Recurrence,
    Text,
    Time,
    Uri,
    UtcOffset,
    XName(String),
    IanaToken(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum ParticipationStatusUnknown {
    #[default]
    NeedsAction,
    Accepted,
    Declined,
    Tentative,
    Delegated,
    Completed,
    InProcess,
    XName(String),
    IanaToken(String),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Status {
    Tentative,
    Confirmed,
    Cancelled,
    NeedsAction,
    Completed,
    InProcess,
    Draft,
    Final,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TimeTransparency {
    Opaque,
    Transparent,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RecurFreq {
    Secondly,
    Minutely,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OffsetWeekday {
    pub offset_weeks: Option<i8>,
    pub weekday: Weekday,
}

impl OffsetWeekday {
    pub fn new(weekday: Weekday, offset_weeks: Option<i8>) -> Self {
        OffsetWeekday {
            weekday,
            offset_weeks,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalendarDateTime {
    date: time::Date,
    time: Option<time::Time>,
    utc: bool,
}

impl From<(time::Date, time::Time, bool)> for CalendarDateTime {
    fn from((date, time, utc): (time::Date, time::Time, bool)) -> Self {
        CalendarDateTime {
            date,
            time: Some(time),
            utc,
        }
    }
}

impl From<(time::Date, Option<time::Time>, bool)> for CalendarDateTime {
    fn from((date, time, utc): (time::Date, Option<time::Time>, bool)) -> Self {
        CalendarDateTime { date, time, utc }
    }
}

impl CalendarDateTime {
    pub fn add(&self, duration: &Duration) -> anyhow::Result<Self> {
        match self.time {
            Some(time) => {
                // TODO otherwise you have to account for daylight changes. Not yet supported
                assert!(self.utc);

                if let Some(weeks) = duration.weeks {
                    let new_date = if duration.sign > 0 {
                        self.date
                            .add(std::time::Duration::from_secs(weeks * 7 * 24 * 60 * 60))
                    } else {
                        self.date
                            .sub(std::time::Duration::from_secs(weeks * 7 * 24 * 60 * 60))
                    };
                    Ok(CalendarDateTime {
                        date: new_date,
                        time: self.time,
                        utc: self.utc,
                    })
                } else {
                    let mut new_date = self.date;
                    let mut new_time = time;

                    if duration.sign > 0 {
                        let mut add_days = 0;

                        if let Some(days) = duration.days {
                            add_days += days;
                        }

                        let mut add_seconds = duration.hours.unwrap_or(0) * 60 * 60
                            + duration.minutes.unwrap_or(0) * 60
                            + duration.seconds.unwrap_or(0);

                        const ONE_DAY: u64 = 24 * 60 * 60;
                        let part_day = new_time.hour() as u64 * 60 * 60
                            + new_time.minute() as u64 * 60
                            + new_time.second() as u64;
                        let gap = ONE_DAY - part_day;

                        if add_seconds > gap {
                            add_days += 1;
                            add_seconds -= gap;
                            new_time = new_time.add(std::time::Duration::from_secs(gap));
                        }

                        add_days += add_seconds / ONE_DAY;
                        add_seconds %= ONE_DAY;

                        new_date = new_date.add(std::time::Duration::from_secs(add_days * ONE_DAY));
                        new_time = new_time.add(std::time::Duration::from_secs(add_seconds));

                        Ok(CalendarDateTime {
                            date: new_date,
                            time: Some(new_time),
                            utc: self.utc,
                        })
                    } else {
                        let mut sub_days = 0;

                        if let Some(days) = duration.days {
                            sub_days += days;
                        }

                        let mut sub_seconds = duration.hours.unwrap_or(0) * 60 * 60
                            + duration.minutes.unwrap_or(0) * 60
                            + duration.seconds.unwrap_or(0);

                        const ONE_DAY: u64 = 24 * 60 * 60;
                        let part_day = new_time.hour() as u64 * 60 * 60
                            + new_time.minute() as u64 * 60
                            + new_time.second() as u64;

                        if sub_seconds > part_day {
                            sub_days += 1;
                            sub_seconds -= part_day;
                            new_time = new_time.sub(std::time::Duration::from_secs(part_day));
                        }

                        sub_days += sub_seconds / ONE_DAY;
                        sub_seconds %= ONE_DAY;

                        new_date = new_date.sub(std::time::Duration::from_secs(sub_days * ONE_DAY));
                        new_time = new_time.sub(std::time::Duration::from_secs(sub_seconds));

                        Ok(CalendarDateTime {
                            date: new_date,
                            time: Some(new_time),
                            utc: self.utc,
                        })
                    }
                }
            }
            None => {
                if let Some(weeks) = duration.weeks {
                    let new_date = if duration.sign > 0 {
                        self.date
                            .add(std::time::Duration::from_secs(weeks * 7 * 24 * 60 * 60))
                    } else {
                        self.date
                            .sub(std::time::Duration::from_secs(weeks * 7 * 24 * 60 * 60))
                    };
                    Ok(CalendarDateTime {
                        date: new_date,
                        time: self.time,
                        utc: self.utc,
                    })
                } else if let Some(days) = duration.days {
                    let new_date = if duration.sign > 0 {
                        self.date
                            .add(std::time::Duration::from_secs(days * 24 * 60 * 60))
                    } else {
                        self.date
                            .sub(std::time::Duration::from_secs(days * 24 * 60 * 60))
                    };
                    Ok(CalendarDateTime {
                        date: new_date,
                        time: self.time,
                        utc: self.utc,
                    })
                } else {
                    Err(anyhow::anyhow!(
                        "Duration is a time, but the calendar date time is just a date"
                    ))
                }
            }
        }
    }

    //
    // Query
    //

    pub fn is_date(&self) -> bool {
        self.time.is_none()
    }

    pub fn is_date_time(&self) -> bool {
        self.time.is_some()
    }

    pub fn is_utc(&self) -> bool {
        self.time.is_some() && self.utc
    }

    //
    // Get
    //

    pub fn date(&self) -> &time::Date {
        &self.date
    }

    pub fn time_opt(&self) -> Option<&time::Time> {
        self.time.as_ref()
    }

    //
    // Mutate
    //

    pub fn set_utc(&mut self, utc: bool) {
        self.utc = utc;
    }
}

impl PartialOrd for CalendarDateTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CalendarDateTime {
    fn cmp(&self, other: &Self) -> Ordering {
        let date_cmp = self.date.cmp(&other.date);
        if date_cmp != Ordering::Equal {
            return date_cmp;
        }

        let time_cmp = self.time.cmp(&other.time);
        if time_cmp != Ordering::Equal {
            return time_cmp;
        }

        self.utc.cmp(&other.utc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_duration_week() {
        let cdt: CalendarDateTime = (
            time::Date::from_calendar_date(1992, time::Month::April, 12).unwrap(),
            time::Time::from_hms(14, 0, 0).unwrap(),
            true,
        )
            .into();

        let duration = Duration {
            weeks: Some(1),
            ..Default::default()
        };
        let new = cdt.add(&duration).unwrap();

        check_duration_invariant(cdt, new, duration);
    }

    #[test]
    fn add_duration_day() {
        let cdt: CalendarDateTime = (
            time::Date::from_calendar_date(1992, time::Month::April, 12).unwrap(),
            time::Time::from_hms(14, 30, 0).unwrap(),
            true,
        )
            .into();

        let duration = Duration {
            days: Some(1),
            ..Default::default()
        };
        let new = cdt.add(&duration).unwrap();

        check_duration_invariant(cdt, new, duration);
    }

    #[test]
    fn add_duration_time_same_day() {
        let cdt: CalendarDateTime = (
            time::Date::from_calendar_date(1992, time::Month::April, 12).unwrap(),
            time::Time::from_hms(14, 30, 10).unwrap(),
            true,
        )
            .into();

        let duration = Duration {
            hours: Some(1),
            minutes: Some(30),
            seconds: Some(30),
            ..Default::default()
        };
        let new = cdt.add(&duration).unwrap();

        check_duration_invariant(cdt, new, duration);
    }

    #[test]
    fn add_duration_time_next_day() {
        let cdt: CalendarDateTime = (
            time::Date::from_calendar_date(1992, time::Month::April, 12).unwrap(),
            time::Time::from_hms(23, 30, 0).unwrap(),
            true,
        )
            .into();

        let duration = Duration {
            hours: Some(10),
            minutes: Some(30),
            seconds: Some(0),
            ..Default::default()
        };
        let new = cdt.add(&duration).unwrap();

        check_duration_invariant(cdt, new, duration);
    }

    #[test]
    fn add_negative_duration_week() {
        let cdt: CalendarDateTime = (
            time::Date::from_calendar_date(1992, time::Month::April, 12).unwrap(),
            time::Time::from_hms(14, 0, 0).unwrap(),
            true,
        )
            .into();

        let duration = Duration {
            sign: -1,
            weeks: Some(5),
            ..Default::default()
        };
        let new = cdt.add(&duration).unwrap();

        check_duration_invariant(cdt, new, duration);
    }

    #[test]
    fn add_negative_duration_day() {
        let cdt: CalendarDateTime = (
            time::Date::from_calendar_date(1992, time::Month::April, 12).unwrap(),
            time::Time::from_hms(14, 30, 0).unwrap(),
            true,
        )
            .into();

        let duration = Duration {
            sign: -1,
            days: Some(3),
            ..Default::default()
        };
        let new = cdt.add(&duration).unwrap();

        check_duration_invariant(cdt, new, duration);
    }

    #[test]
    fn add_negative_duration_time_same_day() {
        let cdt: CalendarDateTime = (
            time::Date::from_calendar_date(1992, time::Month::April, 12).unwrap(),
            time::Time::from_hms(14, 30, 10).unwrap(),
            true,
        )
            .into();

        let duration = Duration {
            sign: -1,
            hours: Some(1),
            minutes: Some(30),
            seconds: Some(30),
            ..Default::default()
        };
        let new = cdt.add(&duration).unwrap();

        check_duration_invariant(cdt, new, duration);
    }

    #[test]
    fn add_negative_duration_time_previous_day() {
        let cdt: CalendarDateTime = (
            time::Date::from_calendar_date(1992, time::Month::April, 12).unwrap(),
            time::Time::from_hms(3, 30, 0).unwrap(),
            true,
        )
            .into();

        let duration = Duration {
            sign: -1,
            hours: Some(10),
            minutes: Some(30),
            seconds: Some(0),
            ..Default::default()
        };
        let new = cdt.add(&duration).unwrap();

        check_duration_invariant(cdt, new, duration);
    }

    fn check_duration_invariant(
        original: CalendarDateTime,
        new: CalendarDateTime,
        duration: Duration,
    ) {
        let original = chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd_opt(
                original.date.year(),
                original.date.month() as u32,
                original.date.day() as u32,
            )
            .unwrap(),
            match original.time {
                Some(time) => chrono::NaiveTime::from_hms_opt(
                    time.hour() as u32,
                    time.minute() as u32,
                    time.second() as u32,
                )
                .unwrap(),
                None => chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            },
        );
        let new = chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd_opt(
                new.date.year(),
                new.date.month() as u32,
                new.date.day() as u32,
            )
            .unwrap(),
            match new.time {
                Some(time) => chrono::NaiveTime::from_hms_opt(
                    time.hour() as u32,
                    time.minute() as u32,
                    time.second() as u32,
                )
                .unwrap(),
                None => chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            },
        );

        println!("Original: {}", original);
        println!("New: {}", new);

        let dur = new.signed_duration_since(original);

        let (sign, duration) = duration.to_std();
        assert_eq!(sign as i64 * duration.as_secs() as i64, dur.num_seconds());
    }
}
