use crate::parser::param::{other_params, params, Encoding, Param, ParamValue, Value};
use crate::parser::property::recur::{recur, RecurRulePart};
use crate::parser::property::uri::{param_value_uri, Uri};
use crate::parser::property::{
    prop_value_binary, prop_value_calendar_user_address, prop_value_date, prop_value_date_time,
    prop_value_duration, prop_value_float, prop_value_integer, prop_value_period, prop_value_text,
    DateOrDateTime, DateOrDateTimeOrPeriod, DateTime, Duration, Period,
};
use crate::parser::{iana_token, read_int, x_name, Error};
use nom::branch::alt;
use nom::bytes::complete::take_while1;
use nom::bytes::streaming::tag;
use nom::character::is_digit;
use nom::character::streaming::char;
use nom::combinator::{map_res, opt, recognize, verify};
use nom::multi::{fold_many_m_n, separated_list1};
use nom::sequence::tuple;
use nom::{IResult, Parser};

#[derive(Debug, Eq, PartialEq)]
pub enum AttachValue<'a> {
    Uri(Uri<'a>),
    Binary(&'a [u8]),
}

#[derive(Debug, Eq, PartialEq)]
pub struct AttachProperty<'a> {
    pub params: Vec<Param<'a>>,
    pub value: AttachValue<'a>,
}

