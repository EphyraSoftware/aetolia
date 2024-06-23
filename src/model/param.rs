pub enum Param {
    AltRep { uri: String },
    CommonName { name: String },
    Value { value: Value },
    TimeZoneId { tz_id: String, unique: bool },
    AlternateRepresentation { value: String },
    Language { language: String },
    DirectoryEntryReference { value: String },
    SentBy { value: String },
    Other { name: String, value: String },
    Others { name: String, values: Vec<String> },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Value {
    Binary,
    Boolean,
    CalendarAddress,
    Date,
    DateTime,
    Duration,
    Float,
    Integer,
    Period,
    Recurrence,
    Text,
    Time,
    Uri,
    UtcOffset,
    XName(String),
    IanaToken(String),
}

pub trait OtherParamsBuilder {
    fn add_iana_param<N: ToString, V: ToString>(self, name: N, value: V) -> Self;

    fn add_iana_param_values<N: ToString>(self, name: N, values: Vec<String>) -> Self;

    fn add_x_param<N: ToString, V: ToString>(self, name: N, value: V) -> Self;

    fn add_x_param_values<N: ToString>(self, name: N, values: Vec<String>) -> Self;
}

macro_rules! impl_other_params_builder {
    ($builder:ty) => {
        impl crate::model::param::OtherParamsBuilder for $builder {
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

pub(crate) use impl_other_params_builder;

macro_rules! impl_other_component_params_builder {
    ($builder:ident<$p:ident>) => {
        impl<$p> crate::model::param::OtherParamsBuilder for $builder<$p>
        where
            $p: crate::model::property::AddComponentProperty,
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

pub(crate) use impl_other_component_params_builder;

macro_rules! altrep_param {
    () => {
        // TODO no generic URI representation for Rust? Maybe extract the URI parser in this crate and
        //      make that into a URI crate.
        pub fn add_alternate_representation(mut self, value: String) -> Self {
            self.inner
                .params
                .push(Param::AlternateRepresentation { value });
            self
        }
    };
}

pub(crate) use altrep_param;

macro_rules! language_param {
    () => {
        pub fn add_language(mut self, language: String) -> Self {
            self.inner.params.push(Param::Language { language });
            self
        }
    };
}

pub(crate) use language_param;
