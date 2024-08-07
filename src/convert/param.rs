use crate::convert::{convert_string, ToModel};

use crate::model::Param as ModelParam;
use crate::parser::ParamValue as ParserParam;

impl ToModel for ParserParam<'_> {
    type Model = ModelParam;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(match self {
            ParserParam::AltRep { uri } => ModelParam::AltRep {
                uri: convert_string(uri),
            },
            ParserParam::CommonName { name } => ModelParam::CommonName {
                name: name.to_string(),
            },
            ParserParam::CalendarUserType { cu_type } => ModelParam::CalendarUserType {
                cu_type: cu_type.clone(),
            },
            ParserParam::DelegatedFrom { delegators } => ModelParam::DelegatedFrom {
                delegators: delegators.iter().map(|d| convert_string(d)).collect(),
            },
            ParserParam::DelegatedTo { delegates } => ModelParam::DelegatedTo {
                delegates: delegates.iter().map(|d| convert_string(d)).collect(),
            },
            ParserParam::DirectoryEntryReference { uri } => ModelParam::DirectoryEntryReference {
                uri: String::from_utf8_lossy(uri).to_string(),
            },
            ParserParam::Encoding { encoding } => ModelParam::Encoding {
                encoding: encoding.clone(),
            },
            ParserParam::FormatType {
                type_name,
                sub_type_name,
            } => ModelParam::FormatType {
                type_name: type_name.to_string(),
                sub_type_name: sub_type_name.to_string(),
            },
            ParserParam::FreeBusyTimeType { fb_type } => ModelParam::FreeBusyTimeType {
                fb_type: fb_type.clone(),
            },
            ParserParam::Language { language } => ModelParam::Language {
                language: language.clone(),
            },
            ParserParam::Members { members } => ModelParam::Members {
                members: members.iter().map(|m| convert_string(m)).collect(),
            },
            ParserParam::ParticipationStatus { status } => ModelParam::ParticipationStatus {
                status: status.clone(),
            },
            ParserParam::Range { range } => ModelParam::Range {
                range: range.clone(),
            },
            ParserParam::Related { related } => ModelParam::Related {
                related: related.clone(),
            },
            ParserParam::RelationshipType { relationship } => ModelParam::RelationshipType {
                relationship: relationship.clone(),
            },
            ParserParam::Role { role } => ModelParam::Role { role: role.clone() },
            ParserParam::Rsvp { rsvp } => ModelParam::Rsvp { rsvp: *rsvp },
            ParserParam::SentBy { address } => ModelParam::SentBy {
                address: convert_string(address),
            },
            ParserParam::TimeZoneId { tz_id, unique } => ModelParam::TimeZoneId {
                tz_id: tz_id.to_string(),
                unique: *unique,
            },
            ParserParam::ValueType { value } => ModelParam::ValueType {
                value: value.clone(),
            },
            ParserParam::Other { name, value } => ModelParam::Other {
                name: convert_string(name),
                value: convert_string(value),
            },
            ParserParam::Others { name, values } => ModelParam::Others {
                name: convert_string(name),
                values: values.iter().map(|v| convert_string(v)).collect(),
            },
        })
    }
}
