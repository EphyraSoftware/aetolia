pub struct ICalendar<'a> {
    pub properties: Vec<CalendarProperty>,
    pub components: Vec<CalendarComponent<'a>>,
}

pub enum CalendarProperty {
    ProductId(String),
    Version(String),
    CalScale,
    Method,
    XProp,
    IanaProp,
}

pub enum CalendarComponent<'a> {
    IanaComp { name: &'a [u8], lines: Vec<Vec<u8>> },
    XComp { name: &'a [u8], lines: Vec<Vec<u8>> },
}
