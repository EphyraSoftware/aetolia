use crate::parser::component::{
    component_event, component_free_busy, component_journal, component_timezone, component_todo,
};
use crate::parser::property::{
    prop_calendar_scale, prop_iana, prop_method, prop_product_id, prop_version, prop_x,
};
use crate::parser::types::CalendarComponent;
use crate::parser::types::CalendarProperty;
use crate::parser::types::ICalendar;
use crate::parser::{content_line, iana_token, x_name, Error, InnerError};
use nom::branch::alt;
use nom::bytes::streaming::tag;
use nom::character::streaming::crlf;
use nom::combinator::{cut, eof, verify};
use nom::error::ParseError;
use nom::multi::{many0, many1};
use nom::sequence::tuple;
use nom::IResult;
use nom::Parser;

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
        prop_calendar_scale.map(CalendarProperty::CalendarScale),
        prop_method.map(CalendarProperty::Method),
        prop_x.map(CalendarProperty::XProperty),
        prop_iana.map(CalendarProperty::IanaProperty),
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
        cut(many1(verify(content_line, |line| {
            line.property_name != "END".as_bytes()
        }))),
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
        cut(many1(verify(content_line, |cl| cl.property_name != b"END"))),
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
trait ReprStr {
    fn repr_str(&self) -> &str;
}

#[cfg(test)]
impl ReprStr for &[u8] {
    fn repr_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self) }
    }
}

// Borrowed from `nom` and modified (somewhat poorly!) to work with byte arrays rather than strings.
#[cfg(test)]
fn convert_error_mod<I: ReprStr>(input: I, e: nom::error::VerboseError<I>) -> String {
    use nom::error::VerboseErrorKind;
    use nom::Offset;
    use std::fmt::Write;

    let mut result = String::new();

    let input = input.repr_str();

    for (i, (substring, kind)) in e.errors.iter().enumerate() {
        let substring = substring.repr_str();
        let offset = input.offset(substring);

        if input.is_empty() {
            match kind {
                VerboseErrorKind::Char(c) => {
                    write!(&mut result, "{}: expected '{}', got empty input\n\n", i, c)
                }
                VerboseErrorKind::Context(s) => {
                    write!(&mut result, "{}: in {}, got empty input\n\n", i, s)
                }
                VerboseErrorKind::Nom(e) => {
                    write!(&mut result, "{}: in {:?}, got empty input\n\n", i, e)
                }
            }
        } else {
            let prefix = &input.as_bytes()[..offset];

            // Count the number of newlines in the first `offset` bytes of input
            let line_number = prefix.iter().filter(|&&b| b == b'\n').count() + 1;

            // Find the line that includes the subslice:
            // Find the *last* newline before the substring starts
            let line_begin = prefix
                .iter()
                .rev()
                .position(|&b| b == b'\n')
                .map(|pos| offset - pos)
                .unwrap_or(0);

            // Find the full line after that newline
            let line = input[line_begin..]
                .lines()
                .next()
                .unwrap_or(&input[line_begin..])
                .trim_end();

            // The (1-indexed) column number is the offset of our substring into that line
            let column_number = line.offset(substring) + 1;

            match kind {
                VerboseErrorKind::Char(c) => {
                    if let Some(actual) = substring.chars().next() {
                        write!(
                            &mut result,
                            "{i}: at line {line_number}:\n\
               {line}\n\
               {caret:>column$}\n\
               expected '{expected}', found {actual}\n\n",
                            i = i,
                            line_number = line_number,
                            line = line,
                            caret = '^',
                            column = column_number,
                            expected = c,
                            actual = actual,
                        )
                    } else {
                        write!(
                            &mut result,
                            "{i}: at line {line_number}:\n\
               {line}\n\
               {caret:>column$}\n\
               expected '{expected}', got end of input\n\n",
                            i = i,
                            line_number = line_number,
                            line = line,
                            caret = '^',
                            column = column_number,
                            expected = c,
                        )
                    }
                }
                VerboseErrorKind::Context(s) => write!(
                    &mut result,
                    "{i}: at line {line_number}, in {context}:\n\
             {line}\n\
             {caret:>column$}\n\n",
                    i = i,
                    line_number = line_number,
                    context = s,
                    line = line,
                    caret = '^',
                    column = column_number,
                ),
                VerboseErrorKind::Nom(e) => write!(
                    &mut result,
                    "{i}: at line {line_number}, in {nom_err:?}:\n\
             {line}\n\
             {caret:>column$}\n\n",
                    i = i,
                    line_number = line_number,
                    nom_err = e,
                    line = line,
                    caret = '^',
                    column = column_number,
                ),
            }
        }
        // Because `write!` to a `String` is infallible, this `unwrap` is fine.
        .unwrap();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::clear_errors;
    use crate::parser::first_pass::content_line_first_pass;
    use crate::parser::types::VersionProperty;
    use crate::test_utils::check_rem;
    use nom::combinator::complete;
    use nom::error::VerboseError;

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

    #[test]
    #[ignore = "Requires a real file"]
    fn real_file() {
        let input = std::fs::read_to_string("test_data.ics").unwrap();

        let (input, first) = content_line_first_pass::<Error>(input.as_bytes()).unwrap();
        check_rem(input, 0);

        let r = complete::<_, _, VerboseError<&[u8]>, _>(ical_stream).parse(&first);
        match r {
            Err(nom::Err::Error(e) | nom::Err::Failure(e)) => {
                println!("fail:\n\n {}", convert_error_mod(first.as_slice(), e));
            }
            Ok((rem, ical)) => {
                println!("Got an OK result");
                check_rem(rem, 0);
                println!("Calendars: {:?}", ical.len());
                println!("Components: {:?}", ical[0].components.len());
            }
            e => {
                panic!("unexpected result: {:?}", e)
            }
        }

        unsafe { clear_errors() };
    }
}
