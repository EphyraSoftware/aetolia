pub mod alarm;
pub mod event;
pub mod iana_component;
pub mod x_component;

pub enum CalendarComponent {
    Event(EventComponent),
    // ToDo {
    //     properties: Vec<CalendarProperty>,
    // },
    // Journal {
    //     properties: Vec<CalendarProperty>,
    // },
    // FreeBusy {
    //     properties: Vec<CalendarProperty>,
    // },
    // Timezone {
    //     properties: Vec<CalendarProperty>,
    // },
    // Alarm {
    //     properties: Vec<CalendarProperty>,
    // },
    IanaComponent(IanaComponent),
    XComponent(XComponent),
}

macro_rules! impl_finish_component_build {
    ($ev:expr) => {
        pub fn finish_component(mut self) -> ICalObjectBuilder {
            self.owner.inner.components.push($ev(self.inner));
            self.owner
        }
    };
}

pub(crate) use impl_finish_component_build;

macro_rules! impl_other_component_properties {
    ($x_builder:ident, $iana_builder:ident, $inner:ty) => {
        pub fn add_x_property<N: ToString, V: ToString>(
            self,
            name: N,
            value: V,
        ) -> $x_builder<$inner> {
            $x_builder::new(self, name.to_string(), value.to_string())
        }

        pub fn add_iana_property<N: ToString, V: ToString>(
            self,
            name: N,
            value: V,
        ) -> $iana_builder<$inner> {
            $iana_builder::new(self, name.to_string(), value.to_string())
        }
    };
}

pub(crate) use impl_other_component_properties;

use crate::model::component::event::EventComponent;
use crate::model::component::iana_component::IanaComponent;
use crate::model::component::x_component::XComponent;
