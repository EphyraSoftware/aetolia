mod recur;
pub mod types;
mod uri;
mod value;
mod value_types;

use crate::parser::param::other_params;
use crate::parser::property::types::{ProductId, VersionProperty};
use crate::parser::Error;
use crate::single;
use nom::branch::alt;
use nom::bytes::streaming::tag;
use nom::character::is_digit;
use nom::character::streaming::char;
use nom::combinator::recognize;
use nom::sequence::tuple;
use nom::{IResult, Parser};
pub use value::*;
pub use value_types::*;

pub fn prop_product_id(input: &[u8]) -> IResult<&[u8], ProductId, Error> {
    let (input, (_, params, _, value, _)) = tuple((
        tag("PRODID"),
        other_params,
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

pub fn prop_version(input: &[u8]) -> IResult<&[u8], VersionProperty, Error> {
    let (input, (_, params, _, (min_ver, max_ver), _)) = tuple((
        tag("VERSION"),
        other_params,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::param::ParamValue;
    use crate::test_utils::check_rem;

    #[test]
    fn product_id_property() {
        let (rem, prop) =
            prop_product_id(b"PRODID:-//ABC Corporation//NONSGML My Product//EN\r\n;").unwrap();
        check_rem(rem, 1);
        assert!(prop.other_params.is_empty());
        assert_eq!(prop.value, b"-//ABC Corporation//NONSGML My Product//EN");
    }

    #[test]
    fn product_id_property_with_params() {
        let (rem, prop) =
            prop_product_id(b"PRODID;x-prop=val:-//ABC Corporation//NONSGML My Product//EN\r\n;")
                .unwrap();
        check_rem(rem, 1);
        assert_eq!(prop.other_params.len(), 1);
        assert_eq!(prop.other_params[0].name, "x-prop".to_string());
        assert_eq!(
            prop.other_params[0].value,
            ParamValue::Other { value: b"val" }
        );
        assert_eq!(prop.value, b"-//ABC Corporation//NONSGML My Product//EN");
    }

    #[test]
    fn version_property() {
        let input = b"VERSION:2.0\r\n;";
        let (rem, prop) = prop_version(input).unwrap();
        check_rem(rem, 1);
        assert!(prop.other_params.is_empty());
        assert_eq!(prop.min_version, None);
        assert_eq!(prop.max_version, b"2.0");
    }

    #[test]
    fn version_property_with_param() {
        let input = b"VERSION;x-prop=val:2.0\r\n;";
        let (rem, prop) = prop_version(input).unwrap();
        check_rem(rem, 1);
        assert_eq!(prop.other_params.len(), 1);
        assert_eq!(prop.other_params[0].name, "x-prop".to_string());
        assert_eq!(
            prop.other_params[0].value,
            ParamValue::Other { value: b"val" }
        );
        assert_eq!(prop.min_version, None);
        assert_eq!(prop.max_version, b"2.0");
    }

    #[test]
    fn version_property_with_newer_version() {
        let input = b"VERSION:3.1\r\n;";
        let (rem, prop) = prop_version(input).unwrap();
        check_rem(rem, 1);
        assert!(prop.other_params.is_empty());
        assert_eq!(prop.min_version, None);
        assert_eq!(prop.max_version, b"3.1");
    }

    #[test]
    fn version_property_with_version_range() {
        let input = b"VERSION:3.2;3.5\r\n;";
        let (rem, prop) = prop_version(input).unwrap();
        check_rem(rem, 1);
        assert!(prop.other_params.is_empty());
        assert_eq!(prop.min_version.unwrap(), b"3.2");
        assert_eq!(prop.max_version, b"3.5");
    }
}
