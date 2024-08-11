use crate::common::{
    CalendarUserType, Encoding, FreeBusyTimeType, LanguageTag, ParticipationStatusUnknown, Range,
    Related, RelationshipType, Role, Value,
};

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
    /// See <https://www.rfc-editor.org/rfc/rfc4288> section 4.2
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
