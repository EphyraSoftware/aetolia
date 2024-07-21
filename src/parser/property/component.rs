use crate::common::{Encoding, Status, TimeTransparency, Value};
use crate::parser::param::{other_params, params, ParamValue};
use crate::parser::property::recur::{recur, RecurRulePart};
use crate::parser::property::uri::{param_value_uri, Uri};
use crate::parser::property::{
    prop_value_binary, prop_value_calendar_user_address, prop_value_date, prop_value_date_time,
    prop_value_duration, prop_value_float, prop_value_integer, prop_value_period, prop_value_text,
    prop_value_utc_offset, DateOrDateTime, DateOrDateTimeOrPeriod, DateTime, Duration, Period,
    UtcOffset,
};
use crate::parser::{iana_token, read_int, x_name, Error, InnerError};
use nom::branch::alt;
use nom::bytes::complete::{tag_no_case, take_while1};
use nom::bytes::streaming::tag;
use nom::character::is_digit;
use nom::character::streaming::char;
use nom::combinator::{cut, map_res, opt, recognize, verify};
use nom::error::ParseError;
use nom::multi::{fold_many_m_n, separated_list1};
use nom::sequence::tuple;
use nom::{IResult, Parser};

#[derive(Debug, Eq, PartialEq)]
pub enum AttachValue<'a> {
    Uri(&'a [u8]),
    Binary(&'a [u8]),
}

#[derive(Debug, Eq, PartialEq)]
pub struct AttachProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: AttachValue<'a>,
}

/// Parse an ATTACH property.
///
/// RFC 5545, section 3.8.1.1
pub fn prop_attach<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], AttachProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, params, _)) = tuple((tag_no_case("ATTACH"), cut(params), char(':')))(input)?;

    let is_base_64 = params.iter().any(|p| {
        matches!(
            p,
            ParamValue::Encoding {
                encoding: Encoding::Base64,
            }
        )
    });

    let is_binary = params.iter().any(|p| {
        matches!(
            p,
            ParamValue::ValueType {
                value: Value::Binary,
            }
        )
    });

    // Use OR here rather than AND. It's not valid to set one of these and not the other so assume the
    // value is more likely to be binary if one is set and let the error happen later if so.
    if is_base_64 || is_binary {
        let (input, (v, _)) = tuple((cut(prop_value_binary), tag("\r\n")))(input)?;

        Ok((
            input,
            AttachProperty {
                params,
                value: AttachValue::Binary(v),
            },
        ))
    } else {
        let (input, (v, _)) = tuple((cut(recognize(param_value_uri)), tag("\r\n")))(input)?;

        Ok((
            input,
            AttachProperty {
                params,
                value: AttachValue::Uri(v),
            },
        ))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct CategoriesProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: Vec<Vec<u8>>,
}

