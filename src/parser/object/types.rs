use crate::parser::property::types::{
    CalendarScaleProperty, IanaProperty, MethodProperty, ProductId, VersionProperty, XProperty,
};
use crate::parser::ContentLine;
use crate::parser::property::{DateTimeStamp, DateTimeStartProperty};

#[derive(Debug)]
pub struct ICalendar<'a> {
    pub properties: Vec<CalendarProperty<'a>>,
    pub components: Vec<CalendarComponent<'a>>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CalendarProperty<'a> {
    ProductId(ProductId<'a>),
    Version(VersionProperty<'a>),
    CalScale(CalendarScaleProperty<'a>),
    Method(MethodProperty<'a>),
    XProp(XProperty<'a>),
    IanaProp(IanaProperty<'a>),
}

#[derive(Debug, Eq, PartialEq)]
pub enum ComponentProperty<'a> {
    DateTimeStart(DateTimeStartProperty<'a>),
    DateTimeStamp(DateTimeStamp<'a>),
    XProp(XProperty<'a>),
    IanaProp(IanaProperty<'a>),
}

#[derive(Debug)]
pub enum CalendarComponent<'a> {
    Event {
        properties: Vec<ComponentProperty<'a>>,
    },
    IanaComp {
        name: &'a [u8],
        lines: Vec<ContentLine<'a>>,
    },
    XComp {
        name: &'a [u8],
        lines: Vec<ContentLine<'a>>,
    },
}
