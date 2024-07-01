mod types;
mod values;

use crate::common::Range;
use crate::parser::language_tag::language_tag;
use crate::parser::property::uri::param_value_uri;
use crate::parser::{param_name, param_value, read_string, reg_name, x_name, Error};
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::bytes::streaming::tag;
use nom::character::streaming::char;
use nom::combinator::{cut, map_res, recognize};
use nom::error::ParseError;
use nom::multi::{many0, separated_list1};
use nom::sequence::{delimited, separated_pair, tuple};
use nom::{IResult, Parser};
pub use types::*;
pub use values::*;

pub fn params<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Vec<ParamValue<'a>>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    many0(tuple((char(';'), cut(param))).map(|(_, p)| p)).parse(input)
}

pub fn param<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    alt((known_param, iana_param, x_param))(input)
}

/// Parse an ALTREP param
///
/// RFC 5545, section 3.2.1
fn param_alternate_text_representation<'a, E>(
    input: &'a [u8],
) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, _, uri)) = tuple((
        tag_no_case("ALTREP"),
        char('='),
        cut(delimited(char('"'), recognize(param_value_uri), char('"'))),
    ))(input)?;

    Ok((input, ParamValue::AltRep { uri }))
}

/// Parse a CN param
///
/// RFC 5545, section 3.2.2
fn param_common_name<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (_, _, value)) = tuple((tag_no_case("CN"), char('='), cut(param_value)))(input)?;

    Ok((
        input,
        ParamValue::CommonName {
            name: read_string(value, "common_name")?,
        },
    ))
}

/// Parse a CUTYPE param
///
/// RFC 5545, section 3.2.3
fn param_calendar_user_type<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, _, cu_type)) = tuple((
        tag_no_case("CUTYPE"),
        char('='),
        cut(param_value_calendar_user_type),
    ))(input)?;

    Ok((input, ParamValue::CalendarUserType { cu_type }))
}

/// Parse a DELEGATED-FROM param
///
/// RFC 5545, section 3.2.4
fn param_delegated_from<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, _, delegators)) = tuple((
        tag_no_case("DELEGATED-FROM"),
        char('='),
        cut(separated_list1(
            char(','),
            delimited(char('"'), recognize(param_value_uri), char('"')),
        )),
    ))(input)?;

    Ok((input, ParamValue::DelegatedFrom { delegators }))
}

/// Parse a DELEGATED-TO param
///
/// RFC 5545, section 3.2.5
fn param_delegated_to<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, _, delegates)) = tuple((
        tag_no_case("DELEGATED-TO"),
        char('='),
        cut(separated_list1(
            char(','),
            delimited(char('"'), recognize(param_value_uri), char('"')),
        )),
    ))(input)?;

    Ok((input, ParamValue::DelegatedTo { delegates }))
}

/// Parse a DIR param
///
/// RFC 5545, section 3.2.6
fn param_dir<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, _, uri)) = tuple((
        tag_no_case("DIR"),
        char('='),
        cut(delimited(char('"'), recognize(param_value_uri), char('"'))),
    ))(input)?;

    Ok((input, ParamValue::DirectoryEntryReference { uri }))
}

/// Parse an ENCODING param
///
/// RFC 5545, section 3.2.7
fn param_encoding<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, _, encoding)) = tuple((
        tag_no_case("ENCODING"),
        char('='),
        cut(param_value_encoding),
    ))(input)?;

    Ok((input, ParamValue::Encoding { encoding }))
}

/// Parse an FMTTYPE param
///
/// RFC 5545, section 3.2.8
fn param_format_type<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, _, (type_name, sub_type_name))) = tuple((
        tag_no_case("FMTTYPE"),
        char('='),
        cut(separated_pair(
            map_res(reg_name, |t| read_string(t, "FMTTYPE type-name")),
            char('/'),
            map_res(reg_name, |t| read_string(t, "FMTTYPE subtype-name")),
        )),
    ))(input)?;

    Ok((
        input,
        ParamValue::FormatType {
            type_name,
            sub_type_name,
        },
    ))
}

/// Parse an FBTYPE param
///
/// RFC 5545, section 3.2.9
fn param_free_busy_time_type<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, _, fb_type)) = tuple((
        tag_no_case("FBTYPE"),
        char('='),
        cut(param_value_free_busy_time_type),
    ))(input)?;

    Ok((input, ParamValue::FreeBusyTimeType { fb_type }))
}

/// Parse a LANGUAGE param
///
/// RFC 5545, section 3.2.10
fn param_language<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, _, language)) =
        tuple((tag_no_case("LANGUAGE"), char('='), cut(language_tag)))(input)?;

    Ok((input, ParamValue::Language { language }))
}

