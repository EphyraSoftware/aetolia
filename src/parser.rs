#![allow(dead_code)]

use crate::parser::language_tag::LanguageTag;
use nom::branch::alt;
use nom::bytes::complete::{take_until, take_while, take_while1, take_while_m_n};
use nom::bytes::streaming::tag;
use nom::character::streaming::{alphanumeric1, char, crlf};
use nom::combinator::{map_res, opt, recognize};
use nom::error::{ErrorKind, FromExternalError, ParseError};
use nom::multi::separated_list1;
use nom::sequence::{separated_pair, tuple};
use nom::{AsChar, IResult, Parser};

mod language_tag;

#[derive(Clone, Debug, PartialEq)]
pub struct Error<'a> {
    pub input: &'a [u8],
    pub error: InnerError,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InnerError {
    Nom(ErrorKind),
    XNameTooShort,
    EncodingError(String, std::str::Utf8Error),
}

impl<'a> Error<'a> {
    pub fn new(input: &'a [u8], error: InnerError) -> Error<'a> {
        Error { input, error }
    }
}

impl<'a> ParseError<&'a [u8]> for Error<'a> {
    fn from_error_kind(input: &'a [u8], kind: ErrorKind) -> Self {
        Error {
            input,
            error: InnerError::Nom(kind),
        }
    }

    fn append(input: &'a [u8], kind: ErrorKind, _other: Self) -> Self {
        Error {
            input,
            error: InnerError::Nom(kind),
        }
    }
}

// Enables use of `map_res` with nom::Err for the custom Error type.
impl<'a> FromExternalError<&'a [u8], nom::Err<Error<'a>>> for Error<'a> {
    fn from_external_error(input: &'a [u8], kind: ErrorKind, e: nom::Err<Error<'a>>) -> Self {
        match e {
            nom::Err::Error(e) | nom::Err::Failure(e) => Error {
                input: e.input,
                error: e.error,
            },
            nom::Err::Incomplete(_) => Error {
                input,
                error: InnerError::Nom(kind),
            },
        }
    }
}

impl<'a> From<(&'a [u8], ErrorKind)> for Error<'a> {
    fn from((input, kind): (&'a [u8], ErrorKind)) -> Self {
        Error {
            input,
            error: InnerError::Nom(kind),
        }
    }
}

