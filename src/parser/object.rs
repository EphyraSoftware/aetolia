use crate::parser::component::{
    component_event, component_free_busy, component_journal, component_timezone, component_todo,
};
use crate::parser::object::types::{CalendarComponent, CalendarProperty, ICalendar};
use crate::parser::property::{
    prop_calendar_scale, prop_iana, prop_method, prop_product_id, prop_version, prop_x,
};
use crate::parser::{content_line, iana_token, x_name, Error, InnerError};
use nom::branch::alt;
use nom::bytes::streaming::tag;
use nom::character::streaming::crlf;
use nom::combinator::{eof, verify};
use nom::error::ParseError;
use nom::multi::{many0, many1};
use nom::sequence::tuple;
use nom::IResult;
use nom::Parser;

pub mod types;

pub fn ical_stream<'a, E>(mut input: &'a [u8]) -> IResult<&'a [u8], Vec<ICalendar<'a>>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
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

pub fn ical_object<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ICalendar<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (_, body, _)) = tuple((
        tag("BEGIN:VCALENDAR\r\n"),
        ical_body,
        tag("END:VCALENDAR\r\n"),
    ))(input)?;

    Ok((input, body))
}

fn ical_body<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ICalendar<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    let (input, (properties, components)) = tuple((many0(ical_cal_prop), many1(component)))(input)?;

    Ok((
        input,
        ICalendar {
            properties,
            components,
        },
    ))
}

fn ical_cal_prop<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], CalendarProperty<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
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

fn component<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], CalendarComponent<'a>, E>
where
    E: ParseError<&'a [u8]>
        + nom::error::FromExternalError<&'a [u8], nom::Err<E>>
        + From<Error<'a>>,
{
    alt((
        component_event,
        component_todo,
        component_journal,
        component_free_busy,
        component_timezone,
        x_comp,
        iana_comp,
    ))(input)
}

fn iana_comp<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], CalendarComponent<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
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
        return Err(nom::Err::Error(
            Error::new(
                input,
                InnerError::MismatchedComponentEnd(name.to_vec(), end_name.to_vec()),
            )
            .into(),
        ));
    }

    Ok((input, CalendarComponent::IanaComp { name, lines }))
}

fn x_comp<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], CalendarComponent<'a>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
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
        return Err(nom::Err::Error(
            Error::new(
                input,
                InnerError::MismatchedComponentEnd(name.to_vec(), end_name.to_vec()),
            )
            .into(),
        ));
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
        let (rem, ical) = ical_stream::<Error>(input).unwrap();
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

    // #[test]
    // fn real_file() {
    //     let input = std::fs::read_to_string("sample.ics").unwrap();
    //
    //     let (input, first) = content_line_first_pass(input.as_bytes()).unwrap();
    //     check_rem(input, 0);
    //
    //     println!("{:?}", first.len());
    //     println!("{:?}", first[0..100].to_vec());
    //     let r = ical_stream(&first);
    //     match r {
    //         Err(e) => {
    //             println!("{:?}", String::from_utf8_lossy(e.input));
    //         }
    //         _ => {}
    //     }
    //     let (rem, ical) = r.unwrap();
    //     check_rem(rem, 0);
    // }
}
