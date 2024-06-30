use crate::convert::{convert_string, ToModel};

use crate::parser::ParamValue as ParserParam;
use crate::model::Param as ModelParam;

impl ToModel for ParserParam<'_> {
    type Model = ModelParam;

    fn to_model(&self) -> Self::Model {
        match self {
            ParserParam::AltRep { uri } => {
                ModelParam::AltRep { uri: convert_string(uri) }
            }
            ParserParam::CommonName { name } => {
                ModelParam::CommonName { name: name.to_string() }
            }
            ParserParam::CalendarUserType { cu_type } => {
                ModelParam::CalendarUserType { cu_type: cu_type.clone() }
            }
            ParserParam::DelegatedFrom { delegators } => {
                ModelParam::DelegatedFrom { delegators: delegators.iter().map(|d| convert_string(d)).collect() }
            }
            ParserParam::DelegatedTo { delegates } => {
                ModelParam::DelegatedTo { delegates: delegates.iter().map(|d| convert_string(d)).collect() }
            }
            ParserParam::DirectoryEntryReference { uri } => {
                ModelParam::DirectoryEntryReference { uri: String::from_utf8_lossy(uri).to_string() }
            }
            ParserParam::Encoding { encoding } => {
                ModelParam::Encoding { encoding: encoding.clone() }
            }
            ParserParam::FormatType { type_name, sub_type_name } => {
                ModelParam::FormatType { type_name: type_name.to_string(), sub_type_name: sub_type_name.to_string() }
            }
            ParserParam::FreeBusyTimeType { fb_type } => {
                ModelParam::FreeBusyTimeType { fb_type: fb_type.clone() }
            }
            ParserParam::Language { language } => {
                ModelParam::Language { language: language.clone() }
            }
            ParserParam::Members { members } => {
                ModelParam::Members { members: members.iter().map(|m| convert_string(m)).collect() }
            }
            ParserParam::Range { range } => {
                ModelParam::Range { range: range.clone() }
            }
            _ => {
                unimplemented!()
            }
        }
    }
}