/// Parse a MEMBER param
///
/// RFC 5545, section 3.2.11
fn param_member<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, _, members)) = tuple((
        tag_no_case("MEMBER"),
        char('='),
        cut(separated_list1(
            char(','),
            delimited(char('"'), recognize(param_value_uri), char('"')),
        )),
    ))(input)?;

    Ok((input, ParamValue::Members { members }))
}

/// Parse a PARTSTAT param
///
/// RFC 5545, section 3.2.12
fn param_participation_status<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, _, status)) = tuple((
        tag_no_case("PARTSTAT"),
        char('='),
        cut(param_value_particpation_status),
    ))(input)?;

    Ok((input, ParamValue::ParticipationStatus { status }))
}

/// Parse a RANGE param
///
/// RFC 5545, section 3.2.13
fn param_range<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, _, _)) =
        tuple((tag_no_case("RANGE"), char('='), cut(tag("THISANDFUTURE"))))(input)?;

    Ok((
        input,
        ParamValue::Range {
            range: Range::ThisAndFuture,
        },
    ))
}

/// Parse a RELATED param
///
/// RFC 5545, section 3.2.14
fn param_related<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, _, related)) =
        tuple((tag_no_case("RELATED"), char('='), cut(param_value_related)))(input)?;

    Ok((input, ParamValue::Related { related }))
}

/// Parse a RELTYPE param
///
/// RFC 5545, section 3.2.15
fn param_relationship_type<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, _, relationship)) = tuple((
        tag_no_case("RELTYPE"),
        char('='),
        cut(param_value_relationship_type),
    ))(input)?;

    Ok((input, ParamValue::RelationshipType { relationship }))
}

/// Parse a ROLE param
///
/// RFC 5545, section 3.2.16
fn param_role<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, _, role)) =
        tuple((tag_no_case("ROLE"), char('='), cut(param_value_role)))(input)?;

    Ok((input, ParamValue::Role { role }))
}

/// Parse an RSVP param
///
/// RFC 5545, section 3.2.17
fn param_rsvp<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, _, rsvp)) =
        tuple((tag_no_case("RSVP"), char('='), cut(param_value_rsvp)))(input)?;

    Ok((input, ParamValue::Rsvp { rsvp }))
}

/// Parse an SENT-BY param
///
/// RFC 5545, section 3.2.18
fn param_sent_by<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, _, address)) = tuple((
        tag_no_case("SENT-BY"),
        char('='),
        cut(delimited(char('"'), recognize(param_value_uri), char('"'))),
    ))(input)?;

    Ok((input, ParamValue::SentBy { address }))
}

/// Parse a TZID param
///
/// RFC 5545, section 3.2.19
fn param_time_zone_identifier<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, _, (tz_id, unique))) =
        tuple((tag_no_case("TZID"), char('='), cut(param_value_tz_id)))(input)?;

    Ok((input, ParamValue::TimeZoneId { tz_id, unique }))
}

/// Parse a VALUE param
///
/// RFC 5545, section 3.2.20
fn param_value_type<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, _, value)) =
        tuple((tag_no_case("VALUE"), char('='), cut(param_value_value_type)))(input)?;

    Ok((input, ParamValue::ValueType { value }))
}

fn known_param<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, param_value) = alt((
        param_alternate_text_representation,
        param_common_name,
        param_calendar_user_type,
        param_delegated_from,
        param_delegated_to,
        param_dir,
        param_encoding,
        param_format_type,
        param_free_busy_time_type,
        param_language,
        param_member,
        param_participation_status,
        param_range,
        param_related,
        param_relationship_type,
        param_role,
        param_rsvp,
        param_sent_by,
        param_time_zone_identifier,
        param_value_type,
    ))(input)?;

    Ok((input, param_value))
}

pub fn other_params<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Vec<ParamValue<'a>>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    many0(tuple((char(';'), other_param)).map(|(_, p)| p)).parse(input)
}

pub fn other_param<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    alt((iana_param, x_param))(input)
}

fn iana_param<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (name, _, values)) = tuple((
        param_name,
        char('='),
        separated_list1(char(','), param_value),
    ))(input)?;

    Ok((
        input,
        match values.len() {
            1 => ParamValue::Other {
                name,
                value: values[0],
            },
            _ => ParamValue::Others { name, values },
        },
    ))
}

