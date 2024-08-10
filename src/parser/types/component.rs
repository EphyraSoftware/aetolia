use crate::parser::types::component_property::ComponentProperty;
use crate::parser::ContentLine;

#[derive(Debug, PartialEq)]
pub enum CalendarComponent<'a> {
    Event {
        properties: Vec<ComponentProperty<'a>>,
        alarms: Vec<CalendarComponent<'a>>,
    },
    ToDo {
        properties: Vec<ComponentProperty<'a>>,
        alarms: Vec<CalendarComponent<'a>>,
    },
    Journal {
        properties: Vec<ComponentProperty<'a>>,
    },
    FreeBusy {
        properties: Vec<ComponentProperty<'a>>,
    },
    Standard {
        properties: Vec<ComponentProperty<'a>>,
    },
    Daylight {
        properties: Vec<ComponentProperty<'a>>,
    },
    TimeZone {
        properties: Vec<ComponentProperty<'a>>,
        components: Vec<CalendarComponent<'a>>,
    },
    Alarm {
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
