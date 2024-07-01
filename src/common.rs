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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageTag {
    pub language: String,
    pub ext_lang: Option<String>,
    pub script: Option<String>,
    pub region: Option<String>,
    pub variants: Vec<String>,
    pub extensions: Vec<String>,
    pub private_use: Option<String>,
}

impl Default for LanguageTag {
    fn default() -> Self {
        Self {
            language: String::new(),
            ext_lang: None,
            script: None,
            region: None,
            variants: Vec::with_capacity(0),
            extensions: Vec::with_capacity(0),
            private_use: None,
        }
    }
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
