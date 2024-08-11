use crate::convert::{convert_string, ToModel};
use crate::model::param::{
    AlternateRepresentationParam, CalendarUserTypeParam, CommonNameParam, DelegatedFromParam,
    DelegatedToParam, DirectoryEntryReferenceParam, EncodingParam, FormatTypeParam,
    FreeBusyTimeTypeParam, LanguageParam, MembersParam, Param as ModelParam,
    ParticipationStatusParam, RangeParam, RelatedParam, RelationshipTypeParam, RoleParam,
    RsvpParam, SentByParam, TimeZoneIdParam, ValueTypeParam,
};
use crate::parser::types::ParamValue as ParserParam;

impl ToModel for ParserParam<'_> {
    type Model = ModelParam;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        Ok(match self {
            ParserParam::AltRep { uri } => ModelParam::AltRep(AlternateRepresentationParam {
                uri: convert_string(uri),
            }),
            ParserParam::CommonName { name } => ModelParam::CommonName(CommonNameParam {
                name: name.to_string(),
            }),
            ParserParam::CalendarUserType { cu_type } => {
                ModelParam::CalendarUserType(CalendarUserTypeParam {
                    cu_type: cu_type.clone(),
                })
            }
            ParserParam::DelegatedFrom { delegators } => {
                ModelParam::DelegatedFrom(DelegatedFromParam {
                    delegators: delegators.iter().map(|d| convert_string(d)).collect(),
                })
            }
            ParserParam::DelegatedTo { delegates } => ModelParam::DelegatedTo(DelegatedToParam {
                delegates: delegates.iter().map(|d| convert_string(d)).collect(),
            }),
            ParserParam::DirectoryEntryReference { uri } => {
                ModelParam::DirectoryEntryReference(DirectoryEntryReferenceParam {
                    uri: String::from_utf8_lossy(uri).to_string(),
                })
            }
            ParserParam::Encoding { encoding } => ModelParam::Encoding(EncodingParam {
                encoding: encoding.clone(),
            }),
            ParserParam::FormatType {
                type_name,
                sub_type_name,
            } => ModelParam::FormatType(FormatTypeParam {
                type_name: type_name.to_string(),
                sub_type_name: sub_type_name.to_string(),
            }),
            ParserParam::FreeBusyTimeType { fb_type } => {
                ModelParam::FreeBusyTimeType(FreeBusyTimeTypeParam {
                    fb_type: fb_type.clone(),
                })
            }
            ParserParam::Language { language } => ModelParam::Language(LanguageParam {
                language: language.clone(),
            }),
            ParserParam::Members { members } => ModelParam::Members(MembersParam {
                members: members.iter().map(|m| convert_string(m)).collect(),
            }),
            ParserParam::ParticipationStatus { status } => {
                ModelParam::ParticipationStatus(ParticipationStatusParam {
                    status: status.clone(),
                })
            }
            ParserParam::Range { range } => ModelParam::Range(RangeParam {
                range: range.clone(),
            }),
            ParserParam::Related { related } => ModelParam::Related(RelatedParam {
                related: related.clone(),
            }),
            ParserParam::RelationshipType { relationship } => {
                ModelParam::RelationshipType(RelationshipTypeParam {
                    relationship: relationship.clone(),
                })
            }
            ParserParam::Role { role } => ModelParam::Role(RoleParam { role: role.clone() }),
            ParserParam::Rsvp { rsvp } => ModelParam::Rsvp(RsvpParam { rsvp: *rsvp }),
            ParserParam::SentBy { address } => ModelParam::SentBy(SentByParam {
                address: convert_string(address),
            }),
            ParserParam::TimeZoneId { tz_id, unique } => ModelParam::TimeZoneId(TimeZoneIdParam {
                tz_id: tz_id.to_string(),
                unique: *unique,
            }),
            ParserParam::ValueType { value } => ModelParam::ValueType(ValueTypeParam {
                value: value.clone(),
            }),
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
