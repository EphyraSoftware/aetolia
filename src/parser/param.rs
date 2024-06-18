mod types;
mod values;

use crate::parser::{
    language_tag, param_name, param_value, quoted_string, read_string, reg_name, x_name, Error,
    InnerError,
};
use nom::branch::alt;
use nom::bytes::streaming::tag;
use nom::character::streaming::char;
use nom::combinator::{cut, map_res};
use nom::error::ParseError;
use nom::multi::{many0, separated_list1};
use nom::sequence::{separated_pair, tuple};
use nom::{IResult, Parser};
pub use types::*;
pub use values::*;

pub fn params<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Vec<Param<'a>>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    many0(tuple((char(';'), cut(param))).map(|(_, p)| p)).parse(input)
}

pub fn param<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Param<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    alt((known_param, iana_param, x_param))(input)
}

fn known_param<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Param<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (name, _)) = tuple((param_name, char('=')))(input)?;

    let name_s = read_string(name, "param_name")?;
    let (input, param_value) = match name_s.as_str() {
        "ALTREP" => {
            // Requires a quoted string rather than a param-value
            let (input, uri) = quoted_string(input)?;

            (
                input,
                ParamValue::AltRep {
                    uri: read_string(uri, "uri")?,
                },
            )
        }
        "CN" => {
            let (input, value) = param_value(input)?;

            (
                input,
                ParamValue::CommonName {
                    name: read_string(value, "common_name")?,
                },
            )
        }
        "CUTYPE" => {
            let (input, cu_type) = param_calendar_user_type(input)?;

            (input, ParamValue::CalendarUserType { cu_type })
        }
        "DELEGATED-FROM" => {
            // Technically should be 'cal-address' but that's not defined at this point in the spec. Different to quoted string?
            let (input, delegators) = separated_list1(
                char(','),
                map_res(quoted_string, |d| {
                    read_string(d, "DELEGATED-FROM cal-address")
                }),
            )(input)?;

            (input, ParamValue::DelegatedFrom { delegators })
        }
        "DELEGATED-TO" => {
            // Technically should be 'cal-address' but that's not defined at this point in the spec. Different to quoted string?
            let (input, delegates) = separated_list1(
                char(','),
                map_res(quoted_string, |d| {
                    read_string(d, "DELEGATED-TO cal-address")
                }),
            )(input)?;

            (input, ParamValue::DelegatedTo { delegates })
        }
        "DIR" => {
            let (input, uri) = quoted_string(input)?;

            (
                input,
                ParamValue::Dir {
                    uri: read_string(uri, "dir")?,
                },
            )
        }
        "ENCODING" => {
            let (input, encoding) = param_encoding(input)?;

            (input, ParamValue::Encoding { encoding })
        }
        "FMTTYPE" => {
            let (input, (type_name, sub_type_name)) = separated_pair(
                map_res(reg_name, |t| read_string(t, "FMTTYPE type-name")),
                char('/'),
                map_res(reg_name, |t| read_string(t, "FMTTYPE subtype-name")),
            )(input)?;

            (
                input,
                ParamValue::FormatType {
                    type_name,
                    sub_type_name,
                },
            )
        }
        "FBTYPE" => {
            let (input, fb_type) = param_free_busy_time_type(input)?;

            (input, ParamValue::FreeBusyTimeType { fb_type })
        }
        "LANGUAGE" => {
            let (input, language) = language_tag::language_tag(input)?;

            (input, ParamValue::Language { language })
        }
        "MEMBER" => {
            let (input, members) = separated_list1(
                char(','),
                map_res(quoted_string, |m| read_string(m, "MEMBER cal-address")),
            )(input)?;

            (input, ParamValue::Members { members })
        }
        "PARTSTAT" => {
            let (input, status) = param_part_stat(input)?;

            (input, ParamValue::ParticipationStatus { status })
        }
        "RANGE" => {
            let (input, _) = tag("THISANDFUTURE")(input)?;

            (
                input,
                ParamValue::Range {
                    range: Range::ThisAndFuture,
                },
            )
        }
        // TRIGREL
        "RELATED" => {
            let (input, related) = param_related(input)?;

            (input, ParamValue::Related { related })
        }
        "RELTYPE" => {
            let (input, relationship) = param_rel_type(input)?;

            (input, ParamValue::RelationshipType { relationship })
        }
        "ROLE" => {
            let (input, role) = param_role(input)?;

            (input, ParamValue::Role { role })
        }
        "RSVP" => {
            let (input, rsvp) = param_rsvp(input)?;

            (input, ParamValue::Rsvp { rsvp })
        }
        "SENT-BY" => {
            let (input, address) = quoted_string(input)?;

            (
                input,
                ParamValue::SentBy {
                    address: read_string(address, "SENT-BY address")?,
                },
            )
        }
        "TZID" => {
            let (input, (tz_id, unique)) = param_tz_id(input)?;

            (input, ParamValue::TimeZoneId { tz_id, unique })
        }
        "VALUE" => {
            let (input, value) = param_value_type(input)?;

            (input, ParamValue::Value { value })
        }
        _ => {
            return Err(nom::Err::Error(
                Error::new(input, InnerError::UnknownParamName(String::from_utf8_lossy(name).to_string())).into(),
            ));
        }
    };

    Ok((
        input,
        Param {
            name: name_s,
            value: param_value,
        },
    ))
}