#[derive(Debug, Clone)]
struct ContentLine<'a> {
    property_name: &'a [u8],
    value: Vec<u8>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Param {
    name: String,
    value: ParamValue,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum ParamValue {
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
        range: Range
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

/// All ASCII control characters except tab (%x09).
#[inline]
const fn is_control(b: u8) -> bool {
    matches!(b, b'\0'..=b'\x08' | b'\x0A'..=b'\x1F' | b'\x7F')
}

fn param_text(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    take_while(|c| c != b'\"' && c != b';' && c != b':' && c != b',' && !is_control(c))(input)
}

fn quoted_string(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    let (input, (_, content, _)) = tuple((
        char('"'),
        take_while(|c| c != b'\"' && !is_control(c)),
        char('"'),
    ))(input)?;

    Ok((input, content))
}

fn param_value(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    let (input, value) = alt((quoted_string, param_text))(input)?;

    Ok((input, value))
}

fn safe_char(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    take_while(|c| c != b'\"' && !is_control(c))(input)
}

fn iana_token(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    take_while1(|c: u8| c.is_alphanum() || c == b'-')(input)
}

fn vendor_id(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    let (rest, id) = alphanumeric1(input)?;

    if id.len() < 3 {
        return Err(nom::Err::Failure(Error::new(
            rest,
            InnerError::XNameTooShort,
        )));
    }

    Ok((rest, id))
}

fn x_name(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    let (input, x_name) = recognize(tuple((
        tag("X-"),
        opt(tuple((vendor_id, char('-')))),
        take_while1(|c: u8| c.is_alphanum() || c == b'-'),
    )))(input)?;

    Ok((input, x_name))
}

fn name(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    alt((iana_token, x_name))(input)
}

fn param_name(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    alt((iana_token, x_name))(input)
}

fn param_calendar_user_type(input: &[u8]) -> IResult<&[u8], CalendarUserType, Error> {
    let (input, cu_type) = alt((
        tag("INDIVIDUAL").map(|_| CalendarUserType::Individual),
        tag("GROUP").map(|_| CalendarUserType::Group),
        tag("RESOURCE").map(|_| CalendarUserType::Resource),
        tag("ROOM").map(|_| CalendarUserType::Room),
        tag("UNKNOWN").map(|_| CalendarUserType::Unknown),
        map_res(x_name, |x_name| {
            Ok(CalendarUserType::XName(read_string(
                x_name,
                "CUTYPE x-name",
            )?))
        }),
        map_res(iana_token, |iana_token| {
            Ok(CalendarUserType::IanaToken(read_string(
                iana_token,
                "CUTYPE iana-token",
            )?))
        }),
    ))(input)?;

    Ok((input, cu_type))
}

fn param_encoding(input: &[u8]) -> IResult<&[u8], Encoding, Error> {
    let (input, encoding) = alt((
        tag("8BIT").map(|_| Encoding::EightBit),
        tag("BASE64").map(|_| Encoding::Base64),
    ))(input)?;

    Ok((input, encoding))
}

/// See https://www.rfc-editor.org/rfc/rfc5545 section 3.2.9
fn param_free_busy_time_type(input: &[u8]) -> IResult<&[u8], FreeBusyTimeType, Error> {
    let (input, fb_type) = alt((
        tag("FREE").map(|_| FreeBusyTimeType::Free),
        tag("BUSY-UNAVAILABLE").map(|_| FreeBusyTimeType::BusyUnavailable),
        tag("BUSY-TENTATIVE").map(|_| FreeBusyTimeType::BusyTentative),
        tag("BUSY").map(|_| FreeBusyTimeType::Busy),
        map_res(x_name, |x_name| {
            Ok(FreeBusyTimeType::XName(read_string(
                x_name,
                "FBTYPE x-name",
            )?))
        }),
        map_res(iana_token, |iana_token| {
            Ok(FreeBusyTimeType::IanaToken(read_string(
                iana_token,
                "FBTYPE iana-token",
            )?))
        }),
    ))(input)?;

    println!("fb_type: {:?}", fb_type);

    Ok((input, fb_type))
}

fn param_part_stat(input: &[u8]) -> IResult<&[u8], ParticipationStatusUnknown, Error> {
    let (input, part_stat) = alt((
        tag("NEEDS-ACTION").map(|_| ParticipationStatusUnknown::NeedsAction),
        tag("ACCEPTED").map(|_| ParticipationStatusUnknown::Accepted),
        tag("DECLINED").map(|_| ParticipationStatusUnknown::Declined),
        tag("TENTATIVE").map(|_| ParticipationStatusUnknown::Tentative),
        tag("DELEGATED").map(|_| ParticipationStatusUnknown::Delegated),
        tag("COMPLETED").map(|_| ParticipationStatusUnknown::Completed),
        tag("IN-PROCESS").map(|_| ParticipationStatusUnknown::InProcess),
        map_res(x_name, |x_name| {
            Ok(ParticipationStatusUnknown::XName(read_string(
                x_name,
                "PARTSTAT x-name",
            )?))
        }),
        map_res(iana_token, |iana_token| {
            Ok(ParticipationStatusUnknown::IanaToken(read_string(
                iana_token,
                "PARTSTAT iana-token",
            )?))
        }),
    ))(input)?;

    Ok((input, part_stat))
}

fn param_related(input: &[u8]) -> IResult<&[u8], Related, Error> {
    let (input, related) = alt((
        tag("START").map(|_| Related::Start),
        tag("END").map(|_| Related::End),
    ))(input)?;

    Ok((input, related))
}

fn param_rel_type(input: &[u8]) -> IResult<&[u8], RelationshipType, Error> {
    let (input, rel_type) = alt((
        tag("PARENT").map(|_| RelationshipType::Parent),
        tag("CHILD").map(|_| RelationshipType::Child),
        tag("SIBLING").map(|_| RelationshipType::Sibling),
        map_res(x_name, |x_name| {
            Ok(RelationshipType::XName(read_string(
                x_name,
                "RELTYPE x-name",
            )?))
        }),
        map_res(iana_token, |iana_token| {
            Ok(RelationshipType::IanaToken(read_string(
                iana_token,
                "RELTYPE iana-token",
            )?))
        }),
    ))(input)?;

    Ok((input, rel_type))
}

fn param_role(input: &[u8]) -> IResult<&[u8], Role, Error> {
    let (input, role) = alt((
        tag("CHAIR").map(|_| Role::Chair),
        tag("REQ-PARTICIPANT").map(|_| Role::RequiredParticipant),
        tag("OPT-PARTICIPANT").map(|_| Role::OptionalParticipant),
        tag("NON-PARTICIPANT").map(|_| Role::NonParticipant),
        map_res(x_name, |x_name| {
            Ok(Role::XName(read_string(
                x_name,
                "ROLE x-name",
            )?))
        }),
        map_res(iana_token, |iana_token| {
            Ok(Role::IanaToken(read_string(
                iana_token,
                "ROLE iana-token",
            )?))
        }),
    ))(input)?;

    Ok((input, role))
}

fn param_rsvp(input: &[u8]) -> IResult<&[u8], bool, Error> {
    let (input, rsvp) = alt((
        tag("TRUE").map(|_| true),
        tag("FALSE").map(|_| false),
    ))(input)?;

    Ok((input, rsvp))
}

fn param_tz_id(input: &[u8]) -> IResult<&[u8], (String, bool), Error> {
    let (input, (unique, tz_id)) = tuple((opt(char('/')).map(|p| p.is_some()), param_text))(input)?;

    Ok((input, (read_string(tz_id, "TZID")?, unique)))
}

fn param_value_type(input: &[u8]) -> IResult<&[u8], Value, Error> {
    let (input, value) = alt((
        tag("BINARY").map(|_| Value::Binary),
        tag("BOOLEAN").map(|_| Value::Boolean),
        tag("CAL-ADDRESS").map(|_| Value::CalendarAddress),
        tag("DATE").map(|_| Value::Date),
        tag("DATE-TIME").map(|_| Value::DateTime),
        tag("DURATION").map(|_| Value::Duration),
        tag("FLOAT").map(|_| Value::Float),
        tag("INTEGER").map(|_| Value::Integer),
        tag("PERIOD").map(|_| Value::Period),
        tag("RECUR").map(|_| Value::Recurrence),
        tag("TEXT").map(|_| Value::Text),
        tag("TIME").map(|_| Value::Time),
        tag("URI").map(|_| Value::Uri),
        tag("UTC-OFFSET").map(|_| Value::UtcOffset),
        map_res(x_name, |x_name| {
            Ok(Value::XName(read_string(
                x_name,
                "VALUE x-name",
            )?))
        }),
        map_res(iana_token, |iana_token| {
            Ok(Value::IanaToken(read_string(
                iana_token,
                "VALUE iana-token",
            )?))
        }),
    ))(input)?;

    Ok((input, value))
}

#[inline]
const fn is_reg_name_char(b: u8) -> bool {
    matches!(b, b'\x41'..=b'\x5A' | b'\x61'..=b'\x7A' | b'\x30'..=b'\x39' | b'\x21' | b'\x23' | b'\x24' | b'\x26' | b'\x2E' | b'\x2B' | b'\x2D' | b'\x5E' | b'\x5F')
}

// See https://www.rfc-editor.org/rfc/rfc4288 section 4.2
fn reg_name(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    take_while_m_n(1, 127, is_reg_name_char)(input)
}

fn param(input: &[u8]) -> IResult<&[u8], Option<Param>, Error> {
    let (input, (name, _)) = tuple((param_name, char('=')))(input)?;

    let name_s = read_string(name, "param_name")?;
    let (input, maybe_param_value) = match name_s.as_str() {
        "ALTREP" => {
            // Requires a quoted string rather than a param-value
            let (input, uri) = quoted_string(input)?;

            (
                input,
                Some(ParamValue::AltRep {
                    uri: read_string(uri, "uri")?,
                }),
            )
        }
        "CN" => {
            let (input, value) = param_value(input)?;

            (
                input,
                Some(ParamValue::CommonName {
                    name: read_string(value, "common_name")?,
                }),
            )
        }
        "CUTYPE" => {
            let (input, cu_type) = param_calendar_user_type(input)?;

            (input, Some(ParamValue::CalendarUserType { cu_type }))
        }
        "DELEGATED-FROM" => {
            // Technically should be 'cal-address' but that's not defined at this point in the spec. Different to quoted string?
            let (input, delegators) = separated_list1(
                char(','),
                map_res(quoted_string, |d| {
                    read_string(d, "DELEGATED-FROM cal-address")
                }),
            )(input)?;

            (input, Some(ParamValue::DelegatedFrom { delegators }))
        }
        "DELEGATED-TO" => {
            // Technically should be 'cal-address' but that's not defined at this point in the spec. Different to quoted string?
            let (input, delegates) = separated_list1(
                char(','),
                map_res(quoted_string, |d| {
                    read_string(d, "DELEGATED-TO cal-address")
                }),
            )(input)?;

            (input, Some(ParamValue::DelegatedTo { delegates }))
        }
        "DIR" => {
            let (input, uri) = quoted_string(input)?;

            (
                input,
                Some(ParamValue::Dir {
                    uri: read_string(uri, "dir")?,
                }),
            )
        }
        "ENCODING" => {
            let (input, encoding) = param_encoding(input)?;

            (input, Some(ParamValue::Encoding { encoding }))
        }
        "FMTTYPE" => {
            let (input, (type_name, sub_type_name)) = separated_pair(
                map_res(reg_name, |t| read_string(t, "FMTTYPE type-name")),
                char('/'),
                map_res(reg_name, |t| read_string(t, "FMTTYPE subtype-name")),
            )(input)?;

            (
                input,
                Some(ParamValue::FormatType {
                    type_name,
                    sub_type_name,
                }),
            )
        }
        "FBTYPE" => {
            let (input, fb_type) = param_free_busy_time_type(input)?;

            (input, Some(ParamValue::FreeBusyTimeType { fb_type }))
        }
        "LANGUAGE" => {
            let (input, language) = language_tag::language_tag(input)?;

            (input, Some(ParamValue::Language { language }))
        }
        "MEMBER" => {
            let (input, members) = separated_list1(
                char(','),
                map_res(quoted_string, |m| read_string(m, "MEMBER cal-address")),
            )(input)?;

            (input, Some(ParamValue::Members { members }))
        }
        "PARTSTAT" => {
            let (input, status) = param_part_stat(input)?;

            (input, Some(ParamValue::ParticipationStatus { status }))
        }
        "RANGE" => {
            let (input, _) = tag("THISANDFUTURE")(input)?;

            (input, Some(ParamValue::Range { range: Range::ThisAndFuture }))
        }
        "RELATED" => {
            let (input, related) = param_related(input)?;

            (input, Some(ParamValue::Related { related }))
        }
        "RELTYPE" => {
            let (input, relationship) = param_rel_type(input)?;

            (input, Some(ParamValue::RelationshipType { relationship }))
        }
        "ROLE" => {
            let (input, role) = param_role(input)?;

            (input, Some(ParamValue::Role { role }))
        }
        "RSVP" => {
            let (input, rsvp) = param_rsvp(input)?;

            (input, Some(ParamValue::Rsvp { rsvp }))
        }
        "SENT-BY" => {
            let (input, address) = quoted_string(input)?;

            (
                input,
                Some(ParamValue::SentBy {
                    address: read_string(address, "SENT-BY address")?,
                }),
            )
        }
        "TZID" => {
            let (input, (tz_id, unique)) = param_tz_id(input)?;

            (input, Some(ParamValue::TimeZoneId { tz_id, unique }))
        }
        "VALUE" => {
            let (input, value) = param_value_type(input)?;

            (input, Some(ParamValue::Value { value }))
        }
        _ => {
            // TODO not robust! Check 3
            let (input, _) = take_until(";")(input)?;

            (input, None)
        }
    };

    Ok((
        input,
        maybe_param_value.map(|param_value| Param {
            name: name_s,
            value: param_value,
        }),
    ))
}

fn read_string<'a>(input: &'a [u8], context: &str) -> Result<String, nom::Err<Error<'a>>> {
    Ok(std::str::from_utf8(input)
        .map_err(|e| {
            nom::Err::Failure(Error::new(
                input,
                InnerError::EncodingError(context.to_string(), e),
            ))
        })?
        .to_string())
}

