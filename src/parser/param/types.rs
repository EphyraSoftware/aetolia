use crate::parser::language_tag::LanguageTag;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Param<'a> {
    pub name: String,
    pub value: ParamValue<'a>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ParamValue<'a> {
    AltRep {
        uri: String,
    },
    CommonName {
        name: String,
    },
    CalendarUserType {
        cu_type: CalendarUserType,
    },
    DelegatedFrom {
        delegators: Vec<String>,
    },
    DelegatedTo {
        delegates: Vec<String>,
    },
    Dir {
        uri: String,
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
        members: Vec<String>,
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
        address: String,
    },
    TimeZoneId {
        tz_id: String,
        unique: bool,
    },
    Value {
        value: Value,
    },
    Other {
        value: &'a [u8],
    },
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum CalendarUserType {
    #[default]
    Individual,
    Group,
    Resource,
    Room,
    Unknown,
    XName(String),
    IanaToken(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum Encoding {
    #[default]
    EightBit,
    Base64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FreeBusyTimeType {
    Free,
    Busy,
    BusyUnavailable,
    BusyTentative,
    XName(String),
    IanaToken(String),
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
