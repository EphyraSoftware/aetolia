mod component;
mod recur;
pub mod types;
pub(crate) mod uri;
mod value;
mod value_types;

use crate::parser::param::{other_params, params};
use crate::parser::property::types::{
    CalendarScaleProperty, IanaProperty, MethodProperty, ProductId, VersionProperty, XProperty,
};
use crate::parser::{iana_token, value, x_name, Error};
use crate::single;
pub use component::*;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::bytes::streaming::tag;
use nom::character::is_digit;
use nom::character::streaming::char;
use nom::combinator::{cut, recognize, verify};
use nom::error::ParseError;
use nom::sequence::tuple;
use nom::{IResult, Parser};
pub use value::*;
pub use value_types::*;

pub fn prop_product_id<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ProductId<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (_, params, _, value, _)) = tuple((
        tag_no_case("PRODID"),
        cut(other_params),
        char(':'),
        prop_value_text,
        tag("\r\n"),
    ))(input)?;

    Ok((
        input,
        ProductId {
            other_params: params,
            value,
        },
    ))
}

pub fn prop_version<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], VersionProperty<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (_, params, _, (min_ver, max_ver), _)) = tuple((
        tag_no_case("VERSION"),
        cut(other_params),
        char(':'),
        alt((
            tuple((
                recognize(tuple((single(is_digit), char('.'), single(is_digit)))),
                char(';'),
                recognize(tuple((single(is_digit), char('.'), single(is_digit)))),
            ))
            .map(|(min_ver, _, max_ver)| (Some(min_ver), max_ver)),
            recognize(tuple((single(is_digit), char('.'), single(is_digit))))
                .map(|v| (Option::<&[u8]>::None, v)),
        )),
        tag("\r\n"),
    ))(input)?;

    Ok((
        input,
        VersionProperty {
            other_params: params,
            min_version: min_ver,
            max_version: max_ver,
        },
    ))
}

pub fn prop_calendar_scale<'a, E>(
    input: &'a [u8],
) -> IResult<&'a [u8], CalendarScaleProperty<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (_, params, _, value, _)) = tuple((
        tag_no_case("CALSCALE"),
        cut(other_params),
        char(':'),
        prop_value_text,
        tag("\r\n"),
    ))(input)?;

    Ok((
        input,
        CalendarScaleProperty {
            other_params: params,
            value,
        },
    ))
}

pub fn prop_method<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], MethodProperty<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (_, params, _, value, _)) = tuple((
        tag_no_case("METHOD"),
        cut(other_params),
        char(':'),
        iana_token,
        tag("\r\n"),
    ))(input)?;

    Ok((
        input,
        MethodProperty {
            other_params: params,
            value,
        },
    ))
}

pub fn prop_x<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], XProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (name, params, _, value, _)) =
        tuple((x_name, cut(params), char(':'), value, tag("\r\n")))(input)?;

    Ok((
        input,
        XProperty {
            name,
            params,
            value,
        },
    ))
}

