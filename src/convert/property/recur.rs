use crate::convert::ToModel;
use crate::model::RecurrenceRule;
use crate::parser::RecurRulePart;
use anyhow::Context;

impl ToModel for Vec<RecurRulePart> {
    type Model = RecurrenceRule;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        let mut rule = RecurrenceRule::empty_with_capacity(self.len());
        for part in self.iter() {
            match part {
                RecurRulePart::Freq(f) => {
                    rule = rule.set_freq(f.clone());
                }
                RecurRulePart::Until(date_time) => {
                    let (date, maybe_time, is_utc) = date_time.to_model()?;
                    rule = rule.set_until(date, maybe_time, is_utc);
                }
                RecurRulePart::Count(count) => {
                    rule = rule.set_count(*count);
                }
                RecurRulePart::Interval(interval) => {
                    rule = rule.set_interval(*interval);
                }
                RecurRulePart::BySecList(by_sec_list) => {
                    rule = rule.set_by_second(by_sec_list.clone());
                }
                RecurRulePart::ByMinute(by_minute) => {
                    rule = rule.set_by_minute(by_minute.clone());
                }
                RecurRulePart::ByHour(by_hour) => {
                    rule = rule.set_by_hour(by_hour.clone());
                }
                RecurRulePart::ByDay(by_day) => {
                    rule = rule.set_by_day(by_day.clone());
                }
                RecurRulePart::ByMonthDay(by_month_day) => {
                    rule = rule.set_by_month_day(by_month_day.clone());
                }
                RecurRulePart::ByYearDay(by_year_day) => {
                    rule = rule.set_by_year_day(by_year_day.clone());
                }
                RecurRulePart::ByWeek(week) => {
                    rule = rule.set_by_week_number(week.clone());
                }
                RecurRulePart::ByMonth(month) => {
                    rule = rule.set_by_month(
                        month
                            .iter()
                            .map(|m| time::Month::try_from(*m).context("Invalid month"))
                            .collect::<anyhow::Result<Vec<_>>>()?,
                    );
                }
                RecurRulePart::BySetPos(by_set_pos) => {
                    rule = rule.set_by_set_pos(by_set_pos.clone());
                }
                RecurRulePart::WeekStart(week_start) => {
                    rule = rule.set_week_start(week_start.clone());
                }
            }
        }

        Ok(rule)
    }
}