/// Parse an ATTACH property.
///
/// RFC 5545, section 3.8.1.1
pub fn prop_attach(input: &[u8]) -> IResult<&[u8], AttachProperty, Error> {
    let (input, (_, params, _)) = tuple((tag("ATTACH"), params, char(':')))(input)?;

    let is_base_64 = params.iter().any(|p| {
        matches!(
            p.value,
            ParamValue::Encoding {
                encoding: Encoding::Base64,
            }
        )
    });

    let is_binary = params.iter().any(|p| {
        matches!(
            p.value,
            ParamValue::Value {
                value: Value::Binary,
            }
        )
    });

    if is_base_64 && is_binary {
        let (input, (v, _)) = tuple((prop_value_binary, tag("\r\n")))(input)?;

        Ok((
            input,
            AttachProperty {
                params,
                value: AttachValue::Binary(v),
            },
        ))
    } else {
        let (input, (v, _)) = tuple((param_value_uri, tag("\r\n")))(input)?;

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
    pub params: Vec<Param<'a>>,
    pub value: Vec<Vec<u8>>,
}

/// Parse a CATEGORIES property.
///
/// RFC 5545, section 3.8.1.2
pub fn prop_categories(input: &[u8]) -> IResult<&[u8], CategoriesProperty, Error> {
    let (input, (_, params, _, value, _)) = tuple((
        tag("CATEGORIES"),
        params,
        char(':'),
        separated_list1(char(','), prop_value_text),
        tag("\r\n"),
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
    pub other_params: Vec<Param<'a>>,
    pub value: Classification<'a>,
}

/// Parse a CLASS property.
///
/// RFC 5545, section 3.8.1.3
pub fn prop_classification(input: &[u8]) -> IResult<&[u8], ClassificationProperty, Error> {
    let (input, (_, other_params, _, value, _)) = tuple((
        tag("CLASS"),
        other_params,
        char(':'),
        alt((
            tag("PUBLIC").map(|_| Classification::Public),
            tag("PRIVATE").map(|_| Classification::Private),
            tag("CONFIDENTIAL").map(|_| Classification::Confidential),
            x_name.map(Classification::XName),
            iana_token.map(Classification::IanaToken),
        )),
        tag("\r\n"),
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
    pub params: Vec<Param<'a>>,
    pub value: Vec<u8>,
}

/// Parse a COMMENT property.
///
/// RFC 5545, section 3.8.1.4
pub fn prop_comment(input: &[u8]) -> IResult<&[u8], CommentProperty, Error> {
    let (input, (_, params, _, value, _)) = tuple((
        tag("COMMENT"),
        params,
        char(':'),
        prop_value_text,
        tag("\r\n"),
    ))(input)?;

    Ok((input, CommentProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct DescriptionProperty<'a> {
    pub params: Vec<Param<'a>>,
    pub value: Vec<u8>,
}

/// Parse a DESCRIPTION property.
///
/// RFC 5545, section 3.8.1.5
pub fn prop_description(input: &[u8]) -> IResult<&[u8], DescriptionProperty, Error> {
    let (input, (_, params, _, value, _)) = tuple((
        tag("DESCRIPTION"),
        params,
        char(':'),
        prop_value_text.map(|v| v),
        tag("\r\n"),
    ))(input)?;

    Ok((input, DescriptionProperty { params, value }))
}

#[derive(Debug, PartialEq)]
pub struct GeographicPositionProperty<'a> {
    pub other_params: Vec<Param<'a>>,
    pub latitude: f64,
    pub longitude: f64,
}

/// Parse a GEO property.
///
/// RFC 5545, section 3.8.1.6
pub fn prop_geographic_position(input: &[u8]) -> IResult<&[u8], GeographicPositionProperty, Error> {
    let (input, (_, other_params, _, (latitude, _, longitude), _)) = tuple((
        tag("GEO"),
        other_params,
        char(':'),
        tuple((prop_value_float, char(';'), prop_value_float)),
        tag("\r\n"),
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
    pub params: Vec<Param<'a>>,
    pub value: Vec<u8>,
}

/// Parse a LOCATION property.
///
/// RFC 5545, section 3.8.1.7
pub fn prop_location(input: &[u8]) -> IResult<&[u8], LocationProperty, Error> {
    let (input, (_, params, _, value, _)) = tuple((
        tag("LOCATION"),
        params,
        char(':'),
        prop_value_text,
        tag("\r\n"),
    ))(input)?;

    Ok((input, LocationProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct PercentCompleteProperty<'a> {
    pub other_params: Vec<Param<'a>>,
    pub value: u8,
}

/// Parse a PERCENT-COMPLETE property.
///
/// RFC 5545, section 3.8.1.8
pub fn prop_percent_complete(input: &[u8]) -> IResult<&[u8], PercentCompleteProperty, Error> {
    let (input, (_, other_params, _, value, _)) = tuple((
        tag("PERCENT-COMPLETE"),
        other_params,
        char(':'),
        verify(prop_value_integer, |v| 0 <= *v && *v <= 100).map(|v| v as u8),
        tag("\r\n"),
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
    pub other_params: Vec<Param<'a>>,
    pub value: u8,
}

/// Parse a PRIORITY property.
///
/// RFC 5545, section 3.8.1.9
pub fn prop_priority(input: &[u8]) -> IResult<&[u8], PriorityProperty, Error> {
    let (input, (_, other_params, _, value, _)) = tuple((
        tag("PRIORITY"),
        other_params,
        char(':'),
        verify(prop_value_integer, |v| 0 <= *v && *v <= 9).map(|v| v as u8),
        tag("\r\n"),
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
    pub params: Vec<Param<'a>>,
    pub value: Vec<Vec<u8>>,
}

/// Parse a RESOURCES property.
///
/// RFC 5545, section 3.8.1.10
pub fn prop_resources(input: &[u8]) -> IResult<&[u8], ResourcesProperty, Error> {
    let (input, (_, params, _, value, _)) = tuple((
        tag("RESOURCES"),
        params,
        char(':'),
        separated_list1(char(','), prop_value_text),
        tag("\r\n"),
    ))(input)?;

    Ok((input, ResourcesProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub enum Status {
    Tentative,
    Confirmed,
    Cancelled,
    NeedsAction,
    Completed,
    InProcess,
    Draft,
    Final,
}

#[derive(Debug, Eq, PartialEq)]
pub struct StatusProperty<'a> {
    pub other_params: Vec<Param<'a>>,
    pub value: Status,
}

/// Parse a STATUS property.
///
/// RFC 5545, section 3.8.1.11
pub fn prop_status(input: &[u8]) -> IResult<&[u8], StatusProperty, Error> {
    let (input, (_, other_params, _, value, _)) = tuple((
        tag("STATUS"),
        other_params,
        char(':'),
        alt((
            tag("TENTATIVE").map(|_| Status::Tentative),
            tag("CONFIRMED").map(|_| Status::Confirmed),
            tag("CANCELLED").map(|_| Status::Cancelled),
            tag("NEEDS-ACTION").map(|_| Status::NeedsAction),
            tag("COMPLETED").map(|_| Status::Completed),
            tag("IN-PROCESS").map(|_| Status::InProcess),
            tag("DRAFT").map(|_| Status::Draft),
            tag("FINAL").map(|_| Status::Final),
        )),
        tag("\r\n"),
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
    pub params: Vec<Param<'a>>,
    pub value: Vec<u8>,
}

/// Parse a SUMMARY property.
///
/// RFC 5545, section 3.8.1.12
pub fn prop_summary(input: &[u8]) -> IResult<&[u8], SummaryProperty, Error> {
    let (input, (_, params, _, value, _)) = tuple((
        tag("SUMMARY"),
        params,
        char(':'),
        prop_value_text,
        tag("\r\n"),
    ))(input)?;

    Ok((input, SummaryProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct DateTimeCompletedProperty<'a> {
    pub other_params: Vec<Param<'a>>,
    pub value: DateTime,
}

/// Parse a COMPLETED property.
///
/// RFC 5545, section 3.8.2.1
pub fn prop_date_time_completed(input: &[u8]) -> IResult<&[u8], DateTimeCompletedProperty, Error> {
    let (input, (_, other_params, _, value, _)) = tuple((
        tag("COMPLETED"),
        other_params,
        char(':'),
        prop_value_date_time,
        tag("\r\n"),
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
    pub params: Vec<Param<'a>>,
    pub value: DateOrDateTime,
}

/// Parse a DTEND property.
///
/// RFC 5545, section 3.8.2.2
pub fn prop_date_time_end(input: &[u8]) -> IResult<&[u8], DateTimeEndProperty, Error> {
    let (input, (_, params, _, value, _)) = tuple((
        tag("DTEND"),
        params,
        char(':'),
        alt((
            prop_value_date_time.map(DateOrDateTime::DateTime),
            prop_value_date.map(DateOrDateTime::Date),
        )),
        tag("\r\n"),
    ))(input)?;

    Ok((input, DateTimeEndProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct DateTimeDueProperty<'a> {
    pub params: Vec<Param<'a>>,
    pub value: DateOrDateTime,
}

/// Parse a DUE property.
///
/// RFC 5545, section 3.8.2.3
pub fn prop_date_time_due(input: &[u8]) -> IResult<&[u8], DateTimeDueProperty, Error> {
    let (input, (_, params, _, value, _)) = tuple((
        tag("DUE"),
        params,
        char(':'),
        alt((
            prop_value_date_time.map(DateOrDateTime::DateTime),
            prop_value_date.map(DateOrDateTime::Date),
        )),
        tag("\r\n"),
    ))(input)?;

    Ok((input, DateTimeDueProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct DateTimeStartProperty<'a> {
    pub params: Vec<Param<'a>>,
    pub value: DateOrDateTime,
}

/// Parse a DTSTART property.
///
/// RFC 5545, section 3.8.2.4
pub fn prop_date_time_start(input: &[u8]) -> IResult<&[u8], DateTimeStartProperty, Error> {
    let (input, (_, params, _, value, _)) = tuple((
        tag("DTSTART"),
        params,
        char(':'),
        alt((
            prop_value_date_time.map(DateOrDateTime::DateTime),
            prop_value_date.map(DateOrDateTime::Date),
        )),
        tag("\r\n"),
    ))(input)?;

    Ok((input, DateTimeStartProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct DurationProperty<'a> {
    pub other_params: Vec<Param<'a>>,
    pub value: Duration,
}

/// Parse a DURATION property.
///
/// RFC 5545, section 3.8.2.5
pub fn prop_duration(input: &[u8]) -> IResult<&[u8], DurationProperty, Error> {
    let (input, (_, other_params, _, value, _)) = tuple((
        tag("DURATION"),
        other_params,
        char(':'),
        prop_value_duration,
        tag("\r\n"),
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
    pub params: Vec<Param<'a>>,
    pub value: Vec<Period<'a>>,
}

/// Parse a FREEBUSY property.
///
/// RFC 5545, section 3.8.2.6
pub fn prop_free_busy_time(input: &[u8]) -> IResult<&[u8], FreeBusyTimeProperty, Error> {
    let (input, (_, params, _, value, _)) = tuple((
        tag("FREEBUSY"),
        params,
        char(':'),
        separated_list1(char(','), prop_value_period),
        tag("\r\n"),
    ))(input)?;

    Ok((input, FreeBusyTimeProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub enum TimeTransparency {
    Opaque,
    Transparent,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TimeTransparencyProperty<'a> {
    pub other_params: Vec<Param<'a>>,
    pub value: TimeTransparency,
}

/// Parse a TRANSP property.
///
/// RFC 5545, section 3.8.2.7
pub fn prop_time_transparency(input: &[u8]) -> IResult<&[u8], TimeTransparencyProperty, Error> {
    let (input, (_, other_params, _, value, _)) = tuple((
        tag("TRANSP"),
        other_params,
        char(':'),
        alt((
            tag("OPAQUE").map(|_| TimeTransparency::Opaque),
            tag("TRANSPARENT").map(|_| TimeTransparency::Transparent),
        )),
        tag("\r\n"),
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
pub struct AttendeeProperty<'a> {
    pub params: Vec<Param<'a>>,
    pub value: &'a [u8],
}

/// Parse an ATTENDEE property.
///
/// RFC 5545, section 3.8.4.1
pub fn prop_attendee(input: &[u8]) -> IResult<&[u8], AttendeeProperty, Error> {
    let (input, (_, params, _, uri, _)) = tuple((
        tag("ATTENDEE"),
        params,
        char(':'),
        recognize(prop_value_calendar_user_address),
        tag("\r\n"),
    ))(input)?;

    Ok((input, AttendeeProperty { params, value: uri }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct ContactProperty<'a> {
    pub params: Vec<Param<'a>>,
    pub value: Vec<u8>,
}

/// Parse a CONTACT property.
///
/// RFC 5545, section 3.8.4.2
pub fn prop_contact(input: &[u8]) -> IResult<&[u8], ContactProperty, Error> {
    let (input, (_, params, _, value, _)) = tuple((
        tag("CONTACT"),
        params,
        char(':'),
        prop_value_text,
        tag("\r\n"),
    ))(input)?;

    Ok((input, ContactProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrganizerProperty<'a> {
    pub params: Vec<Param<'a>>,
    pub value: &'a [u8],
}

/// Parse an ORGANIZER property.
///
/// RFC 5545, section 3.8.4.3
pub fn prop_organizer(input: &[u8]) -> IResult<&[u8], OrganizerProperty, Error> {
    let (input, (_, params, _, uri, _)) = tuple((
        tag("ORGANIZER"),
        params,
        char(':'),
        recognize(prop_value_calendar_user_address),
        tag("\r\n"),
    ))(input)?;

    Ok((input, OrganizerProperty { params, value: uri }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct RecurrenceIdProperty<'a> {
    pub params: Vec<Param<'a>>,
    pub value: DateOrDateTime,
}

/// Parse a RECURRENCE-ID property.
///
/// RFC 5545, section 3.8.4.4
pub fn prop_recurrence_id(input: &[u8]) -> IResult<&[u8], RecurrenceIdProperty, Error> {
    let (input, (_, params, _, value, _)) = tuple((
        tag("RECURRENCE-ID"),
        params,
        char(':'),
        alt((
            prop_value_date_time.map(DateOrDateTime::DateTime),
            prop_value_date.map(DateOrDateTime::Date),
        )),
        tag("\r\n"),
    ))(input)?;

    Ok((input, RecurrenceIdProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct RelatedToProperty<'a> {
    pub params: Vec<Param<'a>>,
    pub value: Vec<u8>,
}

/// Parse a RELATED-TO property.
///
/// RFC 5545, section 3.8.4.5
pub fn prop_related_to(input: &[u8]) -> IResult<&[u8], RelatedToProperty, Error> {
    let (input, (_, params, _, value, _)) = tuple((
        tag("RELATED-TO"),
        params,
        char(':'),
        prop_value_text,
        tag("\r\n"),
    ))(input)?;

    Ok((input, RelatedToProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct UrlProperty<'a> {
    pub other_params: Vec<Param<'a>>,
    pub value: Uri<'a>,
}

/// Parse a URL property.
///
/// RFC 5545, section 3.8.4.6
pub fn prop_url(input: &[u8]) -> IResult<&[u8], UrlProperty, Error> {
    let (input, (_, other_params, _, value, _)) = tuple((
        tag("URL"),
        other_params,
        char(':'),
        param_value_uri,
        tag("\r\n"),
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
    pub other_params: Vec<Param<'a>>,
    pub value: Vec<u8>,
}

/// Parse a UID property.
///
/// RFC 5545, section 3.8.4.7
pub fn prop_unique_identifier(input: &[u8]) -> IResult<&[u8], UniqueIdentifierProperty, Error> {
    let (input, (_, other_params, _, value, _)) = tuple((
        tag("UID"),
        other_params,
        char(':'),
        prop_value_text,
        tag("\r\n"),
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
    pub params: Vec<Param<'a>>,
    pub value: Vec<DateOrDateTime>,
}

/// Parse an EXDATE property.
///
/// RFC 5545, section 3.8.5.1
pub fn prop_exception_date_times(
    input: &[u8],
) -> IResult<&[u8], ExceptionDateTimesProperty, Error> {
    let (input, (_, params, _, value, _)) = tuple((
        tag("EXDATE"),
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
    ))(input)?;

    Ok((input, ExceptionDateTimesProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct RecurrenceDateTimesProperty<'a> {
    pub params: Vec<Param<'a>>,
    pub value: Vec<DateOrDateTimeOrPeriod<'a>>,
}

/// Parse an RDATE property.
///
/// RFC 5545, section 3.8.5.2
pub fn prop_recurrence_date_times(
    input: &[u8],
) -> IResult<&[u8], RecurrenceDateTimesProperty, Error> {
    let (input, (_, params, _, value, _)) = tuple((
        tag("RDATE"),
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
    ))(input)?;

    Ok((input, RecurrenceDateTimesProperty { params, value }))
}

#[derive(Debug, Eq, PartialEq)]
pub struct RecurrenceRuleProperty<'a> {
    pub other_params: Vec<Param<'a>>,
    pub value: Vec<RecurRulePart>,
}

/// Parse an RRULE property.
///
/// RFC 5545, section 3.8.5.3
pub fn prop_recurrence_rule(input: &[u8]) -> IResult<&[u8], RecurrenceRuleProperty, Error> {
    let (input, (_, other_params, _, value, _)) =
        tuple((tag("RRULE"), other_params, char(':'), recur, tag("\r\n")))(input)?;

    Ok((
        input,
        RecurrenceRuleProperty {
            other_params,
            value,
        },
    ))
}

#[derive(Debug, Eq, PartialEq)]
pub struct CreatedProperty<'a> {
    pub other_params: Vec<Param<'a>>,
    pub value: DateTime,
}

/// Parse a CREATED property.
///
/// RFC 5545, section 3.8.7.1
pub fn prop_created(input: &[u8]) -> IResult<&[u8], CreatedProperty, Error> {
    let (input, (_, other_params, _, value, _)) = tuple((
        tag("CREATED"),
        other_params,
        char(':'),
        prop_value_date_time,
        tag("\r\n"),
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
    pub other_params: Vec<Param<'a>>,
    pub value: DateTime,
}

/// Parse a DTSTAMP property.
///
/// RFC 5545, section 3.8.7.2
pub fn prop_date_time_stamp(input: &[u8]) -> IResult<&[u8], DateTimeStampProperty, Error> {
    let (input, (_, other_params, _, value, _)) = tuple((
        tag("DTSTAMP"),
        other_params,
        char(':'),
        prop_value_date_time,
        tag("\r\n"),
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
    pub other_params: Vec<Param<'a>>,
    pub value: DateTime,
}

/// Parse a LAST-MODIFIED property.
///
/// RFC 5545, section 3.8.7.3
pub fn prop_last_modified(input: &[u8]) -> IResult<&[u8], LastModifiedProperty, Error> {
    let (input, (_, other_params, _, value, _)) = tuple((
        tag("LAST-MODIFIED"),
        other_params,
        char(':'),
        prop_value_date_time,
        tag("\r\n"),
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
    pub other_params: Vec<Param<'a>>,
    pub value: u32,
}

/// Parse a SEQUENCE property.
///
/// RFC 5545, section 3.8.7.4
pub fn prop_sequence(input: &[u8]) -> IResult<&[u8], SequenceProperty, Error> {
    let (input, (_, other_params, _, value, _)) = tuple((
        tag("SEQUENCE"),
        other_params,
        char(':'),
        prop_value_integer.map(|v| v as u32),
        tag("\r\n"),
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
    pub params: Vec<Param<'a>>,
    pub status_code: Vec<u32>,
    pub status_description: Vec<u8>,
    pub extra_data: Option<Vec<u8>>,
}

/// Parse a REQUEST-STATUS property.
///
/// RFC 5545, section 3.8.8.3
pub fn prop_request_status(input: &[u8]) -> IResult<&[u8], RequestStatusProperty, Error> {
    fn status_code(input: &[u8]) -> IResult<&[u8], Vec<u32>, Error> {
        let (input, (num, mut nums)) = tuple((
            map_res(
                verify(take_while1(is_digit), |v: &[u8]| v.len() == 1),
                |v| read_int::<u32>(v),
            ),
            fold_many_m_n(
                1,
                2,
                map_res(tuple((char('.'), take_while1(is_digit))), |(_, v)| {
                    read_int::<u32>(v)
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

    let (input, (_, params, _, status_code, _, status_description, extra_data, _)) = tuple((
        tag("REQUEST-STATUS"),
        params,
        char(':'),
        status_code,
        char(';'),
        prop_value_text,
        opt(tuple((char(';'), prop_value_text)).map(|(_, v)| v)),
        tag("\r\n"),
    ))(input)?;

    Ok((
        input,
        RequestStatusProperty {
            params,
            status_code,
            status_description,
            extra_data,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::param::{
        FreeBusyTimeType, ParamValue, ParticipationStatusUnknown, Range, Role, Value,
    };
    use crate::parser::property::recur::RecurFreq;
    use crate::parser::property::uri::{Authority, Host};
    use crate::parser::property::{Date, Period, PeriodEnd, Time};
    use crate::test_utils::check_rem;
    use base64::Engine;

    #[test]
    fn attach_uri() {
        let (rem, prop) =
            prop_attach(b"ATTACH:CID:jsmith.part3.960817T083000.xyzMail@example.com\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            AttachProperty {
                params: vec![],
                value: AttachValue::Uri(Uri {
                    scheme: b"CID",
                    authority: None,
                    path: b"jsmith.part3.960817T083000.xyzMail@example.com".to_vec(),
                    query: None,
                    fragment: None,
                }),
            }
        );
    }

    #[test]
    fn attach_binary() {
        let (rem, prop) =
            prop_attach(b"ATTACH;VALUE=BINARY;ENCODING=BASE64:dGVzdA==\r\n;").unwrap();
        check_rem(rem, 1);

        let r = base64::prelude::BASE64_STANDARD.encode("test");

        assert_eq!(
            prop,
            AttachProperty {
                params: vec![
                    Param {
                        name: "VALUE".to_string(),
                        value: ParamValue::Value {
                            value: Value::Binary
                        },
                    },
                    Param {
                        name: "ENCODING".to_string(),
                        value: ParamValue::Encoding {
                            encoding: Encoding::Base64,
                        },
                    },
                ],
                value: AttachValue::Binary(r.as_bytes()),
            }
        );
    }

    #[test]
    fn categories() {
        let (rem, prop) = prop_categories(b"CATEGORIES:APPOINTMENT,EDUCATION\r\n;").unwrap();
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
        let (rem, prop) = prop_classification(b"CLASS:PUBLIC\r\n;").unwrap();
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
        let (rem, prop) = prop_comment(b"COMMENT:The meeting really needs to include both ourselves and the customer. We can't hold this meeting without them. As a matter of fact\\, the venue for the meeting ought to be at their site. - - John\r\n;").unwrap();
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
        let (rem, prop) = prop_description(b"DESCRIPTION:Meeting to provide technical review for \"Phoenix\"\r\n  design.\\nHappy Face Conference Room. Phoenix design team\r\n  MUST attend this meeting.\\nRSVP to team leader.\r\n;").unwrap();
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
        let (rem, prop) = prop_geographic_position(b"GEO:37.386013;-122.082932\r\n;").unwrap();
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
            prop_location(b"LOCATION:Conference Room - F123\\, Bldg. 002\r\n;").unwrap();
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
        let (rem, prop) = prop_location(b"LOCATION;ALTREP=\"http://xyzcorp.com/conf-rooms/f123.vcf\":\r\n Conference Room - F123\\, Bldg. 002\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            LocationProperty {
                params: vec![Param {
                    name: "ALTREP".to_string(),
                    value: ParamValue::AltRep {
                        uri: "http://xyzcorp.com/conf-rooms/f123.vcf".to_string(),
                    },
                }],
                value: b"Conference Room - F123, Bldg. 002".to_vec(),
            }
        );
    }

    #[test]
    fn percent_complete() {
        let (rem, prop) = prop_percent_complete(b"PERCENT-COMPLETE:39\r\n;").unwrap();
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
        let (rem, prop) = prop_priority(b"PRIORITY:1\r\n;").unwrap();
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
        let (rem, prop) = prop_resources(b"RESOURCES:EASEL,PROJECTOR,VCR\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            ResourcesProperty {
                params: vec![],
                value: vec![b"EASEL".to_vec(), b"PROJECTOR".to_vec(), b"VCR".to_vec(),],
            }
        );
    }

    #[test]
    fn status() {
        let (rem, prop) = prop_status(b"STATUS:TENTATIVE\r\n;").unwrap();
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
        let (rem, prop) = prop_summary(b"SUMMARY:Department Party\r\n;").unwrap();
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
        let (rem, prop) = prop_date_time_completed(b"COMPLETED:19960401T150000Z\r\n;").unwrap();
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
        let (rem, prop) = prop_date_time_end(b"DTEND;VALUE=DATE:19980704\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            DateTimeEndProperty {
                params: vec![Param {
                    name: "VALUE".to_string(),
                    value: ParamValue::Value { value: Value::Date },
                },],
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
        let (rem, prop) = prop_date_time_end(b"DTEND:19960401T150000Z\r\n;").unwrap();
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
        let (rem, prop) = prop_date_time_due(b"DUE;VALUE=DATE:19980401\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            DateTimeDueProperty {
                params: vec![Param {
                    name: "VALUE".to_string(),
                    value: ParamValue::Value { value: Value::Date },
                },],
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
        let (rem, prop) = prop_date_time_due(b"DUE:19980430T000000Z\r\n;").unwrap();
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
        let (rem, prop) = prop_date_time_start(b"DTSTART:19980118\r\n;").unwrap();
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
        let (rem, prop) = prop_date_time_start(b"DTSTART:19980118T073000Z\r\n;").unwrap();
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
        let (rem, prop) = prop_duration(b"DURATION:PT1H0M0S\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            DurationProperty {
                other_params: vec![],
                value: Duration {
                    sign: 1,
                    weeks: 0,
                    days: 0,
                    seconds: 3600,
                },
            }
        );
    }

    #[test]
    fn free_busy() {
        let (rem, prop) =
            prop_free_busy_time(b"FREEBUSY;FBTYPE=BUSY-UNAVAILABLE:19970308T160000Z/PT8H30M\r\n;")
                .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            FreeBusyTimeProperty {
                params: vec![Param {
                    name: "FBTYPE".to_string(),
                    value: ParamValue::FreeBusyTimeType {
                        fb_type: FreeBusyTimeType::BusyUnavailable,
                    },
                }],
                value: vec![Period {
                    start: b"19970308T160000Z",
                    end: PeriodEnd::Duration(Duration {
                        sign: 1,
                        weeks: 0,
                        days: 0,
                        seconds: 30600,
                    }),
                }],
            }
        );
    }

    #[test]
    fn free_busy_multiple() {
        let (rem, prop) = prop_free_busy_time(
            b"FREEBUSY;FBTYPE=FREE:19970308T160000Z/PT3H,19970308T200000Z/PT1H\r\n;",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            FreeBusyTimeProperty {
                params: vec![Param {
                    name: "FBTYPE".to_string(),
                    value: ParamValue::FreeBusyTimeType {
                        fb_type: FreeBusyTimeType::Free,
                    },
                }],
                value: vec![
                    Period {
                        start: b"19970308T160000Z",
                        end: PeriodEnd::Duration(Duration {
                            sign: 1,
                            weeks: 0,
                            days: 0,
                            seconds: 10800,
                        }),
                    },
                    Period {
                        start: b"19970308T200000Z",
                        end: PeriodEnd::Duration(Duration {
                            sign: 1,
                            weeks: 0,
                            days: 0,
                            seconds: 3600,
                        }),
                    }
                ],
            }
        );
    }

    #[test]
    fn transp_opaque() {
        let (rem, prop) = prop_time_transparency(b"TRANSP:OPAQUE\r\n;").unwrap();
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
        let (rem, prop) = prop_time_transparency(b"TRANSP:TRANSPARENT\r\n;").unwrap();
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
    fn attendee() {
        let (rem, prop) = prop_attendee(b"ATTENDEE;ROLE=REQ-PARTICIPANT;DELEGATED-FROM=\"mailto:bob@example.com\";PARTSTAT=ACCEPTED;CN=Jane Doe:mailto:jdoe@example.com\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            AttendeeProperty {
                params: vec![
                    Param {
                        name: "ROLE".to_string(),
                        value: ParamValue::Role {
                            role: Role::RequiredParticipant,
                        },
                    },
                    Param {
                        name: "DELEGATED-FROM".to_string(),
                        value: ParamValue::DelegatedFrom {
                            delegators: vec!["mailto:bob@example.com".to_string()],
                        },
                    },
                    Param {
                        name: "PARTSTAT".to_string(),
                        value: ParamValue::ParticipationStatus {
                            status: ParticipationStatusUnknown::Accepted,
                        },
                    },
                    Param {
                        name: "CN".to_string(),
                        value: ParamValue::CommonName {
                            name: "Jane Doe".to_string(),
                        },
                    }
                ],
                value: b"mailto:jdoe@example.com",
            }
        );
    }

    #[test]
    fn contact() {
        let (rem, prop) =
            prop_contact(b"CONTACT:Jim Dolittle\\, ABC Industries\\, +1-919-555-1234\r\n;")
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
        let (rem, prop) = prop_contact(b"CONTACT;ALTREP=\"ldap://example.com:6666/o=ABC%20Industries,c=US???(cn=Jim%20Dolittle)\":Jim Dolittle\\, ABC Industries\\, +1-919-555-1234\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            ContactProperty {
                params: vec![Param {
                    name: "ALTREP".to_string(),
                    value: ParamValue::AltRep {
                        uri:
                            "ldap://example.com:6666/o=ABC%20Industries,c=US???(cn=Jim%20Dolittle)"
                                .to_string(),
                    },
                }],
                value: b"Jim Dolittle, ABC Industries, +1-919-555-1234".to_vec(),
            }
        );
    }

    #[test]
    fn organizer() {
        let (rem, prop) =
            prop_organizer(b"ORGANIZER;CN=John Smith:mailto:jsmith@example.com\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            OrganizerProperty {
                params: vec![Param {
                    name: "CN".to_string(),
                    value: ParamValue::CommonName {
                        name: "John Smith".to_string(),
                    },
                }],
                value: b"mailto:jsmith@example.com",
            }
        );
    }

    #[test]
    fn organizer_with_params() {
        let (rem, prop) = prop_organizer(b"ORGANIZER;CN=JohnSmith;DIR=\"ldap://example.com:6666/o=DC%20Associates,c=US???(cn=John%20Smith)\":mailto:jsmith@example.com\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            OrganizerProperty {
                params: vec![
                    Param {
                        name: "CN".to_string(),
                        value: ParamValue::CommonName {
                            name: "JohnSmith".to_string(),
                        },
                    },
                    Param {
                        name: "DIR".to_string(),
                        value: ParamValue::Dir {
                            uri:
                                "ldap://example.com:6666/o=DC%20Associates,c=US???(cn=John%20Smith)"
                                    .to_string(),
                        },
                    },
                ],
                value: b"mailto:jsmith@example.com",
            }
        );
    }

    #[test]
    fn organizer_with_sent_by_param() {
        let (rem, prop) = prop_organizer(
            b"ORGANIZER;SENT-BY=\"mailto:jane_doe@example.com\":mailto:jsmith@example.com\r\n;",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            OrganizerProperty {
                params: vec![Param {
                    name: "SENT-BY".to_string(),
                    value: ParamValue::SentBy {
                        address: "mailto:jane_doe@example.com".to_string(),
                    },
                }],
                value: b"mailto:jsmith@example.com",
            }
        );
    }

    #[test]
    fn recurrence_id_date() {
        let (rem, prop) = prop_recurrence_id(b"RECURRENCE-ID;VALUE=DATE:19960401\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            RecurrenceIdProperty {
                params: vec![Param {
                    name: "VALUE".to_string(),
                    value: ParamValue::Value { value: Value::Date },
                }],
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
            prop_recurrence_id(b"RECURRENCE-ID;RANGE=THISANDFUTURE:19960120T120000Z\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            RecurrenceIdProperty {
                params: vec![Param {
                    name: "RANGE".to_string(),
                    value: ParamValue::Range {
                        range: Range::ThisAndFuture,
                    },
                }],
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
        let (rem, prop) =
            prop_related_to(b"RELATED-TO:jsmith.part7.19960817T083000.xyzMail@example.com\r\n;")
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
            prop_url(b"URL:http://example.com/pub/calendars/jsmith/mytime.ics\r\n;").unwrap();
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
        let (rem, prop) =
            prop_unique_identifier(b"UID:19960401T080045Z-4000F192713-0052@example.com\r\n;")
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
        let (rem, prop) = prop_exception_date_times(
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
        let (rem, prop) =
            prop_recurrence_date_times(b"RDATE;TZID=America/New_York:19970714T083000\r\n;")
                .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            RecurrenceDateTimesProperty {
                params: vec![Param {
                    name: "TZID".to_string(),
                    value: ParamValue::TimeZoneId {
                        tz_id: "America/New_York".to_string(),
                        unique: false,
                    },
                }],
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
        let (rem, prop) = prop_recurrence_date_times(
            b"RDATE;VALUE=PERIOD:19960403T020000Z/19960403T040000Z,19960404T010000Z/PT3H\r\n;",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            RecurrenceDateTimesProperty {
                params: vec![Param {
                    name: "VALUE".to_string(),
                    value: ParamValue::Value {
                        value: Value::Period
                    },
                }],
                value: vec![
                    DateOrDateTimeOrPeriod::Period(Period {
                        start: b"19960403T020000Z",
                        end: PeriodEnd::DateTime(b"19960403T040000Z"),
                    }),
                    DateOrDateTimeOrPeriod::Period(Period {
                        start: b"19960404T010000Z",
                        end: PeriodEnd::Duration(Duration {
                            sign: 1,
                            weeks: 0,
                            days: 0,
                            seconds: 10800,
                        }),
                    }),
                ],
            }
        );
    }

    #[test]
    fn recurrence_date_times_dates() {
        let (rem, prop) = prop_recurrence_date_times(
            b"RDATE;VALUE=DATE:19970101,19970120,19970217,19970421\r\n;",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            RecurrenceDateTimesProperty {
                params: vec![Param {
                    name: "VALUE".to_string(),
                    value: ParamValue::Value { value: Value::Date },
                }],
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
        let (rem, prop) = prop_recurrence_rule(b"RRULE:FREQ=DAILY;COUNT=10\r\n;").unwrap();
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
        let (rem, prop) = prop_created(b"CREATED:19980118T230000Z\r\n;").unwrap();
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
    fn date_time_stamp() {
        let (rem, prop) = prop_created(b"CREATED:19960329T133000Z\r\n;").unwrap();
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
        let (rem, prop) = prop_last_modified(b"LAST-MODIFIED:19960817T133000Z\r\n;").unwrap();
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
        let (rem, prop) = prop_sequence(b"SEQUENCE:2\r\n;").unwrap();
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
        let (rem, prop) = prop_request_status(b"REQUEST-STATUS:2.0;Success\r\n;").unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            RequestStatusProperty {
                params: vec![],
                status_code: vec![2, 0],
                status_description: b"Success".to_vec(),
                extra_data: None,
            }
        );
    }

    #[test]
    fn request_status_rejected() {
        let (rem, prop) = prop_request_status(
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
                extra_data: Some(b"DTSTART:96-Apr-01".to_vec()),
            }
        );
    }
}
