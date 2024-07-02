use crate::common::{OffsetWeekday, RecurFreq, Weekday};

pub struct RecurrenceRule {
    pub freq: RecurFreq,
    pub until: Option<(time::Date, Option<time::Time>)>,
    pub count: Option<u64>,
    pub interval: Option<u64>,
    pub by_second: Option<Vec<u8>>,
    pub by_minute: Option<Vec<u8>>,
    pub by_hour: Option<Vec<u8>>,
    pub by_day: Option<Vec<OffsetWeekday>>,
    pub by_month_day: Option<Vec<i8>>,
    pub by_year_day: Option<Vec<i16>>,
    pub by_week_number: Option<Vec<i8>>,
    pub by_month: Option<Vec<time::Month>>,
    pub by_set_pos: Option<Vec<i16>>,
    pub week_start: Option<Weekday>,
}

impl RecurrenceRule {
    pub fn new(freq: RecurFreq) -> Self {
        RecurrenceRule {
            freq,
            until: None,
            count: None,
            interval: None,
            by_second: None,
            by_minute: None,
            by_hour: None,
            by_day: None,
            by_month_day: None,
            by_year_day: None,
            by_week_number: None,
            by_month: None,
            by_set_pos: None,
            week_start: None,
        }
    }

    pub fn set_until(mut self, date: time::Date, time: Option<time::Time>) -> Self {
        self.until = Some((date, time));
        self
    }

    pub fn set_count(mut self, count: u64) -> Self {
        self.count = Some(count);
        self
    }

    pub fn set_interval(mut self, interval: u64) -> Self {
        self.interval = Some(interval);
        self
    }

    pub fn set_by_second(mut self, by_second: Vec<u8>) -> Self {
        self.by_second = Some(by_second);
        self
    }

    pub fn set_by_minute(mut self, by_minute: Vec<u8>) -> Self {
        self.by_minute = Some(by_minute);
        self
    }

    pub fn set_by_hour(mut self, by_hour: Vec<u8>) -> Self {
        self.by_hour = Some(by_hour);
        self
    }

    pub fn set_by_day(mut self, by_day: Vec<OffsetWeekday>) -> Self {
        self.by_day = Some(by_day);
        self
    }

    pub fn set_by_month_day(mut self, by_month_day: Vec<i8>) -> Self {
        self.by_month_day = Some(by_month_day);
        self
    }

    pub fn set_by_year_day(mut self, by_year_day: Vec<i16>) -> Self {
        self.by_year_day = Some(by_year_day);
        self
    }

    pub fn set_by_week_number(mut self, by_week_number: Vec<i8>) -> Self {
        self.by_week_number = Some(by_week_number);
        self
    }

    pub fn set_by_month(mut self, by_month: Vec<time::Month>) -> Self {
        self.by_month = Some(by_month);
        self
    }

    pub fn set_by_set_pos(mut self, by_set_pos: Vec<i16>) -> Self {
        self.by_set_pos = Some(by_set_pos);
        self
    }

    pub fn set_week_start(mut self, week_start: Weekday) -> Self {
        self.week_start = Some(week_start);
        self
    }
}
