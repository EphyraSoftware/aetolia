pub struct ICalObject {
    properties: Vec<CalendarProperty>,
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
        }
    }
}

pub struct ICalObjectBuilder {
    inner: ICalObject,
}

impl ICalObjectBuilder {
    pub fn add_product_id<V: ToString>(self, value: V) -> ProductIdBuilder {
        ProductIdBuilder::new(self, value.to_string())
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

macro_rules! impl_finish_build {
    ($ev:expr) => {
        fn finish_param(mut self) -> ICalObjectBuilder {
            self.owner.inner.properties.push($ev(self.inner));
            self.owner
        }
    };
}

pub struct ProductIdBuilder {
    owner: ICalObjectBuilder,
    inner: ProductIdProperty,
}

impl ProductIdBuilder {
    fn new(owner: ICalObjectBuilder, value: String) -> ProductIdBuilder {
        ProductIdBuilder {
            owner,
            inner: ProductIdProperty {
                params: Vec::new(),
                value,
            },
        }
    }

    impl_finish_build!(CalendarProperty::ProductId);
}

impl_other_params_builder!(ProductIdBuilder);

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

    impl_finish_build!(CalendarProperty::Version);
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

    impl_finish_build!(CalendarProperty::CalendarScale);
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

    impl_finish_build!(CalendarProperty::Method);
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

    impl_finish_build!(CalendarProperty::XProperty);
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

    impl_finish_build!(CalendarProperty::IanaProperty);
}

impl_other_params_builder!(IanaPropertyBuilder);

enum Param {
    AltRep { uri: String },
    CommonName { name: String },
    Other { name: String, value: String },
    Others { name: String, values: Vec<String> },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_cal_props_cal_object() {
        let obj = ICalObject::builder()
            .add_product_id("-//ABC Corporation//NONSGML My Product//EN")
            .add_x_param("x-special-param", "my-value")
            .finish_param()
            .add_max_version("2.0")
            .add_x_param_values(
                "x-special-param",
                vec!["one-value".to_string(), "another-value".to_string()],
            )
            .finish_param()
            .add_calendar_scale("GREGORIAN")
            .finish_param()
            .add_method("REQUEST")
            .finish_param()
            .add_x_property("X-PROP", "X-VALUE")
            .add_iana_param("special-param", "my-value")
            .finish_param()
            .add_iana_property("IANA-PARAM", "IANA-VALUE")
            .add_iana_param_values(
                "iana-special-param",
                vec!["one-value".to_string(), "another-value".to_string()],
            )
            .finish_param()
            .build();

        assert_eq!(obj.properties.len(), 6);
    }
}
