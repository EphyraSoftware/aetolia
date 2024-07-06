pub enum Param {
    AltRep {
        uri: String,
    },
    CommonName {
        name: String,
    },
    ValueType {
        value: Value,
    },
    TimeZoneId {
        tz_id: String,
        unique: bool,
    },
    Language {
        language: LanguageTag,
    },
    DirectoryEntryReference {
        uri: String,
    },
    SentBy {
        address: String,
    },
    Range {
        range: Range,
    },
    FormatType {
        type_name: String,
        sub_type_name: String,
    },
    Encoding {
        encoding: Encoding,
    },
    CalendarUserType {
        cu_type: CalendarUserType,
    },
    Members {
        members: Vec<String>,
    },
    Role {
        role: Role,
    },
    ParticipationStatus {
        status: ParticipationStatusUnknown,
    },
    Rsvp {
        rsvp: bool,
    },
    DelegatedTo {
        delegates: Vec<String>,
    },
    DelegatedFrom {
        delegators: Vec<String>,
    },
    RelationshipType {
        relationship: RelationshipType,
    },
    FreeBusyTimeType {
        fb_type: FreeBusyTimeType,
    },
    Related {
        related: Related,
    },
    Other {
        name: String,
        value: String,
    },
    Others {
        name: String,
        values: Vec<String>,
    },
}

impl Display for TimeTransparency {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            TimeTransparency::Opaque => write!(f, "OPAQUE"),
            TimeTransparency::Transparent => write!(f, "TRANSPARENT"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum ParticipationStatusEvent {
    #[default]
    NeedsAction,
    Accepted,
    Declined,
    Tentative,
    Delegated,
    XName(String),
    IanaToken(String),
}

impl From<ParticipationStatusEvent> for ParticipationStatusUnknown {
    fn from(value: ParticipationStatusEvent) -> ParticipationStatusUnknown {
        match value {
            ParticipationStatusEvent::NeedsAction => ParticipationStatusUnknown::NeedsAction,
            ParticipationStatusEvent::Accepted => ParticipationStatusUnknown::Accepted,
            ParticipationStatusEvent::Declined => ParticipationStatusUnknown::Declined,
            ParticipationStatusEvent::Tentative => ParticipationStatusUnknown::Tentative,
            ParticipationStatusEvent::Delegated => ParticipationStatusUnknown::Delegated,
            ParticipationStatusEvent::XName(name) => ParticipationStatusUnknown::XName(name),
            ParticipationStatusEvent::IanaToken(token) => {
                ParticipationStatusUnknown::IanaToken(token)
            }
        }
    }
}

pub enum ParticipationStatusToDo {
    NeedsAction,
    Accepted,
    Declined,
    Tentative,
    Delegated,
    Completed,
    InProcess,
    XName(String),
    IanaToken(String),
}

impl From<ParticipationStatusToDo> for ParticipationStatusUnknown {
    fn from(value: ParticipationStatusToDo) -> ParticipationStatusUnknown {
        match value {
            ParticipationStatusToDo::NeedsAction => ParticipationStatusUnknown::NeedsAction,
            ParticipationStatusToDo::Accepted => ParticipationStatusUnknown::Accepted,
            ParticipationStatusToDo::Declined => ParticipationStatusUnknown::Declined,
            ParticipationStatusToDo::Tentative => ParticipationStatusUnknown::Tentative,
            ParticipationStatusToDo::Delegated => ParticipationStatusUnknown::Delegated,
            ParticipationStatusToDo::Completed => ParticipationStatusUnknown::Completed,
            ParticipationStatusToDo::InProcess => ParticipationStatusUnknown::InProcess,
            ParticipationStatusToDo::XName(name) => ParticipationStatusUnknown::XName(name),
            ParticipationStatusToDo::IanaToken(token) => {
                ParticipationStatusUnknown::IanaToken(token)
            }
        }
    }
}

pub enum ParticipationStatusJournal {
    NeedsAction,
    Accepted,
    Declined,
    XName(String),
    IanaToken(String),
}

impl From<ParticipationStatusJournal> for ParticipationStatusUnknown {
    fn from(value: ParticipationStatusJournal) -> ParticipationStatusUnknown {
        match value {
            ParticipationStatusJournal::NeedsAction => ParticipationStatusUnknown::NeedsAction,
            ParticipationStatusJournal::Accepted => ParticipationStatusUnknown::Accepted,
            ParticipationStatusJournal::Declined => ParticipationStatusUnknown::Declined,
            ParticipationStatusJournal::XName(name) => ParticipationStatusUnknown::XName(name),
            ParticipationStatusJournal::IanaToken(token) => {
                ParticipationStatusUnknown::IanaToken(token)
            }
        }
    }
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
use std::fmt;
use std::fmt::{Display, Formatter};

macro_rules! impl_other_component_params_builder {
    ($builder:ident<$p:ident$(,$oth:ident),*>) => {
        impl<$p $(,$oth)*> crate::model::param::OtherParamsBuilder for $builder<$p $(,$oth)*>
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
            self.inner.params.push(Param::AltRep { uri: value });
            self
        }
    };
}

pub(crate) use altrep_param;

macro_rules! language_param {
    () => {
        pub fn add_language(mut self, language: $crate::common::LanguageTag) -> Self {
            self.inner.params.push(Param::Language { language });
            self
        }
    };
}

pub(crate) use language_param;

macro_rules! tz_id_param {
    () => {
        pub fn add_tz_id<V: ToString>(mut self, tz_id: V, unique: bool) -> Self {
            self.inner.params.push(Param::TimeZoneId {
                tz_id: tz_id.to_string(),
                unique,
            });
            self
        }
    };
}

pub(crate) use tz_id_param;

macro_rules! sent_by_param {
    () => {
        // TODO should be a URI
        pub fn add_sent_by(mut self, value: String) -> Self {
            self.inner.params.push(Param::SentBy { address: value });
            self
        }
    };
}

pub(crate) use sent_by_param;

macro_rules! common_name_param {
    () => {
        pub fn add_common_name<V: ToString>(mut self, value: V) -> Self {
            self.inner.params.push(Param::CommonName {
                name: value.to_string(),
            });
            self
        }
    };
}

pub(crate) use common_name_param;

macro_rules! directory_entry_reference_param {
    () => {
        // TODO should be a URI
        pub fn add_directory_entry_reference(mut self, value: String) -> Self {
            self.inner
                .params
                .push(Param::DirectoryEntryReference { uri: value });
            self
        }
    };
}

use crate::common::{
    CalendarUserType, Encoding, FreeBusyTimeType, LanguageTag, ParticipationStatusUnknown, Range,
    Related, RelationshipType, Role, TimeTransparency, Value,
};
pub(crate) use directory_entry_reference_param;

macro_rules! add_is_utc {
    () => {
        pub fn add_is_utc(mut self) -> Self {
            self.inner.is_utc = true;
            self
        }
    };
}

pub(crate) use add_is_utc;
