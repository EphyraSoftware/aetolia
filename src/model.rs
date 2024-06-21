pub struct ICalObject {
    properties: Vec<CalendarProperty>,
    components: Vec<CalendarComponent>,
}

impl ICalObject {
    pub fn builder() -> ICalObjectBuilder {
        ICalObjectBuilder {
            inner: ICalObject::new(),
        }
    }

    fn new() -> ICalObject {
        ICalObject {
            properties: Vec::new(),
            components: Vec::new(),
        }
    }
}

pub struct ICalObjectBuilder {
    inner: ICalObject,
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

    pub fn add_iana_component<N: ToString>(
        self,
        name: N,
        builder: fn(IanaComponentBuilder) -> ICalObjectBuilder,
    ) -> Self {
        builder(IanaComponentBuilder::new(self, name.to_string()))
    }

    pub fn add_x_component<N: ToString>(
        self,
        name: N,
        builder: fn(XComponentBuilder) -> ICalObjectBuilder,
    ) -> Self {
        builder(XComponentBuilder::new(self, name.to_string()))
    }

    fn build(self) -> ICalObject {
        self.inner
    }
}

trait OtherParamsBuilder {
    fn add_iana_param<N: ToString, V: ToString>(self, name: N, value: V) -> Self;

    fn add_iana_param_values<N: ToString>(self, name: N, values: Vec<String>) -> Self;

    fn add_x_param<N: ToString, V: ToString>(self, name: N, value: V) -> Self;

    fn add_x_param_values<N: ToString>(self, name: N, values: Vec<String>) -> Self;
}

macro_rules! impl_other_params_builder {
    ($builder:ty) => {
        impl OtherParamsBuilder for $builder {
            fn add_iana_param<N: ToString, V: ToString>(mut self, name: N, value: V) -> Self {
                self.inner.params.push(Param::Other {
                    name: name.to_string(),
                    value: value.to_string(),
                });
                self
            }

            fn add_iana_param_values<N: ToString>(mut self, name: N, values: Vec<String>) -> Self {
                self.inner.params.push(Param::Others {
                    name: name.to_string(),
                    values,
                });
                self
            }

            fn add_x_param<N: ToString, V: ToString>(mut self, name: N, value: V) -> Self {
                self.inner.params.push(Param::Other {
                    name: name.to_string(),
                    value: value.to_string(),
                });
                self
            }

            fn add_x_param_values<N: ToString>(mut self, name: N, values: Vec<String>) -> Self {
                self.inner.params.push(Param::Others {
                    name: name.to_string(),
                    values,
                });
                self
            }
        }
    };
}

macro_rules! impl_other_component_params_builder {
    ($builder:ident<$p:ident>) => {
        impl<$p> OtherParamsBuilder for $builder<$p>
        where
            $p: AddComponentProperty,
        {
            fn add_iana_param<N: ToString, V: ToString>(mut self, name: N, value: V) -> Self {
                self.inner.params.push(Param::Other {
                    name: name.to_string(),
                    value: value.to_string(),
                });
                self
            }

            fn add_iana_param_values<N: ToString>(mut self, name: N, values: Vec<String>) -> Self {
                self.inner.params.push(Param::Others {
                    name: name.to_string(),
                    values,
                });
                self
            }

            fn add_x_param<N: ToString, V: ToString>(mut self, name: N, value: V) -> Self {
                self.inner.params.push(Param::Other {
                    name: name.to_string(),
                    value: value.to_string(),
                });
                self
            }

            fn add_x_param_values<N: ToString>(mut self, name: N, values: Vec<String>) -> Self {
                self.inner.params.push(Param::Others {
                    name: name.to_string(),
                    values,
                });
                self
            }
        }
    };
}

