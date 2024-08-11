use crate::parser::types::{CalendarComponent, CalendarProperty, ParamValue};

#[derive(Debug)]
pub struct ICalendar<'a> {
    pub properties: Vec<CalendarProperty<'a>>,
    pub components: Vec<CalendarComponent<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct ContentLine<'a> {
    pub(crate) property_name: &'a [u8],
    pub(crate) params: Vec<ParamValue<'a>>,
    pub(crate) value: Vec<u8>,
}
