use crate::model::component::CalendarComponent;
use crate::model::component::IanaComponentBuilder;
use crate::model::component::XComponentBuilder;
use crate::model::component::{
    EventComponentBuilder, FreeBusyComponentBuilder, JournalComponentBuilder,
    TimeZoneComponentBuilder, ToDoComponentBuilder,
};
use crate::model::property::{
    CalendarProperty, CalendarScalePropertyBuilder, IanaPropertyBuilder, MethodPropertyBuilder,
    ProductIdPropertyBuilder, VersionPropertyBuilder, XPropertyBuilder,
};

#[derive(Debug, PartialEq)]
pub struct ICalObject {
    pub properties: Vec<CalendarProperty>,
    pub components: Vec<CalendarComponent>,
}

impl ICalObject {
    pub fn builder() -> ICalObjectBuilder {
        ICalObjectBuilder {
            inner: ICalObject::new(),
        }
    }

    pub(crate) fn new() -> ICalObject {
        ICalObject {
            properties: Vec::new(),
            components: Vec::new(),
        }
    }
}

pub struct ICalObjectBuilder {
    pub(crate) inner: ICalObject,
}

impl ICalObjectBuilder {
    pub fn add_product_id<V: ToString>(self, value: V) -> ProductIdPropertyBuilder {
        ProductIdPropertyBuilder::new(self, value.to_string())
    }

    pub fn add_version_range<U: ToString, V: ToString>(
        self,
        min_version: U,
        max_version: V,
    ) -> VersionPropertyBuilder {
        VersionPropertyBuilder::new(self, Some(min_version.to_string()), max_version.to_string())
    }

    pub fn add_max_version<V: ToString>(self, max_version: V) -> VersionPropertyBuilder {
        VersionPropertyBuilder::new(self, None, max_version.to_string())
    }

    pub fn add_calendar_scale<V: ToString>(self, value: V) -> CalendarScalePropertyBuilder {
        CalendarScalePropertyBuilder::new(self, value.to_string())
    }

    pub fn add_method<V: ToString>(self, value: V) -> MethodPropertyBuilder {
        MethodPropertyBuilder::new(self, value.to_string())
    }

    pub fn add_x_property<N: ToString, V: ToString>(self, name: N, value: V) -> XPropertyBuilder {
        XPropertyBuilder::new(self, name.to_string(), value.to_string())
    }

    pub fn add_iana_property<N: ToString, V: ToString>(
        self,
        name: N,
        value: V,
    ) -> IanaPropertyBuilder {
        IanaPropertyBuilder::new(self, name.to_string(), value.to_string())
    }

    pub fn add_event_component(self) -> EventComponentBuilder {
        EventComponentBuilder::new(self)
    }

    pub fn add_to_do_component(self) -> ToDoComponentBuilder {
        ToDoComponentBuilder::new(self)
    }

    pub fn add_journal_component(self) -> JournalComponentBuilder {
        JournalComponentBuilder::new(self)
    }

    pub fn add_free_busy_component(self) -> FreeBusyComponentBuilder {
        FreeBusyComponentBuilder::new(self)
    }

    pub fn add_time_zone_component(self) -> TimeZoneComponentBuilder {
        TimeZoneComponentBuilder::new(self)
    }

    pub fn add_iana_component<N: ToString>(
        self,
        name: N,
        builder: fn(IanaComponentBuilder) -> IanaComponentBuilder,
    ) -> Self {
        builder(IanaComponentBuilder::new(self, name.to_string())).finish_component()
    }

    pub fn add_x_component<N: ToString>(
        self,
        name: N,
        builder: fn(XComponentBuilder) -> XComponentBuilder,
    ) -> Self {
        builder(XComponentBuilder::new(self, name.to_string())).finish_component()
    }

    pub fn build(self) -> ICalObject {
        self.inner
    }
}