/// Parse a CATEGORIES property.
///
/// RFC 5545, section 3.8.1.2
pub fn prop_categories<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], CategoriesProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, (params, _, value, _))) = tuple((
        tag_no_case("CATEGORIES"),
        cut(tuple((
            params,
            char(':'),
            separated_list1(char(','), prop_value_text),
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((input, CategoriesProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub enum Classification<'a> {
    Public,
    Private,
    Confidential,
    XName(&'a [u8]),
    IanaToken(&'a [u8]),
}

#[derive(Debug, Eq, PartialEq)]
pub struct ClassificationProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: Classification<'a>,
}

/// Parse a CLASS property.
///
/// RFC 5545, section 3.8.1.3
pub fn prop_classification<'a, E>(
    input: &'a [u8],
) -> IResult<&'a [u8], ClassificationProperty<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (_, (other_params, _, value, _))) = tuple((
        tag_no_case("CLASS"),
        cut(tuple((
            other_params,
            char(':'),
            alt((
                tag_no_case("PUBLIC").map(|_| Classification::Public),
                tag_no_case("PRIVATE").map(|_| Classification::Private),
                tag_no_case("CONFIDENTIAL").map(|_| Classification::Confidential),
                x_name.map(Classification::XName),
                iana_token.map(Classification::IanaToken),
            )),
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((
        input,
        ClassificationProperty {
            other_params,
            value,
        },
    ))
}

#[derive(Debug, Eq, PartialEq)]
pub struct CommentProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: Vec<u8>,
}

/// Parse a COMMENT property.
///
/// RFC 5545, section 3.8.1.4
pub fn prop_comment<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], CommentProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, (params, _, value, _))) = tuple((
        tag_no_case("COMMENT"),
        cut(tuple((params, char(':'), prop_value_text, tag("\r\n")))),
    ))(input)?;

    Ok((input, CommentProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct DescriptionProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: Vec<u8>,
}

/// Parse a DESCRIPTION property.
///
/// RFC 5545, section 3.8.1.5
pub fn prop_description<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], DescriptionProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, (params, _, value, _))) = tuple((
        tag_no_case("DESCRIPTION"),
        cut(tuple((
            params,
            char(':'),
            prop_value_text.map(|v| v),
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((input, DescriptionProperty { params, value }))
}

#[derive(Debug, PartialEq)]
pub struct GeographicPositionProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub latitude: f64,
    pub longitude: f64,
}

/// Parse a GEO property.
///
/// RFC 5545, section 3.8.1.6
pub fn prop_geographic_position<'a, E>(
    input: &'a [u8],
) -> IResult<&'a [u8], GeographicPositionProperty<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (_, (other_params, _, (latitude, _, longitude), _))) = tuple((
        tag_no_case("GEO"),
        cut(tuple((
            other_params,
            char(':'),
            tuple((prop_value_float, char(';'), prop_value_float)),
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((
        input,
        GeographicPositionProperty {
            other_params,
            latitude,
            longitude,
        },
    ))
}

#[derive(Debug, Eq, PartialEq)]
pub struct LocationProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: Vec<u8>,
}

/// Parse a LOCATION property.
///
/// RFC 5545, section 3.8.1.7
pub fn prop_location<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], LocationProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, (params, _, value, _))) = tuple((
        tag_no_case("LOCATION"),
        cut(tuple((params, char(':'), prop_value_text, tag("\r\n")))),
    ))(input)?;

    Ok((input, LocationProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct PercentCompleteProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: u8,
}

/// Parse a PERCENT-COMPLETE property.
///
/// RFC 5545, section 3.8.1.8
pub fn prop_percent_complete<'a, E>(
    input: &'a [u8],
) -> IResult<&'a [u8], PercentCompleteProperty<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (_, (other_params, _, value, _))) = tuple((
        tag_no_case("PERCENT-COMPLETE"),
        cut(tuple((
            other_params,
            char(':'),
            verify(prop_value_integer, |v| 0 <= *v && *v <= 100).map(|v| v as u8),
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((
        input,
        PercentCompleteProperty {
            other_params,
            value,
        },
    ))
}

#[derive(Debug, Eq, PartialEq)]
pub struct PriorityProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: u8,
}

/// Parse a PRIORITY property.
///
/// RFC 5545, section 3.8.1.9
pub fn prop_priority<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], PriorityProperty<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (_, (other_params, _, value, _))) = tuple((
        tag_no_case("PRIORITY"),
        cut(tuple((
            other_params,
            char(':'),
            verify(prop_value_integer, |v| 0 <= *v && *v <= 9).map(|v| v as u8),
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((
        input,
        PriorityProperty {
            other_params,
            value,
        },
    ))
}

#[derive(Debug, Eq, PartialEq)]
pub struct ResourcesProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: Vec<Vec<u8>>,
}

/// Parse a RESOURCES property.
///
/// RFC 5545, section 3.8.1.10
pub fn prop_resources<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ResourcesProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, (params, _, value, _))) = tuple((
        tag_no_case("RESOURCES"),
        cut(tuple((
            params,
            char(':'),
            separated_list1(char(','), prop_value_text),
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((input, ResourcesProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct StatusProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: Status,
}

/// Parse a STATUS property.
///
/// RFC 5545, section 3.8.1.11
pub fn prop_status<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], StatusProperty<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (_, (other_params, _, value, _))) = tuple((
        tag_no_case("STATUS"),
        cut(tuple((
            other_params,
            char(':'),
            alt((
                tag_no_case("TENTATIVE").map(|_| Status::Tentative),
                tag_no_case("CONFIRMED").map(|_| Status::Confirmed),
                tag_no_case("CANCELLED").map(|_| Status::Cancelled),
                tag_no_case("NEEDS-ACTION").map(|_| Status::NeedsAction),
                tag_no_case("COMPLETED").map(|_| Status::Completed),
                tag_no_case("IN-PROCESS").map(|_| Status::InProcess),
                tag_no_case("DRAFT").map(|_| Status::Draft),
                tag_no_case("FINAL").map(|_| Status::Final),
            )),
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((
        input,
        StatusProperty {
            other_params,
            value,
        },
    ))
}

#[derive(Debug, Eq, PartialEq)]
pub struct SummaryProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: Vec<u8>,
}

/// Parse a SUMMARY property.
///
/// RFC 5545, section 3.8.1.12
pub fn prop_summary<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], SummaryProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, (params, _, value, _))) = tuple((
        tag_no_case("SUMMARY"),
        cut(tuple((params, char(':'), prop_value_text, tag("\r\n")))),
    ))(input)?;

    Ok((input, SummaryProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct DateTimeCompletedProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: DateTime,
}

/// Parse a COMPLETED property.
///
/// RFC 5545, section 3.8.2.1
pub fn prop_date_time_completed<'a, E>(
    input: &'a [u8],
) -> IResult<&'a [u8], DateTimeCompletedProperty<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (_, (other_params, _, value, _))) = tuple((
        tag_no_case("COMPLETED"),
        cut(tuple((
            other_params,
            char(':'),
            prop_value_date_time,
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((
        input,
        DateTimeCompletedProperty {
            other_params,
            value,
        },
    ))
}

#[derive(Debug, Eq, PartialEq)]
pub struct DateTimeEndProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: DateOrDateTime,
}

/// Parse a DTEND property.
///
/// RFC 5545, section 3.8.2.2
pub fn prop_date_time_end<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], DateTimeEndProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, (params, _, value, _))) = tuple((
        tag_no_case("DTEND"),
        cut(tuple((
            params,
            char(':'),
            alt((
                prop_value_date_time.map(DateOrDateTime::DateTime),
                prop_value_date.map(DateOrDateTime::Date),
            )),
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((input, DateTimeEndProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct DateTimeDueProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: DateOrDateTime,
}

/// Parse a DUE property.
///
/// RFC 5545, section 3.8.2.3
pub fn prop_date_time_due<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], DateTimeDueProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, (params, _, value, _))) = tuple((
        tag_no_case("DUE"),
        cut(tuple((
            params,
            char(':'),
            alt((
                prop_value_date_time.map(DateOrDateTime::DateTime),
                prop_value_date.map(DateOrDateTime::Date),
            )),
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((input, DateTimeDueProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct DateTimeStartProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: DateOrDateTime,
}

/// Parse a DTSTART property.
///
/// RFC 5545, section 3.8.2.4
pub fn prop_date_time_start<'a, E>(
    input: &'a [u8],
) -> IResult<&'a [u8], DateTimeStartProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, (params, _, value, _))) = tuple((
        tag_no_case("DTSTART"),
        cut(tuple((
            params,
            char(':'),
            alt((
                prop_value_date_time.map(DateOrDateTime::DateTime),
                prop_value_date.map(DateOrDateTime::Date),
            )),
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((input, DateTimeStartProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct DurationProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: Duration,
}

/// Parse a DURATION property.
///
/// RFC 5545, section 3.8.2.5
pub fn prop_duration<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], DurationProperty<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (_, (other_params, _, value, _))) = tuple((
        tag_no_case("DURATION"),
        cut(tuple((
            other_params,
            char(':'),
            prop_value_duration,
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((
        input,
        DurationProperty {
            other_params,
            value,
        },
    ))
}

#[derive(Debug, Eq, PartialEq)]
pub struct FreeBusyTimeProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: Vec<Period>,
}

/// Parse a FREEBUSY property.
///
/// RFC 5545, section 3.8.2.6
pub fn prop_free_busy_time<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], FreeBusyTimeProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, (params, _, value, _))) = tuple((
        tag_no_case("FREEBUSY"),
        cut(tuple((
            params,
            char(':'),
            separated_list1(char(','), prop_value_period),
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((input, FreeBusyTimeProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct TimeTransparencyProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: TimeTransparency,
}

/// Parse a TRANSP property.
///
/// RFC 5545, section 3.8.2.7
pub fn prop_time_transparency<'a, E>(
    input: &'a [u8],
) -> IResult<&'a [u8], TimeTransparencyProperty<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (_, (other_params, _, value, _))) = tuple((
        tag_no_case("TRANSP"),
        cut(tuple((
            other_params,
            char(':'),
            alt((
                tag_no_case("OPAQUE").map(|_| TimeTransparency::Opaque),
                tag_no_case("TRANSPARENT").map(|_| TimeTransparency::Transparent),
            )),
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((
        input,
        TimeTransparencyProperty {
            other_params,
            value,
        },
    ))
}

#[derive(Debug, Eq, PartialEq)]
pub struct TimeZoneIdProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub unique_registry_id: bool,
    pub value: Vec<u8>,
}

/// Parse a TZID property.
///
/// RFC 5545, section 3.8.3.1
pub fn prop_time_zone_id<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], TimeZoneIdProperty<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (_, (other_params, _, (unique, value), _))) = tuple((
        tag_no_case("TZID"),
        cut(tuple((
            other_params,
            char(':'),
            tuple((opt(char('/')), prop_value_text)),
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((
        input,
        TimeZoneIdProperty {
            other_params,
            unique_registry_id: unique.is_some(),
            value,
        },
    ))
}

#[derive(Debug, Eq, PartialEq)]
pub struct TimeZoneNameProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: Vec<u8>,
}

/// Parse a TZNAME property.
///
/// RFC 5545, section 3.8.3.2
pub fn prop_time_zone_name<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], TimeZoneNameProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, (params, _, value, _))) = tuple((
        tag_no_case("TZNAME"),
        cut(tuple((params, char(':'), prop_value_text, tag("\r\n")))),
    ))(input)?;

    Ok((input, TimeZoneNameProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct TimeZoneOffsetProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: UtcOffset,
}

/// Parse a TZOFFSETFROM property.
///
/// RFC 5545, section 3.8.3.3
pub fn prop_time_zone_offset_from<'a, E>(
    input: &'a [u8],
) -> IResult<&'a [u8], TimeZoneOffsetProperty<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (_, (other_params, _, value, _))) = tuple((
        tag_no_case("TZOFFSETFROM"),
        cut(tuple((
            other_params,
            char(':'),
            prop_value_utc_offset,
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((
        input,
        TimeZoneOffsetProperty {
            other_params,
            value,
        },
    ))
}

/// Parse a TZOFFSETTO property.
///
/// RFC 5545, section 3.8.3.4
pub fn prop_time_zone_offset_to<'a, E>(
    input: &'a [u8],
) -> IResult<&'a [u8], TimeZoneOffsetProperty<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (_, (other_params, _, value, _))) = tuple((
        tag_no_case("TZOFFSETTO"),
        cut(tuple((
            other_params,
            char(':'),
            prop_value_utc_offset,
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((
        input,
        TimeZoneOffsetProperty {
            other_params,
            value,
        },
    ))
}

#[derive(Debug, Eq, PartialEq)]
pub struct TimeZoneUrlProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: &'a [u8],
}

/// Parse a TZURL property.
///
/// RFC 5545, section 3.8.3.5
pub fn prop_time_zone_url<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], TimeZoneUrlProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, (other_params, _, value, _))) = tuple((
        tag_no_case("TZURL"),
        cut(tuple((
            other_params,
            char(':'),
            cut(recognize(param_value_uri)),
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((
        input,
        TimeZoneUrlProperty {
            other_params,
            value,
        },
    ))
}

#[derive(Debug, Eq, PartialEq)]
pub struct AttendeeProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: &'a [u8],
}

/// Parse an ATTENDEE property.
///
/// RFC 5545, section 3.8.4.1
pub fn prop_attendee<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], AttendeeProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, (params, _, uri, _))) = tuple((
        tag_no_case("ATTENDEE"),
        cut(tuple((
            params,
            char(':'),
            cut(recognize(prop_value_calendar_user_address)),
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((input, AttendeeProperty { params, value: uri }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct ContactProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: Vec<u8>,
}

/// Parse a CONTACT property.
///
/// RFC 5545, section 3.8.4.2
pub fn prop_contact<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ContactProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, (params, _, value, _))) = tuple((
        tag_no_case("CONTACT"),
        cut(tuple((params, char(':'), prop_value_text, tag("\r\n")))),
    ))(input)?;

    Ok((input, ContactProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizerProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: &'a [u8],
}

/// Parse an ORGANIZER property.
///
/// RFC 5545, section 3.8.4.3
pub fn prop_organizer<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], OrganizerProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, (params, _, uri, _))) = tuple((
        tag_no_case("ORGANIZER"),
        cut(tuple((
            params,
            char(':'),
            recognize(prop_value_calendar_user_address),
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((input, OrganizerProperty { params, value: uri }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct RecurrenceIdProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: DateOrDateTime,
}

/// Parse a RECURRENCE-ID property.
///
/// RFC 5545, section 3.8.4.4
pub fn prop_recurrence_id<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], RecurrenceIdProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, (params, _, value, _))) = tuple((
        tag_no_case("RECURRENCE-ID"),
        cut(tuple((
            params,
            char(':'),
            alt((
                prop_value_date_time.map(DateOrDateTime::DateTime),
                prop_value_date.map(DateOrDateTime::Date),
            )),
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((input, RecurrenceIdProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct RelatedToProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: Vec<u8>,
}

/// Parse a RELATED-TO property.
///
/// RFC 5545, section 3.8.4.5
pub fn prop_related_to<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], RelatedToProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, (params, _, value, _))) = tuple((
        tag_no_case("RELATED-TO"),
        cut(tuple((params, char(':'), prop_value_text, tag("\r\n")))),
    ))(input)?;

    Ok((input, RelatedToProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct UrlProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: Uri<'a>,
}

/// Parse a URL property.
///
/// RFC 5545, section 3.8.4.6
pub fn prop_url<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], UrlProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, (other_params, _, value, _))) = tuple((
        tag_no_case("URL"),
        cut(tuple((
            other_params,
            char(':'),
            param_value_uri,
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((
        input,
        UrlProperty {
            other_params,
            value,
        },
    ))
}

#[derive(Debug, Eq, PartialEq)]
pub struct UniqueIdentifierProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: Vec<u8>,
}

/// Parse a UID property.
///
/// RFC 5545, section 3.8.4.7
pub fn prop_unique_identifier<'a, E>(
    input: &'a [u8],
) -> IResult<&'a [u8], UniqueIdentifierProperty<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (_, (other_params, _, value, _))) = tuple((
        tag_no_case("UID"),
        cut(tuple((
            other_params,
            char(':'),
            prop_value_text,
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((
        input,
        UniqueIdentifierProperty {
            other_params,
            value,
        },
    ))
}

#[derive(Debug, Eq, PartialEq)]
pub struct ExceptionDateTimesProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: Vec<DateOrDateTime>,
}

/// Parse an EXDATE property.
///
/// RFC 5545, section 3.8.5.1
pub fn prop_exception_date_times<'a, E>(
    input: &'a [u8],
) -> IResult<&'a [u8], ExceptionDateTimesProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, (params, _, value, _))) = tuple((
        tag_no_case("EXDATE"),
        cut(tuple((
            params,
            char(':'),
            separated_list1(
                char(','),
                alt((
                    prop_value_date_time.map(DateOrDateTime::DateTime),
                    prop_value_date.map(DateOrDateTime::Date),
                )),
            ),
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((input, ExceptionDateTimesProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct RecurrenceDateTimesProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: Vec<DateOrDateTimeOrPeriod>,
}

/// Parse an RDATE property.
///
/// RFC 5545, section 3.8.5.2
pub fn prop_recurrence_date_times<'a, E>(
    input: &'a [u8],
) -> IResult<&'a [u8], RecurrenceDateTimesProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, (params, _, value, _))) = tuple((
        tag_no_case("RDATE"),
        cut(tuple((
            params,
            char(':'),
            separated_list1(
                char(','),
                alt((
                    prop_value_period.map(DateOrDateTimeOrPeriod::Period),
                    prop_value_date_time.map(DateOrDateTimeOrPeriod::DateTime),
                    prop_value_date.map(DateOrDateTimeOrPeriod::Date),
                )),
            ),
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((input, RecurrenceDateTimesProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct RecurrenceRuleProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: Vec<RecurRulePart>,
}

/// Parse an RRULE property.
///
/// RFC 5545, section 3.8.5.3
pub fn prop_recurrence_rule<'a, E>(
    input: &'a [u8],
) -> IResult<&'a [u8], RecurrenceRuleProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, (other_params, _, value, _))) = tuple((
        tag_no_case("RRULE"),
        cut(tuple((other_params, char(':'), recur, tag("\r\n")))),
    ))(input)?;

    Ok((
        input,
        RecurrenceRuleProperty {
            other_params,
            value,
        },
    ))
}

#[derive(Debug, Eq, PartialEq)]
pub enum Action<'a> {
    Audio,
    Display,
    Email,
    XName(&'a [u8]),
    IanaToken(&'a [u8]),
}

#[derive(Debug, Eq, PartialEq)]
pub struct ActionProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: Action<'a>,
}

/// Parse an ACTION property.
///
/// RFC 5545, section 3.8.6.1
pub fn prop_action<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ActionProperty<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (_, (other_params, _, value, _))) = tuple((
        tag_no_case("ACTION"),
        cut(tuple((
            other_params,
            char(':'),
            alt((
                tag_no_case("AUDIO").map(|_| Action::Audio),
                tag_no_case("DISPLAY").map(|_| Action::Display),
                tag_no_case("EMAIL").map(|_| Action::Email),
                x_name.map(Action::XName),
                iana_token.map(Action::IanaToken),
            )),
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((
        input,
        ActionProperty {
            other_params,
            value,
        },
    ))
}

#[derive(Debug, Eq, PartialEq)]
pub struct RepeatCountProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: u32,
}

/// Parse a REPEAT property.
///
/// RFC 5545, section 3.8.6.2
pub fn prop_repeat_count<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], RepeatCountProperty<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (_, (other_params, _, value, _))) = tuple((
        tag_no_case("REPEAT"),
        cut(tuple((
            other_params,
            char(':'),
            prop_value_integer.map(|v| v as u32),
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((
        input,
        RepeatCountProperty {
            other_params,
            value,
        },
    ))
}

#[derive(Debug, Eq, PartialEq)]
pub enum DurationOrDateTime {
    Duration(Duration),
    DateTime(DateTime),
}

#[derive(Debug, Eq, PartialEq)]
pub struct TriggerProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub value: DurationOrDateTime,
}

/// Parse a TRIGGER property.
///
/// RFC 5545, section 3.8.6.3
pub fn prop_trigger<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], TriggerProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, (params, _))) =
        tuple((tag_no_case("TRIGGER"), cut(tuple((params, char(':'))))))(input)?;

    let value_choice = params
        .iter()
        .filter_map(|p| match p {
            ParamValue::ValueType {
                value: Value::Duration,
            } => Some(1),
            ParamValue::ValueType {
                value: Value::DateTime,
            } => Some(2),
            _ => None,
        })
        .collect::<Vec<_>>();

    let (input, value) = match value_choice.as_slice() {
        [1] | [] => cut(prop_value_duration)
            .map(DurationOrDateTime::Duration)
            .parse(input),
        [2] => cut(prop_value_date_time)
            .map(DurationOrDateTime::DateTime)
            .parse(input),
        _ => {
            return Err(nom::Err::Error(
                Error::new(input, InnerError::InvalidValueParam).into(),
            ))
        }
    }?;

    let (input, _) = cut(tag("\r\n"))(input)?;

    Ok((input, TriggerProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct CreatedProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: DateTime,
}

/// Parse a CREATED property.
///
/// RFC 5545, section 3.8.7.1
pub fn prop_created<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], CreatedProperty<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (_, (other_params, _, value, _))) = tuple((
        tag_no_case("CREATED"),
        cut(tuple((
            other_params,
            char(':'),
            prop_value_date_time,
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((
        input,
        CreatedProperty {
            other_params,
            value,
        },
    ))
}

#[derive(Debug, Eq, PartialEq)]
pub struct DateTimeStampProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: DateTime,
}

/// Parse a DTSTAMP property.
///
/// RFC 5545, section 3.8.7.2
pub fn prop_date_time_stamp<'a, E>(
    input: &'a [u8],
) -> IResult<&'a [u8], DateTimeStampProperty<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (_, (other_params, _, value, _))) = tuple((
        tag_no_case("DTSTAMP"),
        cut(tuple((
            other_params,
            char(':'),
            prop_value_date_time,
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((
        input,
        DateTimeStampProperty {
            other_params,
            value,
        },
    ))
}

#[derive(Debug, Eq, PartialEq)]
pub struct LastModifiedProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: DateTime,
}

/// Parse a LAST-MODIFIED property.
///
/// RFC 5545, section 3.8.7.3
pub fn prop_last_modified<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], LastModifiedProperty<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (_, (other_params, _, value, _))) = tuple((
        tag_no_case("LAST-MODIFIED"),
        cut(tuple((
            other_params,
            char(':'),
            prop_value_date_time,
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((
        input,
        LastModifiedProperty {
            other_params,
            value,
        },
    ))
}

#[derive(Debug, Eq, PartialEq)]
pub struct SequenceProperty<'a> {
    pub other_params: Vec<ParamValue<'a>>,
    pub value: u32,
}

/// Parse a SEQUENCE property.
///
/// RFC 5545, section 3.8.7.4
pub fn prop_sequence<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], SequenceProperty<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (_, (other_params, _, value, _))) = tuple((
        tag_no_case("SEQUENCE"),
        cut(tuple((
            other_params,
            char(':'),
            prop_value_integer.map(|v| v as u32),
            tag("\r\n"),
        ))),
    ))(input)?;

    Ok((
        input,
        SequenceProperty {
            other_params,
            value,
        },
    ))
}

#[derive(Debug, Eq, PartialEq)]
pub struct RequestStatusProperty<'a> {
    pub params: Vec<ParamValue<'a>>,
    pub status_code: Vec<u32>,
    pub status_description: Vec<u8>,
    pub exception_data: Option<Vec<u8>>,
}

/// Parse a REQUEST-STATUS property.
///
/// RFC 5545, section 3.8.8.3
pub fn prop_request_status<'a, E>(
    input: &'a [u8],
) -> IResult<&'a [u8], RequestStatusProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    fn status_code<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Vec<u32>, E>
    where
        E: ParseError<&'a [u8]>
            + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
            + From<Error<'a>>,
    {
        let (input, (num, mut nums)) = tuple((
            map_res(
                verify(take_while1(is_digit), |v: &[u8]| v.len() == 1),
                |v| read_int::<E, u32>(v),
            ),
            fold_many_m_n(
                1,
                2,
                map_res(tuple((char('.'), take_while1(is_digit))), |(_, v)| {
                    read_int::<E, u32>(v)
                }),
                Vec::new,
                |mut acc, item| {
                    acc.push(item);
                    acc
                },
            ),
        ))(input)?;

        nums.insert(0, num);
        Ok((input, nums))
    }

    let (input, (_, (params, _, status_code, _, status_description, extra_data, _))) =
        tuple((
            tag_no_case("REQUEST-STATUS"),
            cut(tuple((
                params,
                char(':'),
                status_code,
                char(';'),
                prop_value_text,
                opt(tuple((char(';'), prop_value_text)).map(|(_, v)| v)),
                tag("\r\n"),
            ))),
        ))(input)?;

    Ok((
        input,
        RequestStatusProperty {
            params,
            status_code,
            status_description,
            exception_data: extra_data,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::FreeBusyTimeType;
    use crate::common::RecurFreq;
    use crate::common::{LanguageTag, ParticipationStatusUnknown, Range, Related, Role, Value};
    use crate::parser::param::ParamValue;
    use crate::parser::property::uri::{Authority, Host};
    use crate::parser::property::{Date, Period, PeriodEnd, Time};
    use crate::test_utils::check_rem;
    use base64::Engine;

    #[test]
    fn attach_uri() {
        let (rem, prop) =
            prop_attach::<Error>(b"ATTACH:CID:jsmith.part3.960817T083000.xyzMail@example.com\r\n;")
                .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            AttachProperty {
                params: vec![],
                value: AttachValue::Uri(b"CID:jsmith.part3.960817T083000.xyzMail@example.com"),
            }
        );
    }

    #[test]
    fn attach_binary() {
        let (rem, prop) =
            prop_attach::<Error>(b"ATTACH;VALUE=BINARY;ENCODING=BASE64:dGVzdA==\r\n;").unwrap();
        check_rem(rem, 1);

        let r = base64::prelude::BASE64_STANDARD.encode("test");

        assert_eq!(
            prop,
            AttachProperty {
                params: vec![
                    ParamValue::ValueType {
                        value: Value::Binary
                    },
                    ParamValue::Encoding {
                        encoding: Encoding::Base64,
                    },
                ],
                value: AttachValue::Binary(r.as_bytes()),
            }
        );
    }

    #[test]
    fn categories() {
        let (rem, prop) =
            prop_categories::<Error>(b"CATEGORIES:APPOINTMENT,EDUCATION\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            CategoriesProperty {
                params: vec![],
                value: vec![b"APPOINTMENT".to_vec(), b"EDUCATION".to_vec()],
            }
        );
    }

    #[test]
    fn classification_public() {
        let (rem, prop) = prop_classification::<Error>(b"CLASS:PUBLIC\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            ClassificationProperty {
                other_params: vec![],
                value: Classification::Public,
            }
        );
    }

    #[test]
    fn comment() {
        let (rem, prop) = prop_comment::<Error>(b"COMMENT:The meeting really needs to include both ourselves and the customer. We can't hold this meeting without them. As a matter of fact\\, the venue for the meeting ought to be at their site. - - John\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            CommentProperty {
                params: vec![],
                value: b"The meeting really needs to include both ourselves and the customer. We can't hold this meeting without them. As a matter of fact, the venue for the meeting ought to be at their site. - - John".to_vec(),
            }
        );
    }

    #[test]
    fn description() {
        let (rem, prop) = prop_description::<Error>(b"DESCRIPTION:Meeting to provide technical review for \"Phoenix\"\r\n  design.\\nHappy Face Conference Room. Phoenix design team\r\n  MUST attend this meeting.\\nRSVP to team leader.\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            DescriptionProperty {
                params: vec![],
                value: br#"Meeting to provide technical review for "Phoenix" design.
Happy Face Conference Room. Phoenix design team MUST attend this meeting.
RSVP to team leader."#
                    .to_vec(),
            }
        );
    }

    #[test]
    fn geographic_position() {
        let (rem, prop) =
            prop_geographic_position::<Error>(b"GEO:37.386013;-122.082932\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            GeographicPositionProperty {
                other_params: vec![],
                latitude: 37.386013,
                longitude: -122.082932,
            }
        );
    }

    #[test]
    fn location() {
        let (rem, prop) =
            prop_location::<Error>(b"LOCATION:Conference Room - F123\\, Bldg. 002\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            LocationProperty {
                params: vec![],
                value: b"Conference Room - F123, Bldg. 002".to_vec(),
            }
        );
    }

    #[test]
    fn location_with_params() {
        let (rem, prop) = prop_location::<Error>(b"LOCATION;ALTREP=\"http://xyzcorp.com/conf-rooms/f123.vcf\":\r\n Conference Room - F123\\, Bldg. 002\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            LocationProperty {
                params: vec![ParamValue::AltRep {
                    uri: b"http://xyzcorp.com/conf-rooms/f123.vcf",
                },],
                value: b"Conference Room - F123, Bldg. 002".to_vec(),
            }
        );
    }

    #[test]
    fn percent_complete() {
        let (rem, prop) = prop_percent_complete::<Error>(b"PERCENT-COMPLETE:39\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            PercentCompleteProperty {
                other_params: vec![],
                value: 39,
            }
        );
    }

    #[test]
    fn priority() {
        let (rem, prop) = prop_priority::<Error>(b"PRIORITY:1\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            PriorityProperty {
                other_params: vec![],
                value: 1,
            }
        );
    }

    #[test]
    fn resources() {
        let (rem, prop) = prop_resources::<Error>(b"RESOURCES:EASEL,PROJECTOR,VCR\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            ResourcesProperty {
                params: vec![],
                value: vec![b"EASEL".to_vec(), b"PROJECTOR".to_vec(), b"VCR".to_vec()],
            }
        );
    }

    #[test]
    fn status() {
        let (rem, prop) = prop_status::<Error>(b"STATUS:TENTATIVE\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            StatusProperty {
                other_params: vec![],
                value: Status::Tentative,
            }
        );
    }

    #[test]
    fn summary() {
        let (rem, prop) = prop_summary::<Error>(b"SUMMARY:Department Party\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            SummaryProperty {
                params: vec![],
                value: b"Department Party".to_vec(),
            }
        );
    }

    #[test]
    fn date_time_completed() {
        let (rem, prop) =
            prop_date_time_completed::<Error>(b"COMPLETED:19960401T150000Z\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            DateTimeCompletedProperty {
                other_params: vec![],
                value: DateTime {
                    date: Date {
                        year: 1996,
                        month: 4,
                        day: 1,
                    },
                    time: Time {
                        hour: 15,
                        minute: 0,
                        second: 0,
                        is_utc: true,
                    },
                },
            }
        );
    }

    #[test]
    fn date_time_end_date() {
        let (rem, prop) = prop_date_time_end::<Error>(b"DTEND;VALUE=DATE:19980704\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            DateTimeEndProperty {
                params: vec![ParamValue::ValueType { value: Value::Date },],
                value: DateOrDateTime::Date(Date {
                    year: 1998,
                    month: 7,
                    day: 4,
                }),
            }
        );
    }

    #[test]
    fn date_time_end_datetime() {
        let (rem, prop) = prop_date_time_end::<Error>(b"DTEND:19960401T150000Z\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            DateTimeEndProperty {
                params: vec![],
                value: DateOrDateTime::DateTime(DateTime {
                    date: Date {
                        year: 1996,
                        month: 4,
                        day: 1,
                    },
                    time: Time {
                        hour: 15,
                        minute: 0,
                        second: 0,
                        is_utc: true,
                    },
                }),
            }
        );
    }

    #[test]
    fn date_time_due_date() {
        let (rem, prop) = prop_date_time_due::<Error>(b"DUE;VALUE=DATE:19980401\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            DateTimeDueProperty {
                params: vec![ParamValue::ValueType { value: Value::Date },],
                value: DateOrDateTime::Date(Date {
                    year: 1998,
                    month: 4,
                    day: 1,
                }),
            }
        );
    }

    #[test]
    fn date_time_due_datetime() {
        let (rem, prop) = prop_date_time_due::<Error>(b"DUE:19980430T000000Z\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            DateTimeDueProperty {
                params: vec![],
                value: DateOrDateTime::DateTime(DateTime {
                    date: Date {
                        year: 1998,
                        month: 4,
                        day: 30,
                    },
                    time: Time {
                        hour: 0,
                        minute: 0,
                        second: 0,
                        is_utc: true,
                    },
                }),
            }
        );
    }

    #[test]
    fn date_time_start_date() {
        let (rem, prop) = prop_date_time_start::<Error>(b"DTSTART:19980118\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            DateTimeStartProperty {
                params: vec![],
                value: DateOrDateTime::Date(Date {
                    year: 1998,
                    month: 1,
                    day: 18,
                }),
            }
        );
    }

    #[test]
    fn date_time_start_datetime() {
        let (rem, prop) = prop_date_time_start::<Error>(b"DTSTART:19980118T073000Z\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            DateTimeStartProperty {
                params: vec![],
                value: DateOrDateTime::DateTime(DateTime {
                    date: Date {
                        year: 1998,
                        month: 1,
                        day: 18,
                    },
                    time: Time {
                        hour: 7,
                        minute: 30,
                        second: 0,
                        is_utc: true,
                    },
                }),
            }
        );
    }

    #[test]
    fn duration() {
        let (rem, prop) = prop_duration::<Error>(b"DURATION:PT1H0M0S\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            DurationProperty {
                other_params: vec![],
                value: Duration {
                    sign: 1,
                    hours: Some(1),
                    minutes: Some(0),
                    seconds: Some(0),
                    ..Default::default()
                },
            }
        );
    }

    #[test]
    fn free_busy() {
        let (rem, prop) = prop_free_busy_time::<Error>(
            b"FREEBUSY;FBTYPE=BUSY-UNAVAILABLE:19970308T160000Z/PT8H30M\r\n;",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            FreeBusyTimeProperty {
                params: vec![ParamValue::FreeBusyTimeType {
                    fb_type: FreeBusyTimeType::BusyUnavailable,
                },],
                value: vec![Period {
                    start: DateTime {
                        date: Date {
                            year: 1997,
                            month: 3,
                            day: 8,
                        },
                        time: Time {
                            hour: 16,
                            minute: 0,
                            second: 0,
                            is_utc: true,
                        },
                    },
                    end: PeriodEnd::Duration(Duration {
                        sign: 1,
                        hours: Some(8),
                        minutes: Some(30),
                        ..Default::default()
                    }),
                }],
            }
        );
    }

    #[test]
    fn free_busy_multiple() {
        let (rem, prop) = prop_free_busy_time::<Error>(
            b"FREEBUSY;FBTYPE=FREE:19970308T160000Z/PT3H,19970308T200000Z/PT1H\r\n;",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            FreeBusyTimeProperty {
                params: vec![ParamValue::FreeBusyTimeType {
                    fb_type: FreeBusyTimeType::Free,
                },],
                value: vec![
                    Period {
                        start: DateTime {
                            date: Date {
                                year: 1997,
                                month: 3,
                                day: 8,
                            },
                            time: Time {
                                hour: 16,
                                minute: 0,
                                second: 0,
                                is_utc: true,
                            },
                        },
                        end: PeriodEnd::Duration(Duration {
                            sign: 1,
                            hours: Some(3),
                            ..Default::default()
                        }),
                    },
                    Period {
                        start: DateTime {
                            date: Date {
                                year: 1997,
                                month: 3,
                                day: 8,
                            },
                            time: Time {
                                hour: 20,
                                minute: 0,
                                second: 0,
                                is_utc: true,
                            },
                        },
                        end: PeriodEnd::Duration(Duration {
                            sign: 1,
                            hours: Some(1),
                            ..Default::default()
                        }),
                    },
                ],
            }
        );
    }

    #[test]
    fn transp_opaque() {
        let (rem, prop) = prop_time_transparency::<Error>(b"TRANSP:OPAQUE\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            TimeTransparencyProperty {
                other_params: vec![],
                value: TimeTransparency::Opaque,
            }
        );
    }

    #[test]
    fn transp_transparent() {
        let (rem, prop) = prop_time_transparency::<Error>(b"TRANSP:TRANSPARENT\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            TimeTransparencyProperty {
                other_params: vec![],
                value: TimeTransparency::Transparent,
            }
        );
    }

    #[test]
    fn time_zone_id() {
        let (rem, prop) = prop_time_zone_id::<Error>(b"TZID:America/New_York\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            TimeZoneIdProperty {
                other_params: vec![],
                unique_registry_id: false,
                value: b"America/New_York".to_vec(),
            }
        );
    }

    #[test]
    fn time_zone_id_custom() {
        let (rem, prop) =
            prop_time_zone_id::<Error>(b"TZID:/example.org/America/New_York\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            TimeZoneIdProperty {
                other_params: vec![],
                unique_registry_id: true,
                value: b"example.org/America/New_York".to_vec(),
            }
        );
    }

    #[test]
    fn time_zone_name() {
        let (rem, prop) = prop_time_zone_name::<Error>(b"TZNAME:EST\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            TimeZoneNameProperty {
                params: vec![],
                value: b"EST".to_vec(),
            }
        );
    }

    #[test]
    fn time_zone_name_with_params() {
        let (rem, prop) = prop_time_zone_name::<Error>(b"TZNAME;LANGUAGE=fr-CA:HNE\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            TimeZoneNameProperty {
                params: vec![ParamValue::Language {
                    language: LanguageTag {
                        language: "fr".to_string(),
                        region: Some("CA".to_string()),
                        ..Default::default()
                    },
                },],
                value: b"HNE".to_vec(),
            }
        );
    }

    #[test]
    fn time_zone_offset_from() {
        let (rem, prop) = prop_time_zone_offset_from::<Error>(b"TZOFFSETFROM:-0500\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            TimeZoneOffsetProperty {
                other_params: vec![],
                value: UtcOffset {
                    sign: -1,
                    hours: 5,
                    minutes: 0,
                    seconds: None,
                },
            }
        );
    }

    #[test]
    fn time_zone_offset_to() {
        let (rem, prop) = prop_time_zone_offset_to::<Error>(b"TZOFFSETTO:+1245\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            TimeZoneOffsetProperty {
                other_params: vec![],
                value: UtcOffset {
                    sign: 1,
                    hours: 12,
                    minutes: 45,
                    seconds: None,
                },
            }
        );
    }

    #[test]
    fn time_zone_url() {
        let (rem, prop) = prop_time_zone_url::<Error>(
            b"TZURL:http://timezones.example.org/tz/America-Los_Angeles.ics\r\n;",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            TimeZoneUrlProperty {
                other_params: vec![],
                value: b"http://timezones.example.org/tz/America-Los_Angeles.ics",
            }
        );
    }

    #[test]
    fn attendee() {
        let (rem, prop) = prop_attendee::<Error>(b"ATTENDEE;ROLE=REQ-PARTICIPANT;DELEGATED-FROM=\"mailto:bob@example.com\";PARTSTAT=ACCEPTED;CN=Jane Doe:mailto:jdoe@example.com\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            AttendeeProperty {
                params: vec![
                    ParamValue::Role {
                        role: Role::RequiredParticipant,
                    },
                    ParamValue::DelegatedFrom {
                        delegators: vec![b"mailto:bob@example.com"],
                    },
                    ParamValue::ParticipationStatus {
                        status: ParticipationStatusUnknown::Accepted,
                    },
                    ParamValue::CommonName {
                        name: "Jane Doe".to_string(),
                    },
                ],
                value: b"mailto:jdoe@example.com",
            }
        );
    }

    #[test]
    fn contact() {
        let (rem, prop) = prop_contact::<Error>(
            b"CONTACT:Jim Dolittle\\, ABC Industries\\, +1-919-555-1234\r\n;",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            ContactProperty {
                params: vec![],
                value: b"Jim Dolittle, ABC Industries, +1-919-555-1234".to_vec(),
            }
        );
    }

    #[test]
    fn contact_altrep() {
        let (rem, prop) = prop_contact::<Error>(b"CONTACT;ALTREP=\"ldap://example.com:6666/o=ABC%20Industries,c=US???(cn=Jim%20Dolittle)\":Jim Dolittle\\, ABC Industries\\, +1-919-555-1234\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            ContactProperty {
                params: vec![ParamValue::AltRep {
                    uri: b"ldap://example.com:6666/o=ABC%20Industries,c=US???(cn=Jim%20Dolittle)",
                },],
                value: b"Jim Dolittle, ABC Industries, +1-919-555-1234".to_vec(),
            }
        );
    }

    #[test]
    fn organizer() {
        let (rem, prop) =
            prop_organizer::<Error>(b"ORGANIZER;CN=John Smith:mailto:jsmith@example.com\r\n;")
                .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            OrganizerProperty {
                params: vec![ParamValue::CommonName {
                    name: "John Smith".to_string(),
                },],
                value: b"mailto:jsmith@example.com",
            }
        );
    }

    #[test]
    fn organizer_with_params() {
        let (rem, prop) = prop_organizer::<Error>(b"ORGANIZER;CN=JohnSmith;DIR=\"ldap://example.com:6666/o=DC%20Associates,c=US???(cn=John%20Smith)\":mailto:jsmith@example.com\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            OrganizerProperty {
                params: vec![
                    ParamValue::CommonName {
                        name: "JohnSmith".to_string(),
                    },
                    ParamValue::DirectoryEntryReference {
                        uri: b"ldap://example.com:6666/o=DC%20Associates,c=US???(cn=John%20Smith)",
                    },
                ],
                value: b"mailto:jsmith@example.com",
            }
        );
    }

    #[test]
    fn organizer_with_sent_by_param() {
        let (rem, prop) = prop_organizer::<Error>(
            b"ORGANIZER;SENT-BY=\"mailto:jane_doe@example.com\":mailto:jsmith@example.com\r\n;",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            OrganizerProperty {
                params: vec![ParamValue::SentBy {
                    address: b"mailto:jane_doe@example.com",
                },],
                value: b"mailto:jsmith@example.com",
            }
        );
    }

    #[test]
    fn recurrence_id_date() {
        let (rem, prop) =
            prop_recurrence_id::<Error>(b"RECURRENCE-ID;VALUE=DATE:19960401\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            RecurrenceIdProperty {
                params: vec![ParamValue::ValueType { value: Value::Date },],
                value: DateOrDateTime::Date(Date {
                    year: 1996,
                    month: 4,
                    day: 1,
                }),
            }
        );
    }

    #[test]
    fn recurrence_id_datetime() {
        let (rem, prop) =
            prop_recurrence_id::<Error>(b"RECURRENCE-ID;RANGE=THISANDFUTURE:19960120T120000Z\r\n;")
                .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            RecurrenceIdProperty {
                params: vec![ParamValue::Range {
                    range: Range::ThisAndFuture,
                },],
                value: DateOrDateTime::DateTime(DateTime {
                    date: Date {
                        year: 1996,
                        month: 1,
                        day: 20,
                    },
                    time: Time {
                        hour: 12,
                        minute: 0,
                        second: 0,
                        is_utc: true,
                    },
                }),
            }
        );
    }

    #[test]
    fn related_to() {
        let (rem, prop) = prop_related_to::<Error>(
            b"RELATED-TO:jsmith.part7.19960817T083000.xyzMail@example.com\r\n;",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            RelatedToProperty {
                params: vec![],
                value: b"jsmith.part7.19960817T083000.xyzMail@example.com".to_vec(),
            }
        );
    }

    #[test]
    fn url() {
        let (rem, prop) =
            prop_url::<Error>(b"URL:http://example.com/pub/calendars/jsmith/mytime.ics\r\n;")
                .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            UrlProperty {
                other_params: vec![],
                value: Uri {
                    scheme: b"http",
                    authority: Some(Authority {
                        user_info: None,
                        host: Host::RegName(b"example.com".to_vec()),
                        port: None,
                    }),
                    path: b"/pub/calendars/jsmith/mytime.ics".to_vec(),
                    query: None,
                    fragment: None,
                },
            }
        );
    }

    #[test]
    fn unique_identifier() {
        let (rem, prop) = prop_unique_identifier::<Error>(
            b"UID:19960401T080045Z-4000F192713-0052@example.com\r\n;",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            UniqueIdentifierProperty {
                other_params: vec![],
                value: b"19960401T080045Z-4000F192713-0052@example.com".to_vec(),
            }
        );
    }

    #[test]
    fn exception_date_times() {
        let (rem, prop) = prop_exception_date_times::<Error>(
            b"EXDATE:19960402T010000Z,19960403T010000Z,19960404T010000Z\r\n;",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            ExceptionDateTimesProperty {
                params: vec![],
                value: vec![
                    DateOrDateTime::DateTime(DateTime {
                        date: Date {
                            year: 1996,
                            month: 4,
                            day: 2,
                        },
                        time: Time {
                            hour: 1,
                            minute: 0,
                            second: 0,
                            is_utc: true,
                        },
                    }),
                    DateOrDateTime::DateTime(DateTime {
                        date: Date {
                            year: 1996,
                            month: 4,
                            day: 3,
                        },
                        time: Time {
                            hour: 1,
                            minute: 0,
                            second: 0,
                            is_utc: true,
                        },
                    }),
                    DateOrDateTime::DateTime(DateTime {
                        date: Date {
                            year: 1996,
                            month: 4,
                            day: 4,
                        },
                        time: Time {
                            hour: 1,
                            minute: 0,
                            second: 0,
                            is_utc: true,
                        },
                    }),
                ],
            }
        );
    }

    #[test]
    fn recurrence_date_times_datetime() {
        let (rem, prop) = prop_recurrence_date_times::<Error>(
            b"RDATE;TZID=America/New_York:19970714T083000\r\n;",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            RecurrenceDateTimesProperty {
                params: vec![ParamValue::TimeZoneId {
                    tz_id: "America/New_York".to_string(),
                    unique: false,
                },],
                value: vec![DateOrDateTimeOrPeriod::DateTime(DateTime {
                    date: Date {
                        year: 1997,
                        month: 7,
                        day: 14,
                    },
                    time: Time {
                        hour: 8,
                        minute: 30,
                        second: 0,
                        is_utc: false,
                    },
                }),],
            }
        );
    }

    #[test]
    fn recurrence_date_times_periods() {
        let (rem, prop) = prop_recurrence_date_times::<Error>(
            b"RDATE;VALUE=PERIOD:19960403T020000Z/19960403T040000Z,19960404T010000Z/PT3H\r\n;",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            RecurrenceDateTimesProperty {
                params: vec![ParamValue::ValueType {
                    value: Value::Period
                },],
                value: vec![
                    DateOrDateTimeOrPeriod::Period(Period {
                        start: DateTime {
                            date: Date {
                                year: 1996,
                                month: 4,
                                day: 3,
                            },
                            time: Time {
                                hour: 2,
                                minute: 0,
                                second: 0,
                                is_utc: true,
                            },
                        },
                        end: PeriodEnd::DateTime(DateTime {
                            date: Date {
                                year: 1996,
                                month: 4,
                                day: 3,
                            },
                            time: Time {
                                hour: 4,
                                minute: 0,
                                second: 0,
                                is_utc: true,
                            },
                        }),
                    }),
                    DateOrDateTimeOrPeriod::Period(Period {
                        start: DateTime {
                            date: Date {
                                year: 1996,
                                month: 4,
                                day: 4,
                            },
                            time: Time {
                                hour: 1,
                                minute: 0,
                                second: 0,
                                is_utc: true,
                            },
                        },
                        end: PeriodEnd::Duration(Duration {
                            sign: 1,
                            hours: Some(3),
                            ..Default::default()
                        }),
                    }),
                ],
            }
        );
    }

    #[test]
    fn recurrence_date_times_dates() {
        let (rem, prop) = prop_recurrence_date_times::<Error>(
            b"RDATE;VALUE=DATE:19970101,19970120,19970217,19970421\r\n;",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            RecurrenceDateTimesProperty {
                params: vec![ParamValue::ValueType { value: Value::Date },],
                value: vec![
                    DateOrDateTimeOrPeriod::Date(Date {
                        year: 1997,
                        month: 1,
                        day: 1,
                    }),
                    DateOrDateTimeOrPeriod::Date(Date {
                        year: 1997,
                        month: 1,
                        day: 20,
                    }),
                    DateOrDateTimeOrPeriod::Date(Date {
                        year: 1997,
                        month: 2,
                        day: 17,
                    }),
                    DateOrDateTimeOrPeriod::Date(Date {
                        year: 1997,
                        month: 4,
                        day: 21,
                    }),
                ],
            }
        );
    }

    #[test]
    fn recurrence_rule() {
        let (rem, prop) = prop_recurrence_rule::<Error>(b"RRULE:FREQ=DAILY;COUNT=10\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            RecurrenceRuleProperty {
                other_params: vec![],
                value: vec![
                    RecurRulePart::Freq(RecurFreq::Daily),
                    RecurRulePart::Count(10),
                ],
            }
        );
    }

    #[test]
    fn created() {
        let (rem, prop) = prop_created::<Error>(b"CREATED:19980118T230000Z\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            CreatedProperty {
                other_params: vec![],
                value: DateTime {
                    date: Date {
                        year: 1998,
                        month: 1,
                        day: 18,
                    },
                    time: Time {
                        hour: 23,
                        minute: 0,
                        second: 0,
                        is_utc: true,
                    },
                },
            }
        );
    }

    #[test]
    fn action() {
        let (rem, prop) = prop_action::<Error>(b"ACTION:DISPLAY\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            ActionProperty {
                other_params: vec![],
                value: Action::Display,
            }
        );
    }

    #[test]
    fn repeat() {
        let (rem, prop) = prop_repeat_count::<Error>(b"REPEAT:4\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            RepeatCountProperty {
                other_params: vec![],
                value: 4,
            }
        );
    }

    #[test]
    fn trigger_duration() {
        let (rem, prop) = prop_trigger::<Error>(b"TRIGGER:-PT15M\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            TriggerProperty {
                params: vec![],
                value: DurationOrDateTime::Duration(Duration {
                    sign: -1,
                    minutes: Some(15),
                    ..Default::default()
                }),
            }
        );
    }

    #[test]
    fn trigger_duration_related_end() {
        let (rem, prop) = prop_trigger::<Error>(b"TRIGGER;RELATED=END:PT5M\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            TriggerProperty {
                params: vec![ParamValue::Related {
                    related: Related::End,
                },],
                value: DurationOrDateTime::Duration(Duration {
                    sign: 1,
                    minutes: Some(5),
                    ..Default::default()
                }),
            }
        );
    }

    #[test]
    fn trigger_date_time() {
        let (rem, prop) =
            prop_trigger::<Error>(b"TRIGGER;VALUE=DATE-TIME:19980101T050000Z\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            TriggerProperty {
                params: vec![ParamValue::ValueType {
                    value: Value::DateTime,
                },],
                value: DurationOrDateTime::DateTime(DateTime {
                    date: Date {
                        year: 1998,
                        month: 1,
                        day: 1,
                    },
                    time: Time {
                        hour: 5,
                        minute: 0,
                        second: 0,
                        is_utc: true,
                    },
                }),
            }
        );
    }

    #[test]
    fn date_time_stamp() {
        let (rem, prop) = prop_created::<Error>(b"CREATED:19960329T133000Z\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            CreatedProperty {
                other_params: vec![],
                value: DateTime {
                    date: Date {
                        year: 1996,
                        month: 3,
                        day: 29,
                    },
                    time: Time {
                        hour: 13,
                        minute: 30,
                        second: 0,
                        is_utc: true,
                    },
                },
            }
        );
    }

    #[test]
    fn last_modified() {
        let (rem, prop) =
            prop_last_modified::<Error>(b"LAST-MODIFIED:19960817T133000Z\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            LastModifiedProperty {
                other_params: vec![],
                value: DateTime {
                    date: Date {
                        year: 1996,
                        month: 8,
                        day: 17,
                    },
                    time: Time {
                        hour: 13,
                        minute: 30,
                        second: 0,
                        is_utc: true,
                    },
                },
            }
        );
    }

    #[test]
    fn sequence() {
        let (rem, prop) = prop_sequence::<Error>(b"SEQUENCE:2\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            SequenceProperty {
                other_params: vec![],
                value: 2,
            }
        );
    }

    #[test]
    fn request_status() {
        let (rem, prop) = prop_request_status::<Error>(b"REQUEST-STATUS:2.0;Success\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            RequestStatusProperty {
                params: vec![],
                status_code: vec![2, 0],
                status_description: b"Success".to_vec(),
                exception_data: None,
            }
        );
    }

    #[test]
    fn request_status_rejected() {
        let (rem, prop) = prop_request_status::<Error>(
            b"REQUEST-STATUS:3.1;Invalid property value;DTSTART:96-Apr-01\r\n;",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            RequestStatusProperty {
                params: vec![],
                status_code: vec![3, 1],
                status_description: b"Invalid property value".to_vec(),
                exception_data: Some(b"DTSTART:96-Apr-01".to_vec()),
            }
        );
    }
}
