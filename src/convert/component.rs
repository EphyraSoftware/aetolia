use crate::convert::{convert_string, ToModel};
use crate::model::event::EventComponent;
use crate::model::{
    AlarmComponent, ComponentProperty, DaylightComponent, FreeBusyComponent, JournalComponent,
    StandardComponent, TimeZoneComponent, ToDoComponent,
};
use crate::parser::types::ContentLine;

impl ToModel for crate::parser::types::CalendarComponent<'_> {
    type Model = crate::model::CalendarComponent;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        match self {
            crate::parser::types::CalendarComponent::Event { properties, alarms } => {
                let mut component = EventComponent::new();
                component.properties.reserve(properties.len());

                for property in properties {
                    component.properties.push(property.to_model()?);
                }

                component.alarms.reserve(alarms.len());
                for alarm in alarms {
                    component.alarms.push(alarm.to_model()?);
                }

                Ok(crate::model::CalendarComponent::Event(component))
            }
            crate::parser::types::CalendarComponent::ToDo { properties, alarms } => {
                let mut component = ToDoComponent::new();
                component.properties.reserve(properties.len());

                for property in properties {
                    component.properties.push(property.to_model()?);
                }

                component.alarms.reserve(alarms.len());
                for alarm in alarms {
                    component.alarms.push(alarm.to_model()?);
                }

                Ok(crate::model::CalendarComponent::ToDo(component))
            }
            crate::parser::types::CalendarComponent::Journal { properties } => {
                let mut journal = JournalComponent::new();
                journal.properties.reserve(properties.len());

                for property in properties {
                    journal.properties.push(property.to_model()?);
                }

                Ok(crate::model::CalendarComponent::Journal(journal))
            }
            crate::parser::types::CalendarComponent::FreeBusy { properties } => {
                let mut free_busy = FreeBusyComponent::new();
                free_busy.properties.reserve(properties.len());

                for property in properties {
                    free_busy.properties.push(property.to_model()?);
                }

                Ok(crate::model::CalendarComponent::FreeBusy(free_busy))
            }
            crate::parser::types::CalendarComponent::Standard { properties } => {
                let mut standard = StandardComponent::new();
                standard.properties.reserve(properties.len());

                for property in properties {
                    standard.properties.push(property.to_model()?);
                }

                Ok(crate::model::CalendarComponent::Standard(standard))
            }
            crate::parser::types::CalendarComponent::Daylight { properties } => {
                let mut daylight = DaylightComponent::new();
                daylight.properties.reserve(properties.len());

                for property in properties {
                    daylight.properties.push(property.to_model()?);
                }

                Ok(crate::model::CalendarComponent::Daylight(daylight))
            }
            crate::parser::types::CalendarComponent::TimeZone {
                properties,
                components,
            } => {
                let mut timezone = TimeZoneComponent::new();
                timezone.properties.reserve(properties.len());

                for property in properties {
                    timezone.properties.push(property.to_model()?);
                }

                timezone.components.reserve(components.len());
                for component in components {
                    timezone.components.push(component.to_model()?);
                }

                Ok(crate::model::CalendarComponent::TimeZone(timezone))
            }
            crate::parser::types::CalendarComponent::Alarm { properties } => {
                let mut alarm = AlarmComponent::new();
                alarm.properties.reserve(properties.len());

                for property in properties {
                    alarm.properties.push(property.to_model()?);
                }

                Ok(crate::model::CalendarComponent::Alarm(alarm))
            }
            crate::parser::types::CalendarComponent::IanaComp { name, lines } => {
                let mut component = crate::model::IanaComponent::new(convert_string(name));
                component.properties.reserve(lines.len());

                map_unknown_lines(lines, &mut component.properties)?;

                Ok(crate::model::CalendarComponent::IanaComponent(component))
            }
            crate::parser::types::CalendarComponent::XComp { name, lines } => {
                let mut component = crate::model::XComponent::new(convert_string(name));
                component.properties.reserve(lines.len());

                map_unknown_lines(lines, &mut component.properties)?;

                Ok(crate::model::CalendarComponent::XComponent(component))
            }
        }
    }
}

fn map_unknown_lines(
    lines: &Vec<ContentLine>,
    component_properties: &mut Vec<ComponentProperty>,
) -> anyhow::Result<()> {
    for line in lines {
        let m = line.to_model()?;
        if m.name.starts_with("X-") || m.name.starts_with("x-") {
            component_properties.push(ComponentProperty::XProperty(crate::model::XProperty {
                name: m.name,
                value: m.value,
                params: m.params,
            }));
        } else {
            component_properties.push(ComponentProperty::IanaProperty(m));
        }
    }

    Ok(())
}
