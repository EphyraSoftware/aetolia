use crate::parser::types::calendar_property::CalendarProperty;
use crate::parser::types::component::CalendarComponent;

#[derive(Debug)]
pub struct ICalendar<'a> {
    pub properties: Vec<CalendarProperty<'a>>,
    pub components: Vec<CalendarComponent<'a>>,
}
