use crate::common::{CalendarDateTime, OffsetWeekday, RecurFreq, Weekday};

#[derive(Debug)]
pub enum RecurRulePart {
    Freq(RecurFreq),
    Until(CalendarDateTime),
    Count(u64),
    Interval(u64),
    BySecList(Vec<u8>),
    ByMinute(Vec<u8>),
    ByHour(Vec<u8>),
    ByDay(Vec<OffsetWeekday>),
    ByMonthDay(Vec<i8>),
    ByYearDay(Vec<i16>),
    ByWeekNumber(Vec<i8>),
    ByMonth(Vec<time::Month>),
    BySetPos(Vec<i16>),
    WeekStart(Weekday),
}

#[derive(Debug)]
pub struct RecurrenceRule {
    pub parts: Vec<RecurRulePart>,
}

impl RecurrenceRule {
    pub fn new(freq: RecurFreq) -> Self {
        RecurrenceRule {
            parts: vec![RecurRulePart::Freq(freq)],
        }
    }

    pub(crate) fn empty_with_capacity(capacity: usize) -> Self {
        RecurrenceRule {
            parts: Vec::with_capacity(capacity),
        }
    }

    pub(crate) fn set_freq(mut self, freq: RecurFreq) -> Self {
        self.parts.push(RecurRulePart::Freq(freq));
        self
    }

    pub fn set_until(mut self, date: time::Date, time: Option<time::Time>, is_utc: bool) -> Self {
        self.parts
            .push(RecurRulePart::Until((date, time, is_utc).into()));
        self
    }

    pub fn set_count(mut self, count: u64) -> Self {
        self.parts.push(RecurRulePart::Count(count));
        self
    }

    pub fn set_interval(mut self, interval: u64) -> Self {
        self.parts.push(RecurRulePart::Interval(interval));
        self
    }

    pub fn set_by_second(mut self, by_second: Vec<u8>) -> Self {
        self.parts.push(RecurRulePart::BySecList(by_second));
        self
    }

    pub fn set_by_minute(mut self, by_minute: Vec<u8>) -> Self {
        self.parts.push(RecurRulePart::ByMinute(by_minute));
        self
    }

    pub fn set_by_hour(mut self, by_hour: Vec<u8>) -> Self {
        self.parts.push(RecurRulePart::ByHour(by_hour));
        self
    }

    pub fn set_by_day(mut self, by_day: Vec<OffsetWeekday>) -> Self {
        self.parts.push(RecurRulePart::ByDay(by_day));
        self
    }

    pub fn set_by_month_day(mut self, by_month_day: Vec<i8>) -> Self {
        self.parts.push(RecurRulePart::ByMonthDay(by_month_day));
        self
    }

    pub fn set_by_year_day(mut self, by_year_day: Vec<i16>) -> Self {
        self.parts.push(RecurRulePart::ByYearDay(by_year_day));
        self
    }

    pub fn set_by_week_number(mut self, by_week_number: Vec<i8>) -> Self {
        self.parts.push(RecurRulePart::ByWeekNumber(by_week_number));
        self
    }

    pub fn set_by_month(mut self, by_month: Vec<time::Month>) -> Self {
        self.parts.push(RecurRulePart::ByMonth(by_month));
        self
    }

    pub fn set_by_set_pos(mut self, by_set_pos: Vec<i16>) -> Self {
        self.parts.push(RecurRulePart::BySetPos(by_set_pos));
        self
    }

    pub fn set_week_start(mut self, week_start: Weekday) -> Self {
        self.parts.push(RecurRulePart::WeekStart(week_start));
        self
    }
}
