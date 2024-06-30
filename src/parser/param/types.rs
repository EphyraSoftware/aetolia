use crate::common::{CalendarUserType, Encoding, FreeBusyTimeType};
use crate::parser::language_tag::LanguageTag;

#[derive(Debug, Eq, PartialEq)]
pub struct Param<'a> {
    pub value: ParamValue<'a>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ParamValue<'a> {
    AltRep {
        uri: &'a [u8],
    },
    CommonName {
        name: String,
    },
    CalendarUserType {
        cu_type: CalendarUserType,
    },
    DelegatedFrom {
        delegators: Vec<&'a [u8]>,
    },
    DelegatedTo {
        delegates: Vec<&'a [u8]>,
    },
    DirectoryEntryReference {
        uri: &'a [u8],
    },
    Encoding {
        encoding: Encoding,
    },
    /// See https://www.rfc-editor.org/rfc/rfc4288 section 4.2
    FormatType {
        type_name: String,
        sub_type_name: String,
    },
    FreeBusyTimeType {
        fb_type: FreeBusyTimeType,
    },
    Language {
        language: LanguageTag,
    },
    Members {
        members: Vec<&'a [u8]>,
    },
    ParticipationStatus {
        // TODO convert to ParticipationStatusKind when context is available
        status: ParticipationStatusUnknown,
    },
    Range {
        range: Range,
    },
    Related {
        related: Related,
    },
    RelationshipType {
        relationship: RelationshipType,
    },
    Role {
        role: Role,
    },
    Rsvp {
        rsvp: bool,
    },
    SentBy {
        address: &'a [u8],
    },
    TimeZoneId {
        tz_id: String,
        unique: bool,
    },
    ValueType {
        value: Value,
    },
    Other {
        name: &'a [u8],
        value: &'a [u8],
    },
    Others {
        name: &'a [u8],
        values: Vec<&'a [u8]>,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ParticipationStatusKind {
    Event { status: ParticipationStatusEvent },
    Todo { status: ParticipationStatusTodo },
    Journal { status: ParticipationStatusJournal },
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

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum ParticipationStatusTodo {
    #[default]
    NeedsAction,
    Accepted,
    Declined,
    Tentative,
    Delegated,
    /// To-do completed, the COMPLETED property has DATE-TIME completed.
    Completed,
    InProcess,
    XName(String),
    IanaToken(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum ParticipationStatusJournal {
    #[default]
    NeedsAction,
    Accepted,
    Declined,
    XName(String),
    IanaToken(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum ParticipationStatusUnknown {
    #[default]
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Range {
    ThisAndFuture,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum Related {
    #[default]
    Start,
    End,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum RelationshipType {
    #[default]
    Parent,
    Child,
    Sibling,
    XName(String),
    IanaToken(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum Role {
    Chair,
    #[default]
    RequiredParticipant,
    OptionalParticipant,
    NonParticipant,
    XName(String),
    IanaToken(String),
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