pub fn prop_iana<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], IanaProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (name, params, _, value, _)) = tuple((
        verify(iana_token, |t: &[u8]| {
            // Not ideal, but in order to avoid IANA names colliding with ical structure, filter these values out
            t != b"BEGIN" && t != b"END"
        }),
        cut(params),
        char(':'),
        value,
        tag("\r\n"),
    ))(input)?;

    Ok((
        input,
        IanaProperty {
            name,
            params,
            value,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::param::{ParamValue, Value};
    use crate::test_utils::check_rem;

    #[test]
    fn product_id_property() {
        let (rem, prop) =
            prop_product_id::<Error>(b"PRODID:-//ABC Corporation//NONSGML My Product//EN\r\n;")
                .unwrap();
        check_rem(rem, 1);
        assert!(prop.other_params.is_empty());
        assert_eq!(prop.value, b"-//ABC Corporation//NONSGML My Product//EN");
    }

    #[test]
    fn product_id_property_with_params() {
        let (rem, prop) = prop_product_id::<Error>(
            b"PRODID;x-prop=val:-//ABC Corporation//NONSGML My Product//EN\r\n;",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(prop.other_params.len(), 1);
        assert_eq!(
            prop.other_params[0].value,
            ParamValue::Others {
                name: b"x-prop",
                values: vec![b"val"]
            }
        );
        assert_eq!(prop.value, b"-//ABC Corporation//NONSGML My Product//EN");
    }

    #[test]
    fn version_property() {
        let input = b"VERSION:2.0\r\n;";
        let (rem, prop) = prop_version::<Error>(input).unwrap();
        check_rem(rem, 1);
        assert!(prop.other_params.is_empty());
        assert_eq!(prop.min_version, None);
        assert_eq!(prop.max_version, b"2.0");
    }

    #[test]
    fn version_property_with_param() {
        let input = b"VERSION;x-prop=val:2.0\r\n;";
        let (rem, prop) = prop_version::<Error>(input).unwrap();
        check_rem(rem, 1);
        assert_eq!(prop.other_params.len(), 1);
        assert_eq!(
            prop.other_params[0].value,
            ParamValue::Others {
                name: b"x-prop",
                values: vec![b"val"]
            }
        );
        assert_eq!(prop.min_version, None);
        assert_eq!(prop.max_version, b"2.0");
    }

    #[test]
    fn version_property_with_newer_version() {
        let input = b"VERSION:3.1\r\n;";
        let (rem, prop) = prop_version::<Error>(input).unwrap();
        check_rem(rem, 1);
        assert!(prop.other_params.is_empty());
        assert_eq!(prop.min_version, None);
        assert_eq!(prop.max_version, b"3.1");
    }

    #[test]
    fn version_property_with_version_range() {
        let input = b"VERSION:3.2;3.5\r\n;";
        let (rem, prop) = prop_version::<Error>(input).unwrap();
        check_rem(rem, 1);
        assert!(prop.other_params.is_empty());
        assert_eq!(prop.min_version.unwrap(), b"3.2");
        assert_eq!(prop.max_version, b"3.5");
    }

    #[test]
    fn cal_scale() {
        let input = b"CALSCALE:GREGORIAN\r\n;";
        let (rem, prop) = prop_calendar_scale::<Error>(input).unwrap();
        check_rem(rem, 1);
        assert!(prop.other_params.is_empty());
        assert_eq!(prop.value, b"GREGORIAN");
    }

    #[test]
    fn cal_scale_with_param() {
        let input = b"CALSCALE;x-prop=val:GREGORIAN\r\n;";
        let (rem, prop) = prop_calendar_scale::<Error>(input).unwrap();
        check_rem(rem, 1);
        assert_eq!(prop.other_params.len(), 1);
        assert_eq!(
            prop.other_params[0].value,
            ParamValue::Others {
                name: b"x-prop",
                values: vec![b"val"]
            }
        );
        assert_eq!(prop.value, b"GREGORIAN");
    }

    #[test]
    fn method() {
        let input = b"METHOD:REQUEST\r\n;";
        let (rem, prop) = prop_method::<Error>(input).unwrap();
        check_rem(rem, 1);
        assert!(prop.other_params.is_empty());
        assert_eq!(prop.value, b"REQUEST");
    }

    #[test]
    fn method_with_param() {
        let input = b"METHOD;x-prop=val:REQUEST\r\n;";
        let (rem, prop) = prop_method::<Error>(input).unwrap();
        check_rem(rem, 1);
        assert_eq!(prop.other_params.len(), 1);
        assert_eq!(
            prop.other_params[0].value,
            ParamValue::Others {
                name: b"x-prop",
                values: vec![b"val"]
            }
        );
        assert_eq!(prop.value, b"REQUEST");
    }

    #[test]
    fn x_prop() {
        let input =
            b"X-ABC-MMSUBJ;VALUE=URI;FMTTYPE=audio/basic:http://www.example.org/mysubj.au\r\n;";
        let (rem, prop) = prop_x::<Error>(input).unwrap();
        check_rem(rem, 1);
        assert_eq!(prop.name, b"X-ABC-MMSUBJ");
        assert_eq!(prop.params.len(), 2);
        assert_eq!(
            prop.params[0].value,
            ParamValue::ValueType { value: Value::Uri }
        );
        assert_eq!(
            prop.params[1].value,
            ParamValue::FormatType {
                type_name: "audio".to_string(),
                sub_type_name: "basic".to_string()
            }
        );
        assert_eq!(prop.value, b"http://www.example.org/mysubj.au");
    }

    #[test]
    fn iana_prop() {
        let input = b"DRESSCODE:CASUAL\r\n;";
        let (rem, prop) = prop_iana::<Error>(input).unwrap();
        check_rem(rem, 1);
        assert_eq!(prop.name, b"DRESSCODE");
        assert!(prop.params.is_empty());
        assert_eq!(prop.value, b"CASUAL");
    }

    #[test]
    fn iana_prop_with_params() {
        let input = b"NON-SMOKING;VALUE=BOOLEAN:TRUE\r\n;";
        let (rem, prop) = prop_iana::<Error>(input).unwrap();
        check_rem(rem, 1);
        assert_eq!(prop.name, b"NON-SMOKING");
        assert_eq!(prop.params.len(), 1);
        assert_eq!(
            prop.params[0].value,
            ParamValue::ValueType {
                value: Value::Boolean
            }
        );
        assert_eq!(prop.value, b"TRUE");
    }
}
