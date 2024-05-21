use crate::parser::object::types::{CalendarComponent, CalendarProperty, ICalendar};
use crate::parser::property::{
    prop_calendar_scale, prop_iana, prop_method, prop_product_id, prop_version, prop_x,
};
use crate::parser::{content_line, iana_token, x_name, Error, InnerError};
use nom::branch::alt;
use nom::bytes::streaming::tag;
use nom::character::streaming::crlf;
use nom::combinator::{eof, verify};
use nom::multi::{many0, many1};
use nom::sequence::tuple;
use nom::IResult;
use nom::Parser;

mod types;

pub fn ical_stream(mut input: &[u8]) -> IResult<&[u8], Vec<ICalendar>, Error> {
    let mut out = Vec::new();

    loop {
        if eof::<_, Error>(input).is_ok() {
            break;
        }

        let (i, ical) = ical_object(input)?;
        out.push(ical);

        input = i;
    }

    Ok((input, out))
}

pub fn ical_object(input: &[u8]) -> IResult<&[u8], ICalendar, Error> {
    let (input, (_, body, _)) = tuple((
        tag("BEGIN:VCALENDAR\r\n"),
        ical_body,
        tag("END:VCALENDAR\r\n"),
    ))(input)?;

    Ok((input, body))
}

fn ical_body(input: &[u8]) -> IResult<&[u8], ICalendar, Error> {
    let (input, (properties, components)) = tuple((many0(ical_cal_prop), many1(component)))(input)?;

    Ok((
        input,
        ICalendar {
            properties,
            components,
        },
    ))
}

fn ical_cal_prop(input: &[u8]) -> IResult<&[u8], CalendarProperty, Error> {
    alt((
        prop_product_id.map(CalendarProperty::ProductId),
        prop_version.map(CalendarProperty::Version),
        prop_calendar_scale.map(CalendarProperty::CalScale),
        prop_method.map(CalendarProperty::Method),
        prop_x.map(CalendarProperty::XProp),
        prop_iana.map(CalendarProperty::IanaProp),
    ))
    .parse(input)
}

fn component(input: &[u8]) -> IResult<&[u8], CalendarComponent, Error> {
    alt((x_comp, iana_comp))(input)
}

fn iana_comp(input: &[u8]) -> IResult<&[u8], CalendarComponent, Error> {
    let (input, (_, name, _, lines, _, end_name, _)) = tuple((
        tag("BEGIN:"),
        iana_token,
        crlf,
        many1(content_line),
        tag("END:"),
        iana_token,
        tag("\r\n"),
    ))(input)?;

    if name != end_name {
        return Err(nom::Err::Error(Error::new(
            input,
            InnerError::MismatchedComponentEnd(name.to_vec(), end_name.to_vec()),
        )));
    }

    Ok((input, CalendarComponent::IanaComp { name, lines }))
}

fn x_comp(input: &[u8]) -> IResult<&[u8], CalendarComponent, Error> {
    let (input, (_, name, _, lines, _, end_name, _)) = tuple((
        tag("BEGIN:"),
        x_name,
        crlf,
        many1(verify(content_line, |cl| cl.property_name != b"END")),
        tag("END:"),
        x_name,
        crlf,
    ))(input)?;

    if name != end_name {
        return Err(nom::Err::Error(Error::new(
            input,
            InnerError::MismatchedComponentEnd(name.to_vec(), end_name.to_vec()),
        )));
    }

    Ok((input, CalendarComponent::XComp { name, lines }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::property::types::VersionProperty;
    use crate::test_utils::check_rem;

    #[test]
    fn minimal_ical_stream_test() {
        let input = b"BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:test\r\nBEGIN:x-com\r\nx-prop:I'm a property\r\nEND:x-com\r\nEND:VCALENDAR\r\n";
        let (rem, ical) = ical_stream(input).unwrap();
        check_rem(rem, 0);
        assert_eq!(ical.len(), 1);
        assert_eq!(ical[0].properties.len(), 2);
        assert_eq!(
            ical[0].properties[0],
            CalendarProperty::Version(VersionProperty {
                other_params: vec![],
                min_version: None,
                max_version: b"2.0",
            })
        );
        assert_eq!(ical[0].components.len(), 1);
    }
}
