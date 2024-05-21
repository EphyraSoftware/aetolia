use crate::parser::property::types::{ProductId, VersionProperty};
use crate::parser::ContentLine;

#[derive(Debug)]
pub struct ICalendar<'a> {
    pub properties: Vec<CalendarProperty<'a>>,
    pub components: Vec<CalendarComponent<'a>>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CalendarProperty<'a> {
    ProductId(ProductId<'a>),
    Version(VersionProperty<'a>),
    CalScale,
    Method,
    XProp,
    IanaProp,
}

#[derive(Debug)]
pub enum CalendarComponent<'a> {
    IanaComp {
        name: &'a [u8],
        lines: Vec<ContentLine<'a>>,
    },
    XComp {
        name: &'a [u8],
        lines: Vec<ContentLine<'a>>,
    },
}
