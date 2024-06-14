use crate::parser::param::{
    CalendarUserType, Encoding, FreeBusyTimeType, ParticipationStatusUnknown, Related,
    RelationshipType, Role, Value,
};
use crate::parser::{iana_token, param_text, read_string, x_name, Error};
use nom::branch::alt;
use nom::bytes::streaming::tag;
use nom::character::streaming::char;
use nom::combinator::{map_res, opt};
use nom::error::ParseError;
use nom::sequence::tuple;
use nom::IResult;
use nom::Parser;

pub fn param_calendar_user_type<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], CalendarUserType, E> {
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

pub fn param_encoding<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], Encoding, E> {
    let (input, encoding) = alt((
        tag("8BIT").map(|_| Encoding::EightBit),
        tag("BASE64").map(|_| Encoding::Base64),
    ))(input)?;

    Ok((input, encoding))
}

/// See https://www.rfc-editor.org/rfc/rfc5545 section 3.2.9
pub fn param_free_busy_time_type<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], FreeBusyTimeType, E> {
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

pub fn param_part_stat<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], ParticipationStatusUnknown, E> {
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

pub fn param_related<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], Related, E> {
    let (input, related) = alt((
        tag("START").map(|_| Related::Start),
        tag("END").map(|_| Related::End),
    ))(input)?;

    Ok((input, related))
}

pub fn param_rel_type<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], RelationshipType, E> {
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

pub fn param_role<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], Role, E> {
    let (input, role) = alt((
        tag("CHAIR").map(|_| Role::Chair),
        tag("REQ-PARTICIPANT").map(|_| Role::RequiredParticipant),
        tag("OPT-PARTICIPANT").map(|_| Role::OptionalParticipant),
        tag("NON-PARTICIPANT").map(|_| Role::NonParticipant),
        map_res(x_name, |x_name| {
            Ok(Role::XName(read_string(x_name, "ROLE x-name")?))
        }),
        map_res(iana_token, |iana_token| {
            Ok(Role::IanaToken(read_string(iana_token, "ROLE iana-token")?))
        }),
    ))(input)?;

    Ok((input, role))
}

pub fn param_rsvp<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], bool, E> {
    let (input, rsvp) = alt((tag("TRUE").map(|_| true), tag("FALSE").map(|_| false)))(input)?;

    Ok((input, rsvp))
}

pub fn param_tz_id<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], (String, bool), E> {
    let (input, (unique, tz_id)) = tuple((opt(char('/')).map(|p| p.is_some()), param_text))(input)?;

    Ok((input, (read_string(tz_id, "TZID")?, unique)))
}

pub fn param_value_type<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], Value, E> {
    let (input, value) = alt((
        tag("BINARY").map(|_| Value::Binary),
        tag("BOOLEAN").map(|_| Value::Boolean),
        tag("CAL-ADDRESS").map(|_| Value::CalendarAddress),
        tag("DATE-TIME").map(|_| Value::DateTime),
        tag("DATE").map(|_| Value::Date),
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
            Ok(Value::XName(read_string(x_name, "VALUE x-name")?))
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
