use crate::model::param::{
    AlternateRepresentationParam, CalendarUserTypeParam, CommonNameParam, DelegatedFromParam,
    DelegatedToParam, DirectoryEntryReferenceParam, EncodingParam, FormatTypeParam,
    FreeBusyTimeTypeParam, LanguageParam, MembersParam, ParticipationStatusParam, RangeParam,
    RelatedParam, RelationshipTypeParam, RoleParam, RsvpParam, SentByParam, TimeZoneIdParam,
    ValueTypeParam,
};
use crate::serialize::WriteModel;
use std::io::Write;

impl WriteModel for crate::model::param::Param {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        use crate::model::param::Param;

        match self {
            Param::AltRep(AlternateRepresentationParam { uri }) => {
                write!(writer, "ALTREP=\"{}\"", uri)?;
            }
            Param::CommonName(CommonNameParam { name }) => {
                write!(writer, "CN={}", name)?;
            }
            Param::ValueType(ValueTypeParam { value }) => {
                write!(writer, "VALUE=")?;
                value.write_model(writer)?;
            }
            Param::TimeZoneId(TimeZoneIdParam { tz_id, unique }) => {
                writer.write_all(b"TZID=")?;
                if *unique {
                    writer.write_all(b"/")?;
                }
                writer.write_all(tz_id.as_bytes())?;
            }
            Param::Language(LanguageParam { language }) => {
                writer.write_all(b"LANGUAGE=")?;
                language.write_model(writer)?;
            }
            Param::DirectoryEntryReference(DirectoryEntryReferenceParam { uri }) => {
                write!(writer, "DIR=\"{}\"", uri)?;
            }
            Param::SentBy(SentByParam { address }) => {
                write!(writer, "SENT-BY=\"{}\"", address)?;
            }
            Param::Range(RangeParam { range }) => {
                write!(writer, "RANGE=")?;
                range.write_model(writer)?;
            }
            Param::FormatType(FormatTypeParam {
                type_name,
                sub_type_name,
            }) => {
                write!(writer, "FMTTYPE={}/{}", type_name, sub_type_name)?;
            }
            Param::Encoding(EncodingParam { encoding }) => {
                writer.write_all(b"ENCODING=")?;
                encoding.write_model(writer)?;
            }
            Param::CalendarUserType(CalendarUserTypeParam { cu_type }) => {
                writer.write_all(b"CUTYPE=")?;
                cu_type.write_model(writer)?;
            }
            Param::Members(MembersParam { members }) => {
                writer.write_all(b"MEMBER=")?;
                if let Some(member) = members.first() {
                    write!(writer, "\"{}\"", member)?;
                }
                for member in members.iter().skip(1) {
                    write!(writer, ",\"{}\"", member)?;
                }
            }
            Param::Role(RoleParam { role }) => {
                writer.write_all(b"ROLE=")?;
                role.write_model(writer)?;
            }
            Param::ParticipationStatus(ParticipationStatusParam { status }) => {
                writer.write_all(b"PARTSTAT=")?;
                status.write_model(writer)?;
            }
            Param::Rsvp(RsvpParam { rsvp }) => {
                writer.write_all(b"RSVP=")?;
                rsvp.write_model(writer)?;
            }
            Param::DelegatedTo(DelegatedToParam { delegates }) => {
                writer.write_all(b"DELEGATED-TO=")?;
                if let Some(delegate) = delegates.first() {
                    write!(writer, "\"{}\"", delegate)?;
                }
                for delegate in delegates.iter().skip(1) {
                    write!(writer, ",\"{}\"", delegate)?;
                }
            }
            Param::DelegatedFrom(DelegatedFromParam { delegators }) => {
                writer.write_all(b"DELEGATED-FROM=")?;
                if let Some(delegate) = delegators.first() {
                    write!(writer, "\"{}\"", delegate)?;
                }
                for delegate in delegators.iter().skip(1) {
                    write!(writer, ",\"{}\"", delegate)?;
                }
            }
            Param::RelationshipType(RelationshipTypeParam { relationship }) => {
                writer.write_all(b"RELTYPE=")?;
                relationship.write_model(writer)?;
            }
            Param::FreeBusyTimeType(FreeBusyTimeTypeParam { fb_type }) => {
                writer.write_all(b"FBTYPE=")?;
                fb_type.write_model(writer)?;
            }
            Param::Related(RelatedParam { related }) => {
                writer.write_all(b"RELATED=")?;
                related.write_model(writer)?;
            }
            Param::Other { name, value } => {
                write!(writer, "{}={}", name, value)?;
            }
            Param::Others { name, values } => {
                write!(writer, "{}=", name)?;
                if let Some(value) = values.first() {
                    write!(writer, "\"{}\"", value)?;
                }
                for value in values.iter().skip(1) {
                    write!(writer, ",\"{}\"", value)?;
                }
            }
        }

        Ok(())
    }
}
