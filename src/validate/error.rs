use std::fmt::{Display, Formatter};

#[derive(Clone, PartialEq, Debug)]
pub enum ICalendarErrorSeverity {
    /// Invalid according to the iCalendar specification.
    Error,
    /// Non-fatal issue that could be fixed but can be ignored.
    ///
    /// For example, redundant VALUE parameters or unnecessary WKST properties in an RRULE.
    Warning,
}

#[derive(Clone)]
pub struct ICalendarError {
    pub message: String,
    pub severity: ICalendarErrorSeverity,
    pub location: Option<ICalendarLocation>,
}

impl Display for ICalendarError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(location) = &self.location {
            match location {
                ICalendarLocation::CalendarProperty(cp) => {
                    write!(
                        f,
                        "In calendar property \"{}\" at index {}",
                        cp.name, cp.index
                    )?;
                }
                ICalendarLocation::Component(component) => {
                    write!(
                        f,
                        "In component \"{}\" at index {}",
                        component.name, component.index
                    )?;
                    if let Some(within) = &component.location {
                        match &**within {
                            WithinComponentLocation::Property(cp) => {
                                write!(
                                    f,
                                    ", in component property \"{}\" at index {}",
                                    cp.name, cp.index
                                )?;
                            }
                            WithinComponentLocation::Component(nested_component_location) => {
                                write!(
                                    f,
                                    ", in nested component \"{}\" at index {}",
                                    nested_component_location.name, nested_component_location.index
                                )?;

                                if let Some(nested_within) = &nested_component_location.location {
                                    if let WithinComponentLocation::Property(cp) = &**nested_within
                                    {
                                        write!(
                                            f,
                                            ", in nested component property \"{}\" at index {}",
                                            cp.name, cp.index
                                        )?;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            write!(f, ": {}", self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl ICalendarError {
    pub(super) fn many_from_calendar_property_errors(
        errors: Vec<CalendarPropertyError>,
    ) -> Vec<Self> {
        errors
            .into_iter()
            .map(|error| ICalendarError {
                message: error.message,
                severity: error.severity,
                location: error.location.map(ICalendarLocation::CalendarProperty),
            })
            .collect()
    }

    pub(super) fn many_from_component_property_errors(
        errors: Vec<ComponentPropertyError>,
        index: usize,
        name: String,
    ) -> Vec<Self> {
        errors
            .into_iter()
            .map(|error| ICalendarError {
                message: error.message,
                severity: error.severity,
                location: Some(ICalendarLocation::Component(ComponentLocation {
                    index,
                    name: name.clone(),
                    location: error
                        .location
                        .map(|l| Box::new(WithinComponentLocation::Property(l))),
                })),
            })
            .collect()
    }

    pub(super) fn many_from_nested_component_property_errors(
        errors: Vec<ComponentPropertyError>,
        index: usize,
        name: String,
        nested_index: usize,
        nested_name: String,
    ) -> Vec<Self> {
        errors
            .into_iter()
            .map(|error| ICalendarError {
                message: error.message,
                severity: error.severity,
                location: Some(ICalendarLocation::Component(ComponentLocation {
                    index,
                    name: name.clone(),
                    location: Some(
                        WithinComponentLocation::Component(ComponentLocation {
                            index: nested_index,
                            name: nested_name.clone(),
                            location: error
                                .location
                                .map(|l| Box::new(WithinComponentLocation::Property(l))),
                        })
                        .into(),
                    ),
                })),
            })
            .collect()
    }
}

#[derive(Clone)]
pub enum ICalendarLocation {
    CalendarProperty(CalendarPropertyLocation),
    Component(ComponentLocation),
}

#[derive(Clone)]
pub struct ComponentLocation {
    pub index: usize,
    pub name: String,
    pub location: Option<Box<WithinComponentLocation>>,
}

#[derive(Clone)]
pub enum WithinComponentLocation {
    Property(ComponentPropertyLocation),
    Component(ComponentLocation),
}

#[derive(Clone)]
pub struct CalendarPropertyError {
    pub message: String,
    pub severity: ICalendarErrorSeverity,
    pub location: Option<CalendarPropertyLocation>,
}

impl CalendarPropertyError {
    pub(super) fn many_from_param_errors(
        errors: Vec<ParamError>,
        index: usize,
        name: String,
    ) -> Vec<Self> {
        errors
            .into_iter()
            .map(|error| CalendarPropertyError {
                message: error.message,
                severity: error.severity,
                location: Some(CalendarPropertyLocation {
                    index,
                    name: name.clone(),
                    property_location: Some(WithinPropertyLocation::Param {
                        index: error.index,
                        name: error.name,
                    }),
                }),
            })
            .collect()
    }
}

#[derive(Clone)]
pub struct CalendarPropertyLocation {
    pub index: usize,
    pub name: String,
    pub property_location: Option<WithinPropertyLocation>,
}

#[derive(Clone)]
pub struct ComponentPropertyError {
    pub message: String,
    pub severity: ICalendarErrorSeverity,
    pub location: Option<ComponentPropertyLocation>,
}

impl ComponentPropertyError {
    pub(super) fn many_from_param_errors(
        errors: Vec<ParamError>,
        index: usize,
        name: String,
    ) -> Vec<Self> {
        errors
            .into_iter()
            .map(|error| ComponentPropertyError {
                message: error.message,
                severity: error.severity,
                location: Some(ComponentPropertyLocation {
                    index,
                    name: name.clone(),
                    property_location: Some(WithinPropertyLocation::Param {
                        index: error.index,
                        name: error.name,
                    }),
                }),
            })
            .collect()
    }
}

#[derive(Clone)]
pub struct ComponentPropertyLocation {
    pub index: usize,
    pub name: String,
    pub property_location: Option<WithinPropertyLocation>,
}

#[derive(Clone)]
pub enum WithinPropertyLocation {
    Param { index: usize, name: String },
    Value,
}

pub struct ParamError {
    pub message: String,
    pub severity: ICalendarErrorSeverity,
    pub index: usize,
    pub name: String,
}
