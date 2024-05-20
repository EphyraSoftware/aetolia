use crate::parser::object::types::{CalendarComponent, CalendarProperty, ICalendar};
use crate::parser::{iana_token, parse_line_content, x_name, Error, InnerError};
use nom::branch::alt;
use nom::bytes::streaming::tag;
use nom::character::streaming::crlf;
use nom::combinator::eof;
use nom::multi::{many0, many1};
use nom::sequence::tuple;
use nom::IResult;

mod types;

pub fn ical_stream(input: &[u8]) -> IResult<&[u8], Vec<ICalendar>, Error> {
    many1(ical_object)(input)
}

pub fn ical_object(input: &[u8]) -> IResult<&[u8], ICalendar, Error> {
    let (input, (_, body, _, _)) = tuple((
        tag("BEGIN:VCALENDAR\r\n"),
        ical_body,
        tag("END:VCALENDAR\r\n"),
        eof,
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
    todo!()
}

fn component(input: &[u8]) -> IResult<&[u8], CalendarComponent, Error> {
    alt((iana_comp, x_comp))(input)
}

fn iana_comp(input: &[u8]) -> IResult<&[u8], CalendarComponent, Error> {
    let (input, (_, name, _, lines, _, end_name, _)) = tuple((
        tag("BEGIN:"),
        iana_token,
        crlf,
        many1(parse_line_content),
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
        many1(parse_line_content),
        tag("END:"),
        x_name,
        tag("\r\n"),
    ))(input)?;

    if name != end_name {
        return Err(nom::Err::Error(Error::new(
            input,
            InnerError::MismatchedComponentEnd(name.to_vec(), end_name.to_vec()),
        )));
    }

    Ok((input, CalendarComponent::XComp { name, lines }))
}
