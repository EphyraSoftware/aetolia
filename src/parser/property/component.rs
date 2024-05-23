use crate::parser::param::{other_params, params, Param};
use crate::parser::property::{
    prop_value_calendar_user_address, prop_value_date, prop_value_date_time, prop_value_float,
    prop_value_text, DateOrDateTime, DateTime,
};
use crate::parser::{iana_token, x_name, Error};
use nom::branch::alt;
use nom::bytes::streaming::tag;
use nom::character::streaming::char;
use nom::combinator::recognize;
use nom::sequence::tuple;
use nom::{IResult, Parser};

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
pub struct DescriptionProperty<'a> {
    pub params: Vec<Param<'a>>,
    pub value: Vec<u8>,
}

/// Parse a DESCRIPTION property.
///
/// RFC 5545, section 3.8.1.5
fn prop_description(input: &[u8]) -> IResult<&[u8], DescriptionProperty, Error> {
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
fn prop_geographic_position(input: &[u8]) -> IResult<&[u8], GeographicPositionProperty, Error> {
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
pub struct UniqueIdentifier<'a> {
    pub other_params: Vec<Param<'a>>,
    pub value: Vec<u8>,
}

/// Parse a UID property.
///
/// RFC 5545, section 3.8.4.7
pub fn prop_unique_identifier(input: &[u8]) -> IResult<&[u8], UniqueIdentifier, Error> {
    let (input, (_, other_params, _, value, _)) = tuple((
        tag("UID"),
        other_params,
        char(':'),
        prop_value_text,
        tag("\r\n"),
    ))(input)?;

    Ok((
        input,
        UniqueIdentifier {
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
pub fn prop_date_time_created(input: &[u8]) -> IResult<&[u8], CreatedProperty, Error> {
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
pub struct DateTimeStamp<'a> {
    pub other_params: Vec<Param<'a>>,
    pub value: DateTime,
}

/// Parse a DTSTAMP property.
///
/// RFC 5545, section 3.8.7.2
pub fn prop_date_time_stamp(input: &[u8]) -> IResult<&[u8], DateTimeStamp, Error> {
    let (input, (_, other_params, _, value, _)) = tuple((
        tag("DTSTAMP"),
        other_params,
        char(':'),
        prop_value_date_time,
        tag("\r\n"),
    ))(input)?;

    Ok((
        input,
        DateTimeStamp {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::param::ParamValue;
    use crate::parser::property::{Date, Time};
    use crate::test_utils::check_rem;

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
                    }
                }],
                value: b"Conference Room - F123, Bldg. 002".to_vec(),
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
                    }
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
                        }
                    },
                    Param {
                        name: "DIR".to_string(),
                        value: ParamValue::Dir {
                            uri:
                                "ldap://example.com:6666/o=DC%20Associates,c=US???(cn=John%20Smith)"
                                    .to_string(),
                        }
                    }
                ],
                value: b"mailto:jsmith@example.com",
            }
        );
    }

    #[test]
    #[ignore = "Unfolding required for all values, not just text"]
    fn organizer_with_sent_by_param() {
        let (rem, prop) = prop_organizer(
            b"ORGANIZER;SENT-BY=\"mailto:jane_doe@example.com\":\r\n mailto:jsmith@example.com",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(
            prop,
            OrganizerProperty {
                params: vec![Param {
                    name: "SENT-BY".to_string(),
                    value: ParamValue::AltRep {
                        uri: "mailto:jane_doe@example.com".to_string(),
                    }
                }],
                value: b"mailto:jsmith@example.com",
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
            UniqueIdentifier {
                other_params: vec![],
                value: b"19960401T080045Z-4000F192713-0052@example.com".to_vec(),
            }
        );
    }

    #[test]
    fn created() {
        let (rem, prop) = prop_date_time_created(b"CREATED:19980118T230000Z\r\n;").unwrap();
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
        let (rem, prop) = prop_date_time_created(b"CREATED:19960329T133000Z\r\n;").unwrap();
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
                }
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
                }
            }
        );
    }
}