fn x_param<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ParamValue<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (name, _, values)) =
        tuple((x_name, char('='), separated_list1(char(','), param_value)))(input)?;

    Ok((
        input,
        match values.len() {
            1 => ParamValue::Other {
                name,
                value: values[0],
            },
            _ => ParamValue::Others { name, values },
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::LanguageTag;
    use crate::common::{
        CalendarUserType, Encoding, FreeBusyTimeType, ParticipationStatusUnknown, Related,
        RelationshipType, Role, Value,
    };
    use crate::test_utils::check_rem;

    #[test]
    fn param_altrep() {
        let (rem, param) =
            known_param::<Error>(b"ALTREP=\"http://example.com/calendar\";").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::AltRep {
                uri: b"http://example.com/calendar"
            },
            param
        );
    }

    #[test]
    fn param_cn() {
        let (rem, param) = known_param::<Error>(b"CN=\"John Smith\";").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::CommonName {
                name: "John Smith".to_string()
            },
            param
        );
    }

    #[test]
    fn param_cn_not_quoted() {
        let (rem, param) = known_param::<Error>(b"CN=Danny;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::CommonName {
                name: "Danny".to_string()
            },
            param
        );
    }

    #[test]
    fn param_cu_type_individual() {
        let (rem, param) = known_param::<Error>(b"CUTYPE=INDIVIDUAL;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::CalendarUserType {
                cu_type: CalendarUserType::Individual
            },
            param
        );
    }

    #[test]
    fn param_cu_type_group() {
        let (rem, param) = known_param::<Error>(b"CUTYPE=GROUP;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::CalendarUserType {
                cu_type: CalendarUserType::Group
            },
            param
        );
    }

    #[test]
    fn param_cu_type_resource() {
        let (rem, param) = known_param::<Error>(b"CUTYPE=RESOURCE;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::CalendarUserType {
                cu_type: CalendarUserType::Resource
            },
            param
        );
    }

    #[test]
    fn param_cu_type_room() {
        let (rem, param) = known_param::<Error>(b"CUTYPE=ROOM;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::CalendarUserType {
                cu_type: CalendarUserType::Room
            },
            param
        );
    }

    #[test]
    fn param_cu_type_unknown() {
        let (rem, param) = known_param::<Error>(b"CUTYPE=UNKNOWN;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::CalendarUserType {
                cu_type: CalendarUserType::Unknown
            },
            param
        );
    }

    #[test]
    fn param_cu_type_x_name() {
        let (rem, param) = known_param::<Error>(b"CUTYPE=X-esl-special;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::CalendarUserType {
                cu_type: CalendarUserType::XName("X-esl-special".to_string())
            },
            param
        );
    }

    #[test]
    fn param_cu_type_iana_token() {
        let (rem, param) = known_param::<Error>(b"CUTYPE=other;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::CalendarUserType {
                cu_type: CalendarUserType::IanaToken("other".to_string())
            },
            param
        );
    }

    #[test]
    fn param_delegated_from() {
        let (rem, param) =
            known_param::<Error>(b"DELEGATED-FROM=\"mailto:jsmith@example.com\";").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::DelegatedFrom {
                delegators: vec![b"mailto:jsmith@example.com"],
            },
            param
        );
    }

    #[test]
    fn param_delegated_from_multi() {
        let (rem, param) = known_param::<Error>(
            b"DELEGATED-FROM=\"mailto:jsmith@example.com\",\"mailto:danny@example.com\";",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::DelegatedFrom {
                delegators: vec![b"mailto:jsmith@example.com", b"mailto:danny@example.com",],
            },
            param
        );
    }

    #[test]
    fn param_delegated_to() {
        let (rem, param) =
            known_param::<Error>(b"DELEGATED-TO=\"mailto:jsmith@example.com\";").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::DelegatedTo {
                delegates: vec![b"mailto:jsmith@example.com"],
            },
            param
        );
    }

    #[test]
    fn param_delegated_to_multi() {
        let (rem, param) = known_param::<Error>(
            b"DELEGATED-TO=\"mailto:jsmith@example.com\",\"mailto:danny@example.com\";",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::DelegatedTo {
                delegates: vec![b"mailto:jsmith@example.com", b"mailto:danny@example.com",],
            },
            param
        );
    }

    #[test]
    fn param_dir() {
        let (rem, param) = known_param::<Error>(
            b"DIR=\"ldap://example.com:6666/o=ABC%20Industries,c=US???(cn=Jim%20Dolittle)\";",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::DirectoryEntryReference {
                uri: b"ldap://example.com:6666/o=ABC%20Industries,c=US???(cn=Jim%20Dolittle)"
            },
            param
        );
    }

    #[test]
    fn param_encoding_8bit() {
        let (rem, param) = known_param::<Error>(b"ENCODING=8BIT;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::Encoding {
                encoding: Encoding::EightBit
            },
            param
        );
    }

    #[test]
    fn param_encoding_base64() {
        let (rem, param) = known_param::<Error>(b"ENCODING=BASE64;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::Encoding {
                encoding: Encoding::Base64
            },
            param
        );
    }

    #[test]
    fn param_fmt_type() {
        let (rem, param) = known_param::<Error>(b"FMTTYPE=application/msword;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::FormatType {
                type_name: "application".to_string(),
                sub_type_name: "msword".to_string(),
            },
            param
        );
    }

    #[test]
    fn param_fb_type_free() {
        let (rem, param) = known_param::<Error>(b"FBTYPE=FREE;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::FreeBusyTimeType {
                fb_type: FreeBusyTimeType::Free
            },
            param
        );
    }

    #[test]
    fn param_fb_type_busy() {
        let (rem, param) = known_param::<Error>(b"FBTYPE=BUSY;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::FreeBusyTimeType {
                fb_type: FreeBusyTimeType::Busy
            },
            param
        );
    }

    #[test]
    fn param_fb_type_busy_unavailable() {
        let (rem, param) = known_param::<Error>(b"FBTYPE=BUSY-UNAVAILABLE;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::FreeBusyTimeType {
                fb_type: FreeBusyTimeType::BusyUnavailable
            },
            param
        );
    }

    #[test]
    fn param_fb_type_busy_tentative() {
        let (rem, param) = known_param::<Error>(b"FBTYPE=BUSY-TENTATIVE;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::FreeBusyTimeType {
                fb_type: FreeBusyTimeType::BusyTentative
            },
            param
        );
    }

    #[test]
    fn param_language() {
        let (rem, param) = known_param::<Error>(b"LANGUAGE=en-US;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::Language {
                language: LanguageTag {
                    language: "en".to_string(),
                    region: Some("US".to_string()),
                    ..Default::default()
                }
            },
            param
        );
    }

    #[test]
    fn param_member() {
        let (rem, param) =
            known_param::<Error>(b"MEMBER=\"mailto:ietf-calsch@example.org\";").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::Members {
                members: vec![b"mailto:ietf-calsch@example.org"],
            },
            param
        );
    }

    #[test]
    fn param_member_multi() {
        let (rem, param) = known_param::<Error>(
            b"MEMBER=\"mailto:projectA@example.com\",\"mailto:projectB@example.com\";",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::Members {
                members: vec![
                    b"mailto:projectA@example.com",
                    b"mailto:projectB@example.com",
                ],
            },
            param
        );
    }

    #[test]
    fn param_part_stat_declined() {
        let (rem, param) = known_param::<Error>(b"PARTSTAT=DECLINED;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::ParticipationStatus {
                status: ParticipationStatusUnknown::Declined
            },
            param
        );
    }

    #[test]
    fn param_range() {
        let (rem, param) = known_param::<Error>(b"RANGE=THISANDFUTURE;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::Range {
                range: Range::ThisAndFuture
            },
            param
        );
    }

    #[test]
    fn param_related_start() {
        let (rem, param) = known_param::<Error>(b"RELATED=START;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::Related {
                related: Related::Start
            },
            param
        );
    }

    #[test]
    fn param_related_end() {
        let (rem, param) = known_param::<Error>(b"RELATED=END;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::Related {
                related: Related::End
            },
            param
        );
    }

    #[test]
    fn param_rel_type() {
        let (rem, param) = known_param::<Error>(b"RELTYPE=SIBLING;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::RelationshipType {
                relationship: RelationshipType::Sibling
            },
            param
        );
    }

    #[test]
    fn param_role() {
        let (rem, param) = known_param::<Error>(b"ROLE=CHAIR;").unwrap();
        check_rem(rem, 1);
        assert_eq!(ParamValue::Role { role: Role::Chair }, param);
    }

    #[test]
    fn param_rsvp_true() {
        let (rem, param) = known_param::<Error>(b"RSVP=TRUE;").unwrap();
        check_rem(rem, 1);
        assert_eq!(ParamValue::Rsvp { rsvp: true }, param);
    }

    #[test]
    fn param_rsvp_false() {
        let (rem, param) = known_param::<Error>(b"RSVP=FALSE;").unwrap();
        check_rem(rem, 1);
        assert_eq!(ParamValue::Rsvp { rsvp: false }, param);
    }

    #[test]
    fn param_sent_by() {
        let (rem, param) = known_param::<Error>(b"SENT-BY=\"mailto:sray@example.com\";").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::SentBy {
                address: b"mailto:sray@example.com"
            },
            param
        );
    }

    #[test]
    fn param_tz_id() {
        let (rem, param) = known_param::<Error>(b"TZID=America/New_York;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::TimeZoneId {
                tz_id: "America/New_York".to_string(),
                unique: false,
            },
            param
        );
    }

    #[test]
    fn param_tz_id_unique() {
        let (rem, param) = known_param::<Error>(b"TZID=/America/New_York;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::TimeZoneId {
                tz_id: "America/New_York".to_string(),
                unique: true,
            },
            param
        );
    }

    #[test]
    fn param_value_binary() {
        let (rem, param) = known_param::<Error>(b"VALUE=BINARY;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            ParamValue::ValueType {
                value: Value::Binary
            },
            param
        );
    }
}