pub fn other_params<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Vec<Param<'a>>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    many0(tuple((char(';'), other_param)).map(|(_, p)| p)).parse(input)
}

pub fn other_param<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Param<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    alt((iana_param, x_param))(input)
}

fn iana_param<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Param<'a>, E>
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
        Param {
            name: read_string(name, "iana-param name")?,
            value: ParamValue::Others { values },
        },
    ))
}

fn x_param<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Param<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (name, _, values)) =
        tuple((x_name, char('='), separated_list1(char(','), param_value)))(input)?;

    Ok((
        input,
        Param {
            name: read_string(name, "x-param name")?,
            value: ParamValue::Others { values },
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::language_tag::LanguageTag;
    use crate::test_utils::check_rem;

    #[test]
    fn param_altrep() {
        let (rem, param) =
            known_param::<Error>(b"ALTREP=\"http://example.com/calendar\";").unwrap();
        check_rem(rem, 1);
        assert_eq!("ALTREP", param.name);
        assert_eq!(
            ParamValue::AltRep {
                uri: "http://example.com/calendar".to_string()
            },
            param.value
        );
    }

    #[test]
    fn param_cn() {
        let (rem, param) = known_param::<Error>(b"CN=\"John Smith\";").unwrap();
        check_rem(rem, 1);
        assert_eq!("CN", param.name);
        assert_eq!(
            ParamValue::CommonName {
                name: "John Smith".to_string()
            },
            param.value
        );
    }

    #[test]
    fn param_cn_not_quoted() {
        let (rem, param) = known_param::<Error>(b"CN=Danny;").unwrap();
        check_rem(rem, 1);
        assert_eq!("CN", param.name);
        assert_eq!(
            ParamValue::CommonName {
                name: "Danny".to_string()
            },
            param.value
        );
    }

    #[test]
    fn param_cu_type_individual() {
        let (rem, param) = known_param::<Error>(b"CUTYPE=INDIVIDUAL;").unwrap();
        check_rem(rem, 1);
        assert_eq!("CUTYPE", param.name);
        assert_eq!(
            ParamValue::CalendarUserType {
                cu_type: CalendarUserType::Individual
            },
            param.value
        );
    }

    #[test]
    fn param_cu_type_group() {
        let (rem, param) = known_param::<Error>(b"CUTYPE=GROUP;").unwrap();
        check_rem(rem, 1);
        assert_eq!("CUTYPE", param.name);
        assert_eq!(
            ParamValue::CalendarUserType {
                cu_type: CalendarUserType::Group
            },
            param.value
        );
    }

    #[test]
    fn param_cu_type_resource() {
        let (rem, param) = known_param::<Error>(b"CUTYPE=RESOURCE;").unwrap();
        check_rem(rem, 1);
        assert_eq!("CUTYPE", param.name);
        assert_eq!(
            ParamValue::CalendarUserType {
                cu_type: CalendarUserType::Resource
            },
            param.value
        );
    }

    #[test]
    fn param_cu_type_room() {
        let (rem, param) = known_param::<Error>(b"CUTYPE=ROOM;").unwrap();
        check_rem(rem, 1);
        assert_eq!("CUTYPE", param.name);
        assert_eq!(
            ParamValue::CalendarUserType {
                cu_type: CalendarUserType::Room
            },
            param.value
        );
    }

    #[test]
    fn param_cu_type_unknown() {
        let (rem, param) = known_param::<Error>(b"CUTYPE=UNKNOWN;").unwrap();
        check_rem(rem, 1);
        assert_eq!("CUTYPE", param.name);
        assert_eq!(
            ParamValue::CalendarUserType {
                cu_type: CalendarUserType::Unknown
            },
            param.value
        );
    }

    #[test]
    fn param_cu_type_x_name() {
        let (rem, param) = known_param::<Error>(b"CUTYPE=X-esl-special;").unwrap();
        check_rem(rem, 1);
        assert_eq!("CUTYPE", param.name);
        assert_eq!(
            ParamValue::CalendarUserType {
                cu_type: CalendarUserType::XName("X-esl-special".to_string())
            },
            param.value
        );
    }

    #[test]
    fn param_cu_type_iana_token() {
        let (rem, param) = known_param::<Error>(b"CUTYPE=other;").unwrap();
        check_rem(rem, 1);
        assert_eq!("CUTYPE", param.name);
        assert_eq!(
            ParamValue::CalendarUserType {
                cu_type: CalendarUserType::IanaToken("other".to_string())
            },
            param.value
        );
    }

    #[test]
    fn param_delegated_from() {
        let (rem, param) =
            known_param::<Error>(b"DELEGATED-FROM=\"mailto:jsmith@example.com\";").unwrap();
        check_rem(rem, 1);
        assert_eq!("DELEGATED-FROM", param.name);
        assert_eq!(
            ParamValue::DelegatedFrom {
                delegators: vec!["mailto:jsmith@example.com".to_string()],
            },
            param.value
        );
    }

    #[test]
    fn param_delegated_from_multi() {
        let (rem, param) = known_param::<Error>(
            b"DELEGATED-FROM=\"mailto:jsmith@example.com\",\"mailto:danny@example.com\";",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!("DELEGATED-FROM", param.name);
        assert_eq!(
            ParamValue::DelegatedFrom {
                delegators: vec![
                    "mailto:jsmith@example.com".to_string(),
                    "mailto:danny@example.com".to_string()
                ],
            },
            param.value
        );
    }

    #[test]
    fn param_delegated_to() {
        let (rem, param) =
            known_param::<Error>(b"DELEGATED-TO=\"mailto:jsmith@example.com\";").unwrap();
        check_rem(rem, 1);
        assert_eq!("DELEGATED-TO", param.name);
        assert_eq!(
            ParamValue::DelegatedTo {
                delegates: vec!["mailto:jsmith@example.com".to_string()],
            },
            param.value
        );
    }

    #[test]
    fn param_delegated_to_multi() {
        let (rem, param) = known_param::<Error>(
            b"DELEGATED-TO=\"mailto:jsmith@example.com\",\"mailto:danny@example.com\";",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!("DELEGATED-TO", param.name);
        assert_eq!(
            ParamValue::DelegatedTo {
                delegates: vec![
                    "mailto:jsmith@example.com".to_string(),
                    "mailto:danny@example.com".to_string()
                ],
            },
            param.value
        );
    }

    #[test]
    fn param_dir() {
        let (rem, param) = known_param::<Error>(
            b"DIR=\"ldap://example.com:6666/o=ABC%20Industries,c=US???(cn=Jim%20Dolittle)\";",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!("DIR", param.name);
        assert_eq!(
            ParamValue::Dir {
                uri: "ldap://example.com:6666/o=ABC%20Industries,c=US???(cn=Jim%20Dolittle)"
                    .to_string()
            },
            param.value
        );
    }

    #[test]
    fn param_encoding_8bit() {
        let (rem, param) = known_param::<Error>(b"ENCODING=8BIT;").unwrap();
        check_rem(rem, 1);
        assert_eq!("ENCODING", param.name);
        assert_eq!(
            ParamValue::Encoding {
                encoding: Encoding::EightBit
            },
            param.value
        );
    }

    #[test]
    fn param_encoding_base64() {
        let (rem, param) = known_param::<Error>(b"ENCODING=BASE64;").unwrap();
        check_rem(rem, 1);
        assert_eq!("ENCODING", param.name);
        assert_eq!(
            ParamValue::Encoding {
                encoding: Encoding::Base64
            },
            param.value
        );
    }

    #[test]
    fn param_fmt_type() {
        let (rem, param) = known_param::<Error>(b"FMTTYPE=application/msword;").unwrap();
        check_rem(rem, 1);
        assert_eq!("FMTTYPE", param.name);
        assert_eq!(
            ParamValue::FormatType {
                type_name: "application".to_string(),
                sub_type_name: "msword".to_string()
            },
            param.value
        );
    }

    #[test]
    fn param_fb_type_free() {
        let (rem, param) = known_param::<Error>(b"FBTYPE=FREE;").unwrap();
        check_rem(rem, 1);
        assert_eq!("FBTYPE", param.name);
        assert_eq!(
            ParamValue::FreeBusyTimeType {
                fb_type: FreeBusyTimeType::Free
            },
            param.value
        );
    }

    #[test]
    fn param_fb_type_busy() {
        let (rem, param) = known_param::<Error>(b"FBTYPE=BUSY;").unwrap();
        check_rem(rem, 1);
        assert_eq!("FBTYPE", param.name);
        assert_eq!(
            ParamValue::FreeBusyTimeType {
                fb_type: FreeBusyTimeType::Busy
            },
            param.value
        );
    }

    #[test]
    fn param_fb_type_busy_unavailable() {
        let (rem, param) = known_param::<Error>(b"FBTYPE=BUSY-UNAVAILABLE;").unwrap();
        check_rem(rem, 1);
        assert_eq!("FBTYPE", param.name);
        assert_eq!(
            ParamValue::FreeBusyTimeType {
                fb_type: FreeBusyTimeType::BusyUnavailable
            },
            param.value
        );
    }

    #[test]
    fn param_fb_type_busy_tentative() {
        let (rem, param) = known_param::<Error>(b"FBTYPE=BUSY-TENTATIVE;").unwrap();
        check_rem(rem, 1);
        assert_eq!("FBTYPE", param.name);
        assert_eq!(
            ParamValue::FreeBusyTimeType {
                fb_type: FreeBusyTimeType::BusyTentative
            },
            param.value
        );
    }

    #[test]
    fn param_language() {
        let (rem, param) = known_param::<Error>(b"LANGUAGE=en-US;").unwrap();
        check_rem(rem, 1);
        assert_eq!("LANGUAGE", param.name);
        assert_eq!(
            ParamValue::Language {
                language: LanguageTag {
                    language: "en".to_string(),
                    region: Some("US".to_string()),
                    ..Default::default()
                }
            },
            param.value
        );
    }

    #[test]
    fn param_member() {
        let (rem, param) =
            known_param::<Error>(b"MEMBER=\"mailto:ietf-calsch@example.org\";").unwrap();
        check_rem(rem, 1);
        assert_eq!("MEMBER", param.name);
        assert_eq!(
            ParamValue::Members {
                members: vec!["mailto:ietf-calsch@example.org".to_string()],
            },
            param.value
        );
    }

    #[test]
    fn param_member_multi() {
        let (rem, param) = known_param::<Error>(
            b"MEMBER=\"mailto:projectA@example.com\",\"mailto:projectB@example.com\";",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!("MEMBER", param.name);
        assert_eq!(
            ParamValue::Members {
                members: vec![
                    "mailto:projectA@example.com".to_string(),
                    "mailto:projectB@example.com".to_string()
                ],
            },
            param.value
        );
    }

    #[test]
    fn param_part_stat_declined() {
        let (rem, param) = known_param::<Error>(b"PARTSTAT=DECLINED;").unwrap();
        check_rem(rem, 1);
        assert_eq!("PARTSTAT", param.name);
        assert_eq!(
            ParamValue::ParticipationStatus {
                status: ParticipationStatusUnknown::Declined
            },
            param.value
        );
    }

    #[test]
    fn param_range() {
        let (rem, param) = known_param::<Error>(b"RANGE=THISANDFUTURE;").unwrap();
        check_rem(rem, 1);
        assert_eq!("RANGE", param.name);
        assert_eq!(
            ParamValue::Range {
                range: Range::ThisAndFuture
            },
            param.value
        );
    }

    #[test]
    fn param_related_start() {
        let (rem, param) = known_param::<Error>(b"RELATED=START;").unwrap();
        check_rem(rem, 1);
        assert_eq!("RELATED", param.name);
        assert_eq!(
            ParamValue::Related {
                related: Related::Start
            },
            param.value
        );
    }

    #[test]
    fn param_related_end() {
        let (rem, param) = known_param::<Error>(b"RELATED=END;").unwrap();
        check_rem(rem, 1);
        assert_eq!("RELATED", param.name);
        assert_eq!(
            ParamValue::Related {
                related: Related::End
            },
            param.value
        );
    }

    #[test]
    fn param_rel_type() {
        let (rem, param) = known_param::<Error>(b"RELTYPE=SIBLING;").unwrap();
        check_rem(rem, 1);
        assert_eq!("RELTYPE", param.name);
        assert_eq!(
            ParamValue::RelationshipType {
                relationship: RelationshipType::Sibling
            },
            param.value
        );
    }

    #[test]
    fn param_role() {
        let (rem, param) = known_param::<Error>(b"ROLE=CHAIR;").unwrap();
        check_rem(rem, 1);
        assert_eq!("ROLE", param.name);
        assert_eq!(ParamValue::Role { role: Role::Chair }, param.value);
    }

    #[test]
    fn param_rsvp_true() {
        let (rem, param) = known_param::<Error>(b"RSVP=TRUE;").unwrap();
        check_rem(rem, 1);
        assert_eq!("RSVP", param.name);
        assert_eq!(ParamValue::Rsvp { rsvp: true }, param.value);
    }

    #[test]
    fn param_rsvp_false() {
        let (rem, param) = known_param::<Error>(b"RSVP=FALSE;").unwrap();
        check_rem(rem, 1);
        assert_eq!("RSVP", param.name);
        assert_eq!(ParamValue::Rsvp { rsvp: false }, param.value);
    }

    #[test]
    fn param_sent_by() {
        let (rem, param) = known_param::<Error>(b"SENT-BY=\"mailto:sray@example.com\";").unwrap();
        check_rem(rem, 1);
        assert_eq!("SENT-BY", param.name);
        assert_eq!(
            ParamValue::SentBy {
                address: "mailto:sray@example.com".to_string()
            },
            param.value
        );
    }

    #[test]
    fn param_tz_id() {
        let (rem, param) = known_param::<Error>(b"TZID=America/New_York;").unwrap();
        check_rem(rem, 1);
        assert_eq!("TZID", param.name);
        assert_eq!(
            ParamValue::TimeZoneId {
                tz_id: "America/New_York".to_string(),
                unique: false
            },
            param.value
        );
    }

    #[test]
    fn param_tz_id_unique() {
        let (rem, param) = known_param::<Error>(b"TZID=/America/New_York;").unwrap();
        check_rem(rem, 1);
        assert_eq!("TZID", param.name);
        assert_eq!(
            ParamValue::TimeZoneId {
                tz_id: "America/New_York".to_string(),
                unique: true
            },
            param.value
        );
    }

    #[test]
    fn param_value_binary() {
        let (rem, param) = known_param::<Error>(b"VALUE=BINARY;").unwrap();
        check_rem(rem, 1);
        assert_eq!("VALUE", param.name);
        assert_eq!(
            ParamValue::Value {
                value: Value::Binary
            },
            param.value
        );
    }
}
