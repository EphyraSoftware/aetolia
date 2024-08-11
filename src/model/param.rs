#[derive(Debug, Clone, PartialEq)]
pub enum Param {
    AltRep(AlternateRepresentationParam),
    CommonName(CommonNameParam),
    ValueType(ValueTypeParam),
    TimeZoneId(TimeZoneIdParam),
    Language(LanguageParam),
    DirectoryEntryReference(DirectoryEntryReferenceParam),
    SentBy(SentByParam),
    Range(RangeParam),
    FormatType(FormatTypeParam),
    Encoding(EncodingParam),
    CalendarUserType(CalendarUserTypeParam),
    Members(MembersParam),
    Role(RoleParam),
    ParticipationStatus(ParticipationStatusParam),
    Rsvp(RsvpParam),
    DelegatedTo(DelegatedToParam),
    DelegatedFrom(DelegatedFromParam),
    RelationshipType(RelationshipTypeParam),
    FreeBusyTimeType(FreeBusyTimeTypeParam),
    Related(RelatedParam),
    Other { name: String, value: String },
    Others { name: String, values: Vec<String> },
}

pub trait ParamInner<T> {
    fn param_inner(&self) -> Option<&T>;
}

macro_rules! impl_param_inner {
    ($for_type:ty, $variant:ident) => {
        impl $crate::model::ParamInner<$for_type> for Param {
            fn param_inner(&self) -> Option<&$for_type> {
                match self {
                    $crate::model::Param::$variant(p) => Some(p),
                    _ => None,
                }
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlternateRepresentationParam {
    pub uri: String,
}

impl_param_inner!(AlternateRepresentationParam, AltRep);

#[derive(Debug, Clone, PartialEq)]
pub struct CommonNameParam {
    pub name: String,
}

impl_param_inner!(CommonNameParam, CommonName);

#[derive(Debug, Clone, PartialEq)]
pub struct ValueTypeParam {
    pub value: Value,
}

impl_param_inner!(ValueTypeParam, ValueType);

#[derive(Debug, Clone, PartialEq)]
pub struct TimeZoneIdParam {
    pub tz_id: String,
    pub unique: bool,
}

impl_param_inner!(TimeZoneIdParam, TimeZoneId);

#[derive(Debug, Clone, PartialEq)]
pub struct LanguageParam {
    pub language: LanguageTag,
}

impl_param_inner!(LanguageParam, Language);

#[derive(Debug, Clone, PartialEq)]
pub struct DirectoryEntryReferenceParam {
    pub uri: String,
}

impl_param_inner!(DirectoryEntryReferenceParam, DirectoryEntryReference);

#[derive(Debug, Clone, PartialEq)]
pub struct SentByParam {
    pub address: String,
}

impl_param_inner!(SentByParam, SentBy);

#[derive(Debug, Clone, PartialEq)]
pub struct RangeParam {
    pub range: Range,
}

impl_param_inner!(RangeParam, Range);

#[derive(Debug, Clone, PartialEq)]
pub struct FormatTypeParam {
    pub type_name: String,
    pub sub_type_name: String,
}

impl_param_inner!(FormatTypeParam, FormatType);

#[derive(Debug, Clone, PartialEq)]
pub struct EncodingParam {
    pub encoding: Encoding,
}

impl_param_inner!(EncodingParam, Encoding);

#[derive(Debug, Clone, PartialEq)]
pub struct CalendarUserTypeParam {
    pub cu_type: CalendarUserType,
}

impl_param_inner!(CalendarUserTypeParam, CalendarUserType);

#[derive(Debug, Clone, PartialEq)]
pub struct MembersParam {
    pub members: Vec<String>,
}

impl_param_inner!(MembersParam, Members);

#[derive(Debug, Clone, PartialEq)]
pub struct RoleParam {
    pub role: Role,
}

impl_param_inner!(RoleParam, Role);

#[derive(Debug, Clone, PartialEq)]
pub struct ParticipationStatusParam {
    pub status: ParticipationStatusUnknown,
}

impl_param_inner!(ParticipationStatusParam, ParticipationStatus);

#[derive(Debug, Clone, PartialEq)]
pub struct RsvpParam {
    pub rsvp: bool,
}

impl_param_inner!(RsvpParam, Rsvp);

#[derive(Debug, Clone, PartialEq)]
pub struct DelegatedToParam {
    pub delegates: Vec<String>,
}

impl_param_inner!(DelegatedToParam, DelegatedTo);

#[derive(Debug, Clone, PartialEq)]
pub struct DelegatedFromParam {
    pub delegators: Vec<String>,
}

impl_param_inner!(DelegatedFromParam, DelegatedFrom);

#[derive(Debug, Clone, PartialEq)]
pub struct RelationshipTypeParam {
    pub relationship: RelationshipType,
}

impl_param_inner!(RelationshipTypeParam, RelationshipType);

#[derive(Debug, Clone, PartialEq)]
pub struct FreeBusyTimeTypeParam {
    pub fb_type: FreeBusyTimeType,
}

impl_param_inner!(FreeBusyTimeTypeParam, FreeBusyTimeType);

#[derive(Debug, Clone, PartialEq)]
pub struct RelatedParam {
    pub related: TriggerRelationship,
}

impl_param_inner!(RelatedParam, Related);

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
        pub fn add_alternate_representation(mut self, value: &str) -> Self {
            self.inner
                .params
                .push(Param::AltRep($crate::model::AlternateRepresentationParam {
                    uri: value.to_string(),
                }));
            self
        }
    };
}

pub(crate) use altrep_param;

macro_rules! language_param {
    () => {
        pub fn add_language(mut self, language: $crate::common::LanguageTag) -> Self {
            self.inner
                .params
                .push(Param::Language($crate::model::LanguageParam { language }));
            self
        }
    };
}

pub(crate) use language_param;

macro_rules! tz_id_param {
    () => {
        pub fn add_tz_id<V: ToString>(mut self, tz_id: V, unique: bool) -> Self {
            self.inner
                .params
                .push(Param::TimeZoneId($crate::model::TimeZoneIdParam {
                    tz_id: tz_id.to_string(),
                    unique,
                }));
            self
        }
    };
}

pub(crate) use tz_id_param;

macro_rules! sent_by_param {
    () => {
        // TODO should be a URI
        pub fn add_sent_by(mut self, value: &str) -> Self {
            self.inner
                .params
                .push(Param::SentBy($crate::model::SentByParam {
                    address: value.to_string(),
                }));
            self
        }
    };
}

pub(crate) use sent_by_param;

macro_rules! common_name_param {
    () => {
        pub fn add_common_name<V: ToString>(mut self, value: V) -> Self {
            self.inner
                .params
                .push(Param::CommonName($crate::model::CommonNameParam {
                    name: value.to_string(),
                }));
            self
        }
    };
}

pub(crate) use common_name_param;

macro_rules! directory_entry_reference_param {
    () => {
        // TODO should be a URI
        pub fn add_directory_entry_reference(mut self, value: &str) -> Self {
            self.inner.params.push(Param::DirectoryEntryReference(
                $crate::model::DirectoryEntryReferenceParam {
                    uri: value.to_string(),
                },
            ));
            self
        }
    };
}

use crate::common::{
    CalendarUserType, Encoding, FreeBusyTimeType, LanguageTag, ParticipationStatusUnknown, Range,
    RelationshipType, Role, TimeTransparency, TriggerRelationship, Value,
};
pub(crate) use directory_entry_reference_param;

macro_rules! add_is_utc {
    () => {
        pub fn set_is_utc(mut self) -> Self {
            self.inner.value.set_utc(true);
            self
        }
    };
}

pub(crate) use add_is_utc;