fn parse_line_content(input: &[u8]) -> IResult<&[u8], Vec<u8>, Error> {
    let (input, (parts, _)) = tuple((
        separated_list1(
            tuple((crlf, alt((char(' '), char('\t'))))),
            take_until("\r\n"),
        ),
        crlf,
    ))(input)?;

    Ok((
        input,
        parts.iter().fold(vec![], |mut acc, x| {
            acc.extend_from_slice(x);
            acc
        }),
    ))
}

fn parse_line(input: &[u8]) -> IResult<&[u8], ContentLine, Error> {
    let (input, (property_name, value)) =
        separated_pair(name, char(':'), parse_line_content)(input)?;

    Ok((
        input,
        ContentLine {
            property_name,
            value,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::check_rem;

    #[test]
    fn iana_token_desc() {
        let (rem, token) = iana_token(b"DESCRIPTION").unwrap();
        check_rem(rem, 0);
        assert_eq!(b"DESCRIPTION", token);
    }

    #[test]
    fn simple_x_name() {
        let (rem, x_name) = x_name(b"X-TEST ").unwrap();
        check_rem(rem, 1);
        assert_eq!(b"X-TEST", x_name);
    }

    #[test]
    fn simple_x_name_with_vendor() {
        let (rem, x_name) = x_name(b"X-ESL-TEST ").unwrap();
        check_rem(rem, 1);
        assert_eq!(b"X-ESL-TEST", x_name);
    }

    #[test]
    fn param_altrep() {
        let (rem, param) = param(b"ALTREP=\"http://example.com/calendar\";").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"CN=\"John Smith\";").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"CN=Danny;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"CUTYPE=INDIVIDUAL;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"CUTYPE=GROUP;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"CUTYPE=RESOURCE;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"CUTYPE=ROOM;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"CUTYPE=UNKNOWN;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"CUTYPE=X-esl-special;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"CUTYPE=other;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"DELEGATED-FROM=\"mailto:jsmith@example.com\";").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) =
            param(b"DELEGATED-FROM=\"mailto:jsmith@example.com\",\"mailto:danny@example.com\";")
                .unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"DELEGATED-TO=\"mailto:jsmith@example.com\";").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) =
            param(b"DELEGATED-TO=\"mailto:jsmith@example.com\",\"mailto:danny@example.com\";")
                .unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(
            b"DIR=\"ldap://example.com:6666/o=ABC%20Industries,c=US???(cn=Jim%20Dolittle)\";",
        )
        .unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"ENCODING=8BIT;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"ENCODING=BASE64;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"FMTTYPE=application/msword;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"FBTYPE=FREE;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"FBTYPE=BUSY;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"FBTYPE=BUSY-UNAVAILABLE;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"FBTYPE=BUSY-TENTATIVE;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"LANGUAGE=en-US;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"MEMBER=\"mailto:ietf-calsch@example.org\";").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) =
            param(b"MEMBER=\"mailto:projectA@example.com\",\"mailto:projectB@example.com\";")
                .unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"PARTSTAT=DECLINED;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"RANGE=THISANDFUTURE;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"RELATED=START;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"RELATED=END;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"RELTYPE=SIBLING;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"ROLE=CHAIR;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
        assert_eq!("ROLE", param.name);
        assert_eq!(
            ParamValue::Role {
                role: Role::Chair
            },
            param.value
        );
    }

    #[test]
    fn param_rsvp_true() {
        let (rem, param) = param(b"RSVP=TRUE;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
        assert_eq!("RSVP", param.name);
        assert_eq!(
            ParamValue::Rsvp {
                rsvp: true
            },
            param.value
        );
    }

    #[test]
    fn param_rsvp_false() {
        let (rem, param) = param(b"RSVP=FALSE;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
        assert_eq!("RSVP", param.name);
        assert_eq!(
            ParamValue::Rsvp {
                rsvp: false
            },
            param.value
        );
    }

    #[test]
    fn param_sent_by() {
        let (rem, param) = param(b"SENT-BY=\"mailto:sray@example.com\";").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"TZID=America/New_York;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"TZID=/America/New_York;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
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
        let (rem, param) = param(b"VALUE=BINARY;").unwrap();
        check_rem(rem, 1);
        let param = param.unwrap();
        assert_eq!("VALUE", param.name);
        assert_eq!(
            ParamValue::Value {
                value: Value::Binary
            },
            param.value
        );
    }

    #[test]
    fn simple_content_line() {
        let (rem, content_line) = parse_line(
            b"DESCRIPTION:This is a long description that exists on a long line.\r\nnext",
        )
        .unwrap();
        check_rem(rem, 4);
        assert_eq!(b"DESCRIPTION", content_line.property_name);
        assert_eq!(
            b"This is a long description that exists on a long line.",
            content_line.value.as_slice()
        );
    }

    #[test]
    fn content_line_multi_line() {
        let (rem, content_line) = parse_line(
            b"DESCRIPTION:This is a lo\r\n ng description\r\n  that exists on a long line.\r\nnext",
        )
        .unwrap();
        check_rem(rem, 4);
        assert_eq!(b"DESCRIPTION", content_line.property_name);
        assert_eq!(
            b"This is a long description that exists on a long line.",
            content_line.value.as_slice(),
            "Got: {}",
            String::from_utf8(content_line.value.clone()).unwrap()
        );
    }

    #[test]
    fn content_line_multi_line_with_tab() {
        let (rem, content_line) =
            parse_line(b"DESCRIPTION:This is a lo\r\n ng description\r\n\t that exists on a long line.\r\nnext")
                .unwrap();
        check_rem(rem, 4);
        assert_eq!(b"DESCRIPTION", content_line.property_name);
        assert_eq!(
            b"This is a long description that exists on a long line.",
            content_line.value.as_slice(),
            "Got: {}",
            String::from_utf8(content_line.value.clone()).unwrap()
        );
    }
}
