use crate::common::{OffsetWeekday, RecurFreq, Weekday};
use crate::parser::types::DateOrDateTime;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RecurRulePart {
    Freq(RecurFreq),
    Until(DateOrDateTime),
    Count(u64),
    Interval(u64),
    BySecList(Vec<u8>),
    ByMinute(Vec<u8>),
    ByHour(Vec<u8>),
    ByDay(Vec<OffsetWeekday>),
    ByMonthDay(Vec<i8>),
    ByYearDay(Vec<i16>),
    ByWeek(Vec<i8>),
    ByMonth(Vec<u8>),
    BySetPos(Vec<i16>),
    WeekStart(Weekday),
}
