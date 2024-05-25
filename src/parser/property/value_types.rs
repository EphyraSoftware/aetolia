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
    pub weeks: u64,
    pub days: u64,
    pub seconds: u64,
}

impl Default for Duration {
    fn default() -> Self {
        Duration {
            sign: 1,
            weeks: 0,
            days: 0,
            seconds: 0,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Period<'a> {
    pub start: &'a [u8],
    pub end: PeriodEnd<'a>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PeriodEnd<'a> {
    DateTime(&'a [u8]),
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
pub enum DateOrDateTimeOrPeriod<'a> {
    Date(Date),
    DateTime(DateTime),
    Period(Period<'a>),
}
