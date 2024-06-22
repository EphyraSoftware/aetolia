use crate::model::object::ICalObjectBuilder;
use crate::model::param::Param;
use crate::model::param::{impl_other_component_params_builder, impl_other_params_builder};

pub trait AddComponentProperty {
    fn add_property(&mut self, property: ComponentProperty);
}

macro_rules! impl_finish_property_build {
    ($ev:expr) => {
        pub fn finish_property(mut self) -> ICalObjectBuilder {
            self.owner.inner.properties.push($ev(self.inner));
            self.owner
        }
    };
}

macro_rules! impl_finish_component_property_build {
    ($ev:expr) => {
        pub fn finish_property(mut self) -> P {
            self.owner.add_property($ev(self.inner));
            self.owner
        }
    };
}

pub enum CalendarProperty {
    ProductId(ProductIdProperty),
    Version(VersionProperty),
    CalendarScale(CalendarScaleProperty),
    Method(MethodProperty),
    XProperty(XProperty),
    IanaProperty(IanaProperty),
}

pub struct ProductIdProperty {
    params: Vec<Param>,
    value: String,
}

pub struct ProductIdPropertyBuilder {
    owner: ICalObjectBuilder,
    inner: ProductIdProperty,
}

impl ProductIdPropertyBuilder {
    pub(crate) fn new(owner: ICalObjectBuilder, value: String) -> ProductIdPropertyBuilder {
        ProductIdPropertyBuilder {
            owner,
            inner: ProductIdProperty {
                params: Vec::new(),
                value,
            },
        }
    }

    impl_finish_property_build!(CalendarProperty::ProductId);
}

impl_other_params_builder!(ProductIdPropertyBuilder);

pub struct VersionProperty {
    params: Vec<Param>,
    min_version: Option<String>,
    max_version: String,
}

pub struct VersionPropertyBuilder {
    owner: ICalObjectBuilder,
    inner: VersionProperty,
}

impl VersionPropertyBuilder {
    pub(crate) fn new(
        owner: ICalObjectBuilder,
        min_version: Option<String>,
        max_version: String,
    ) -> VersionPropertyBuilder {
        VersionPropertyBuilder {
            owner,
            inner: VersionProperty {
                params: Vec::new(),
                min_version,
                max_version,
            },
        }
    }

    impl_finish_property_build!(CalendarProperty::Version);
}

impl_other_params_builder!(VersionPropertyBuilder);

pub struct CalendarScaleProperty {
    params: Vec<Param>,
    value: String,
}

pub struct CalendarScalePropertyBuilder {
    owner: ICalObjectBuilder,
    inner: CalendarScaleProperty,
}

impl CalendarScalePropertyBuilder {
    pub(crate) fn new(owner: ICalObjectBuilder, value: String) -> CalendarScalePropertyBuilder {
        CalendarScalePropertyBuilder {
            owner,
            inner: CalendarScaleProperty {
                params: Vec::new(),
                value,
            },
        }
    }

    impl_finish_property_build!(CalendarProperty::CalendarScale);
}

impl_other_params_builder!(CalendarScalePropertyBuilder);

pub struct MethodProperty {
    params: Vec<Param>,
    value: String,
}

pub struct MethodPropertyBuilder {
    owner: ICalObjectBuilder,
    inner: MethodProperty,
}

impl MethodPropertyBuilder {
    pub(crate) fn new(owner: ICalObjectBuilder, value: String) -> MethodPropertyBuilder {
        MethodPropertyBuilder {
            owner,
            inner: MethodProperty {
                params: Vec::new(),
                value,
            },
        }
    }

    impl_finish_property_build!(CalendarProperty::Method);
}

impl_other_params_builder!(MethodPropertyBuilder);

pub enum ComponentProperty {
    DateTimeStamp(DateTimeStampProperty),
    IanaProperty(IanaProperty),
    XProperty(XProperty),
}

pub struct XProperty {
    pub(crate) params: Vec<Param>,
    name: String,
    value: String,
}

pub struct XPropertyBuilder {
    owner: ICalObjectBuilder,
    inner: XProperty,
}

impl XPropertyBuilder {
    pub(crate) fn new(owner: ICalObjectBuilder, name: String, value: String) -> XPropertyBuilder {
        XPropertyBuilder {
            owner,
            inner: XProperty {
                params: Vec::new(),
                name,
                value,
            },
        }
    }

    impl_finish_property_build!(CalendarProperty::XProperty);
}

impl_other_params_builder!(XPropertyBuilder);

pub struct IanaProperty {
    pub(crate) params: Vec<Param>,
    name: String,
    value: String,
}

pub struct IanaPropertyBuilder {
    owner: ICalObjectBuilder,
    inner: IanaProperty,
}

impl IanaPropertyBuilder {
    pub(crate) fn new(
        owner: ICalObjectBuilder,
        name: String,
        value: String,
    ) -> IanaPropertyBuilder {
        IanaPropertyBuilder {
            owner,
            inner: IanaProperty {
                params: Vec::new(),
                name,
                value,
            },
        }
    }

    impl_finish_property_build!(CalendarProperty::IanaProperty);
}

impl_other_params_builder!(IanaPropertyBuilder);

pub struct XComponentPropertyBuilder<P> {
    owner: P,
    inner: XProperty,
}

impl<P> XComponentPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, name: String, value: String) -> XComponentPropertyBuilder<P> {
        XComponentPropertyBuilder {
            owner,
            inner: XProperty {
                params: Vec::new(),
                name,
                value,
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::XProperty);
}

impl_other_component_params_builder!(XComponentPropertyBuilder<P>);

pub struct IanaComponentPropertyBuilder<P> {
    owner: P,
    inner: IanaProperty,
}

impl<P> IanaComponentPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(owner: P, name: String, value: String) -> IanaComponentPropertyBuilder<P> {
        IanaComponentPropertyBuilder {
            owner,
            inner: IanaProperty {
                params: Vec::new(),
                name,
                value,
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::IanaProperty);
}

impl_other_component_params_builder!(IanaComponentPropertyBuilder<P>);

pub struct DateTimeStampProperty {
    date: time::Date,
    time: time::Time,
    pub(crate) params: Vec<Param>,
}

pub struct DateTimeStampPropertyBuilder<P: AddComponentProperty> {
    owner: P,
    inner: DateTimeStampProperty,
}

impl<P> DateTimeStampPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    pub(crate) fn new(
        owner: P,
        date: time::Date,
        time: time::Time,
    ) -> DateTimeStampPropertyBuilder<P> {
        DateTimeStampPropertyBuilder {
            owner,
            inner: DateTimeStampProperty {
                date,
                time,
                params: Vec::new(),
            },
        }
    }

    impl_finish_component_property_build!(ComponentProperty::DateTimeStamp);
}

impl_other_component_params_builder!(DateTimeStampPropertyBuilder<P>);
