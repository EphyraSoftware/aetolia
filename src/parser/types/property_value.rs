#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Date {
    pub year: u32,
    pub month: u8,
    pub day: u8,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Time {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub is_utc: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DateTime {
    pub date: Date,
    pub time: Time,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Duration {
    pub sign: i8,
    pub weeks: Option<u64>,
    pub days: Option<u64>,
    pub hours: Option<u64>,
    pub minutes: Option<u64>,
    pub seconds: Option<u64>,
}

impl Default for Duration {
    fn default() -> Self {
        Duration {
            sign: 1,
            weeks: None,
            days: None,
            hours: None,
            minutes: None,
            seconds: None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Period {
    pub start: DateTime,
    pub end: PeriodEnd,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PeriodEnd {
    DateTime(DateTime),
    Duration(Duration),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UtcOffset {
    pub sign: i8,
    pub hours: u64,
    pub minutes: u64,
    pub seconds: Option<u64>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DateOrDateTime {
    Date(Date),
    DateTime(DateTime),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DateOrDateTimeOrPeriod {
    Date(Date),
    DateTime(DateTime),
    Period(Period),
}