macro_rules! impl_finish_property_build {
    ($ev:expr) => {
        fn finish_property(mut self) -> ICalObjectBuilder {
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

macro_rules! impl_finish_component_build {
    ($ev:expr) => {
        fn finish_component(mut self) -> ICalObjectBuilder {
            self.owner.inner.components.push($ev(self.inner));
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
    fn new(owner: ICalObjectBuilder, value: String) -> ProductIdPropertyBuilder {
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
    fn new(
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
    fn new(owner: ICalObjectBuilder, value: String) -> CalendarScalePropertyBuilder {
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
    fn new(owner: ICalObjectBuilder, value: String) -> MethodPropertyBuilder {
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

pub struct XProperty {
    params: Vec<Param>,
    name: String,
    value: String,
}

pub struct XPropertyBuilder {
    owner: ICalObjectBuilder,
    inner: XProperty,
}

impl XPropertyBuilder {
    fn new(owner: ICalObjectBuilder, name: String, value: String) -> XPropertyBuilder {
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
    params: Vec<Param>,
    name: String,
    value: String,
}

pub struct IanaPropertyBuilder {
    owner: ICalObjectBuilder,
    inner: IanaProperty,
}

impl IanaPropertyBuilder {
    fn new(owner: ICalObjectBuilder, name: String, value: String) -> IanaPropertyBuilder {
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

enum Param {
    AltRep { uri: String },
    CommonName { name: String },
    Other { name: String, value: String },
    Others { name: String, values: Vec<String> },
}

pub struct XComponentPropertyBuilder<P> {
    owner: P,
    inner: XProperty,
}

impl<P> XComponentPropertyBuilder<P>
where
    P: AddComponentProperty,
{
    fn new(owner: P, name: String, value: String) -> XComponentPropertyBuilder<P> {
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
    fn new(owner: P, name: String, value: String) -> IanaComponentPropertyBuilder<P> {
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

pub trait AddComponentProperty {
    fn add_property(&mut self, property: ComponentProperty);
}

pub struct XComponent {
    name: String,
    properties: Vec<ComponentProperty>,
}

impl AddComponentProperty for XComponentBuilder {
    fn add_property(&mut self, property: ComponentProperty) {
        self.inner.properties.push(property);
    }
}

macro_rules! impl_component_properties {
    ($builder:ident, $inner:ty) => {
        fn add_x_property<N: ToString, V: ToString>(self, name: N, value: V) -> $builder<$inner> {
            $builder::new(self, name.to_string(), value.to_string())
        }

        fn add_iana_property<N: ToString, V: ToString>(
            self,
            name: N,
            value: V,
        ) -> $builder<$inner> {
            $builder::new(self, name.to_string(), value.to_string())
        }
    };
}

pub struct XComponentBuilder {
    owner: ICalObjectBuilder,
    inner: XComponent,
}

impl XComponentBuilder {
    fn new(owner: ICalObjectBuilder, name: String) -> XComponentBuilder {
        XComponentBuilder {
            owner,
            inner: XComponent {
                name,
                properties: Vec::new(),
            },
        }
    }

    impl_component_properties!(XComponentPropertyBuilder, XComponentBuilder);

    impl_finish_component_build!(CalendarComponent::XComponent);
}

pub struct IanaComponent {
    name: String,
    properties: Vec<ComponentProperty>,
}

impl AddComponentProperty for IanaComponentBuilder {
    fn add_property(&mut self, property: ComponentProperty) {
        self.inner.properties.push(property);
    }
}

pub struct IanaComponentBuilder {
    owner: ICalObjectBuilder,
    inner: IanaComponent,
}

impl IanaComponentBuilder {
    fn new(owner: ICalObjectBuilder, name: String) -> IanaComponentBuilder {
        IanaComponentBuilder {
            owner,
            inner: IanaComponent {
                name,
                properties: Vec::new(),
            },
        }
    }

    impl_component_properties!(IanaComponentPropertyBuilder, IanaComponentBuilder);

    impl_finish_component_build!(CalendarComponent::IanaComponent);
}

pub enum CalendarComponent {
    // Event {
    //     properties: Vec<CalendarProperty>,
    // },
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

pub enum ComponentProperty {
    IanaProperty(IanaProperty),
    XProperty(XProperty),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_cal_props_cal_object() {
        let obj = ICalObject::builder()
            .add_product_id("-//ABC Corporation//NONSGML My Product//EN")
            .add_x_param("x-special-param", "my-value")
            .finish_property()
            .add_max_version("2.0")
            .add_x_param_values(
                "x-special-param",
                vec!["one-value".to_string(), "another-value".to_string()],
            )
            .finish_property()
            .add_calendar_scale("GREGORIAN")
            .finish_property()
            .add_method("REQUEST")
            .finish_property()
            .add_x_property("X-PROP", "X-VALUE")
            .add_iana_param("special-param", "my-value")
            .finish_property()
            .add_iana_property("IANA-PARAM", "IANA-VALUE")
            .add_iana_param_values(
                "iana-special-param",
                vec!["one-value".to_string(), "another-value".to_string()],
            )
            .finish_property()
            .build();

        assert_eq!(obj.properties.len(), 6);
    }

    #[test]
    fn x_component_cal_object() {
        let obj = ICalObject::builder()
            .add_product_id("x_component_cal_object")
            .finish_property()
            .add_x_component("X-SOME-COMPONENT", |b| {
                b.add_x_property("X-SOME-PROP", "X-SOME-VALUE")
                    .add_x_param("x-special-param", "my-value")
                    .add_iana_param("special-param", "my-value")
                    .finish_property()
                    .finish_component()
            })
            .add_iana_component("IANA-SOME-COMPONENT", |b| {
                b.add_iana_property("IANA-SOME-PROP", "IANA-SOME-VALUE")
                    .add_iana_param("special-param", "my-value")
                    .add_x_param("x-special-param", "my-value")
                    .finish_property()
                    .finish_component()
            })
            .build();

        assert_eq!(obj.components.len(), 2);

        match &obj.components[0] {
            CalendarComponent::XComponent(x) => {
                assert_eq!(x.properties.len(), 1);
                match &x.properties[0] {
                    ComponentProperty::XProperty(p) => {
                        assert_eq!(p.params.len(), 2);
                    }
                    _ => panic!("Expected XProperty"),
                }
            }
            _ => panic!("Expected XComponent"),
        }

        match &obj.components[1] {
            CalendarComponent::IanaComponent(x) => {
                assert_eq!(x.properties.len(), 1);
                match &x.properties[0] {
                    ComponentProperty::IanaProperty(p) => {
                        assert_eq!(p.params.len(), 2);
                    }
                    _ => panic!("Expected IanaProperty"),
                }
            }
            _ => panic!("Expected IanaComponent"),
        }
    }
}
