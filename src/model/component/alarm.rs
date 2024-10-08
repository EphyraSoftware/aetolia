use crate::model::component::{
    add_action, add_attach, add_description, add_duration, add_repeat, add_summary, add_trigger,
    impl_other_component_properties,
};
use crate::model::impl_component_access;
use crate::model::param::ParticipationStatusEvent;
use crate::model::property::{
    Action, AddComponentProperty, AttendeePropertyBuilder, ComponentProperty,
    IanaComponentPropertyBuilder, XComponentPropertyBuilder,
};

#[derive(Debug, PartialEq)]
pub struct AlarmComponent {
    pub(crate) properties: Vec<ComponentProperty>,
}

impl_component_access!(AlarmComponent);

impl AlarmComponent {
    pub(crate) fn new() -> Self {
        AlarmComponent {
            properties: Vec::new(),
        }
    }
}

impl Default for AlarmComponent {
    fn default() -> Self {
        Self::new()
    }
}

pub trait AddAlarmComponent {
    fn add_alarm(self, alarm: AlarmComponent) -> Self;
}

pub struct AudioAlarmComponentBuilder<P: AddAlarmComponent> {
    owner: P,
    inner: AlarmComponent,
}

impl<P> AudioAlarmComponentBuilder<P>
where
    P: AddAlarmComponent,
{
    pub(crate) fn new(owner: P) -> Self {
        AudioAlarmComponentBuilder {
            owner,
            inner: AlarmComponent {
                properties: Vec::new(),
            },
        }
    }

    add_action!(Action::Audio);

    add_trigger!();

    add_duration!();

    add_repeat!();

    add_attach!();

    impl_other_component_properties!(
        XComponentPropertyBuilder,
        IanaComponentPropertyBuilder,
        AudioAlarmComponentBuilder<P>
    );

    pub fn finish_component(self) -> P {
        self.owner.add_alarm(self.inner)
    }
}

impl<P> AddComponentProperty for AudioAlarmComponentBuilder<P>
where
    P: AddAlarmComponent,
{
    fn add_property(&mut self, property: ComponentProperty) {
        self.inner.properties.push(property);
    }
}

pub struct DisplayAlarmComponentBuilder<P: AddAlarmComponent> {
    owner: P,
    inner: AlarmComponent,
}

impl<P> DisplayAlarmComponentBuilder<P>
where
    P: AddAlarmComponent,
{
    pub(crate) fn new(owner: P) -> Self {
        DisplayAlarmComponentBuilder {
            owner,
            inner: AlarmComponent {
                properties: Vec::new(),
            },
        }
    }

    add_action!(Action::Display);

    add_description!();

    add_trigger!();

    add_duration!();

    add_repeat!();

    impl_other_component_properties!(
        XComponentPropertyBuilder,
        IanaComponentPropertyBuilder,
        DisplayAlarmComponentBuilder<P>
    );

    pub fn finish_component(self) -> P {
        self.owner.add_alarm(self.inner)
    }
}

impl<P> AddComponentProperty for DisplayAlarmComponentBuilder<P>
where
    P: AddAlarmComponent,
{
    fn add_property(&mut self, property: ComponentProperty) {
        self.inner.properties.push(property);
    }
}

pub struct EmailAlarmComponentBuilder<P: AddAlarmComponent> {
    owner: P,
    inner: AlarmComponent,
}

impl<P> EmailAlarmComponentBuilder<P>
where
    P: AddAlarmComponent,
{
    pub(crate) fn new(owner: P) -> Self {
        EmailAlarmComponentBuilder {
            owner,
            inner: AlarmComponent {
                properties: Vec::new(),
            },
        }
    }

    add_action!(Action::Email);

    add_description!();

    add_trigger!();

    add_summary!();

    pub fn add_attendee(
        self,
        value: &str,
    ) -> AttendeePropertyBuilder<Self, ParticipationStatusEvent> {
        AttendeePropertyBuilder::new(self, value.to_string())
    }

    add_duration!();

    add_repeat!();

    add_attach!();

    impl_other_component_properties!(
        XComponentPropertyBuilder,
        IanaComponentPropertyBuilder,
        EmailAlarmComponentBuilder<P>
    );

    pub fn finish_component(self) -> P {
        self.owner.add_alarm(self.inner)
    }
}

impl<P> AddComponentProperty for EmailAlarmComponentBuilder<P>
where
    P: AddAlarmComponent,
{
    fn add_property(&mut self, property: ComponentProperty) {
        self.inner.properties.push(property);
    }
}
