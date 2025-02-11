use crate::common::ParticipationStatusUnknown;
use crate::common::{
    CalendarUserType, Encoding, FreeBusyTimeType, RelationshipType, Role, TriggerRelationship,
    Value,
};
use crate::parser::{iana_token, param_text, read_string, x_name, Error};
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::streaming::char;
use nom::combinator::{map_res, opt};
use nom::error::ParseError;
use nom::IResult;
use nom::Parser;

pub fn param_value_calendar_user_type<'a, E>(
    input: &'a [u8],
) -> IResult<&'a [u8], CalendarUserType, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, cu_type) = alt((
        tag_no_case("INDIVIDUAL").map(|_| CalendarUserType::Individual),
        tag_no_case("GROUP").map(|_| CalendarUserType::Group),
        tag_no_case("RESOURCE").map(|_| CalendarUserType::Resource),
        tag_no_case("ROOM").map(|_| CalendarUserType::Room),
        tag_no_case("UNKNOWN").map(|_| CalendarUserType::Unknown),
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
    ))
    .parse(input)?;

    Ok((input, cu_type))
}

pub fn param_value_encoding<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Encoding, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, encoding) = alt((
        tag_no_case("8BIT").map(|_| Encoding::EightBit),
        tag_no_case("BASE64").map(|_| Encoding::Base64),
    ))
    .parse(input)?;

    Ok((input, encoding))
}

pub fn param_value_free_busy_time_type<'a, E>(
    input: &'a [u8],
) -> IResult<&'a [u8], FreeBusyTimeType, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, fb_type) = alt((
        tag_no_case("FREE").map(|_| FreeBusyTimeType::Free),
        tag_no_case("BUSY-UNAVAILABLE").map(|_| FreeBusyTimeType::BusyUnavailable),
        tag_no_case("BUSY-TENTATIVE").map(|_| FreeBusyTimeType::BusyTentative),
        tag_no_case("BUSY").map(|_| FreeBusyTimeType::Busy),
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
    ))
    .parse(input)?;

    Ok((input, fb_type))
}

pub fn param_value_participation_status<'a, E>(
    input: &'a [u8],
) -> IResult<&'a [u8], ParticipationStatusUnknown, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, part_stat) = alt((
        tag_no_case("NEEDS-ACTION").map(|_| ParticipationStatusUnknown::NeedsAction),
        tag_no_case("ACCEPTED").map(|_| ParticipationStatusUnknown::Accepted),
        tag_no_case("DECLINED").map(|_| ParticipationStatusUnknown::Declined),
        tag_no_case("TENTATIVE").map(|_| ParticipationStatusUnknown::Tentative),
        tag_no_case("DELEGATED").map(|_| ParticipationStatusUnknown::Delegated),
        tag_no_case("COMPLETED").map(|_| ParticipationStatusUnknown::Completed),
        tag_no_case("IN-PROCESS").map(|_| ParticipationStatusUnknown::InProcess),
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
    ))
    .parse(input)?;

    Ok((input, part_stat))
}

pub fn param_value_trigger_relationship<'a, E>(
    input: &'a [u8],
) -> IResult<&'a [u8], TriggerRelationship, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, related) = alt((
        tag_no_case("START").map(|_| TriggerRelationship::Start),
        tag_no_case("END").map(|_| TriggerRelationship::End),
    ))
    .parse(input)?;

    Ok((input, related))
}

pub fn param_value_relationship_type<'a, E>(
    input: &'a [u8],
) -> IResult<&'a [u8], RelationshipType, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, rel_type) = alt((
        tag_no_case("PARENT").map(|_| RelationshipType::Parent),
        tag_no_case("CHILD").map(|_| RelationshipType::Child),
        tag_no_case("SIBLING").map(|_| RelationshipType::Sibling),
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
    ))
    .parse(input)?;

    Ok((input, rel_type))
}

pub fn param_value_role<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Role, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, role) = alt((
        tag_no_case("CHAIR").map(|_| Role::Chair),
        tag_no_case("REQ-PARTICIPANT").map(|_| Role::RequiredParticipant),
        tag_no_case("OPT-PARTICIPANT").map(|_| Role::OptionalParticipant),
        tag_no_case("NON-PARTICIPANT").map(|_| Role::NonParticipant),
        map_res(x_name, |x_name| {
            Ok(Role::XName(read_string(x_name, "ROLE x-name")?))
        }),
        map_res(iana_token, |iana_token| {
            Ok(Role::IanaToken(read_string(iana_token, "ROLE iana-token")?))
        }),
    ))
    .parse(input)?;

    Ok((input, role))
}

pub fn param_value_rsvp<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], bool, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, rsvp) = alt((
        tag_no_case("TRUE").map(|_| true),
        tag_no_case("FALSE").map(|_| false),
    ))
    .parse(input)?;

    Ok((input, rsvp))
}

pub fn param_value_time_zone_id<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], (String, bool), E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (unique, tz_id)) =
        (opt(char('/')).map(|p| p.is_some()), param_text).parse(input)?;

    Ok((input, (read_string(tz_id, "TZID")?, unique)))
}

pub fn param_value_value_type<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Value, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, value) = alt((
        tag_no_case("BINARY").map(|_| Value::Binary),
        tag_no_case("BOOLEAN").map(|_| Value::Boolean),
        tag_no_case("CAL-ADDRESS").map(|_| Value::CalendarAddress),
        tag_no_case("DATE-TIME").map(|_| Value::DateTime),
        tag_no_case("DATE").map(|_| Value::Date),
        tag_no_case("DURATION").map(|_| Value::Duration),
        tag_no_case("FLOAT").map(|_| Value::Float),
        tag_no_case("INTEGER").map(|_| Value::Integer),
        tag_no_case("PERIOD").map(|_| Value::Period),
        tag_no_case("RECUR").map(|_| Value::Recurrence),
        tag_no_case("TEXT").map(|_| Value::Text),
        tag_no_case("TIME").map(|_| Value::Time),
        tag_no_case("URI").map(|_| Value::Uri),
        tag_no_case("UTC-OFFSET").map(|_| Value::UtcOffset),
        map_res(x_name, |x_name| {
            Ok(Value::XName(read_string(x_name, "VALUE x-name")?))
        }),
        map_res(iana_token, |iana_token| {
            Ok(Value::IanaToken(read_string(
                iana_token,
                "VALUE iana-token",
            )?))
        }),
    ))
    .parse(input)?;

    Ok((input, value))
}
