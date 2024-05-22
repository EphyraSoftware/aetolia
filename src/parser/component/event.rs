use nom::branch::alt;
use nom::bytes::streaming::tag;
use nom::{IResult, Parser};
use nom::multi::many0;
use nom::sequence::tuple;
use crate::parser::Error;
use crate::parser::object::types::{CalendarComponent, ComponentProperty};
use crate::parser::property::{prop_date_time_stamp, prop_date_time_start, prop_iana, prop_x};



pub fn component_event(input: &[u8]) -> IResult<&[u8], CalendarComponent, Error> {
    let (input, (_, properties, _)) = tuple((
        tag("BEGIN:VEVENT\r\n"),
        many0(alt((
            prop_date_time_start.map(ComponentProperty::DateTimeStart),
            prop_date_time_stamp.map(ComponentProperty::DateTimeStamp),
            prop_x.map(ComponentProperty::XProp),
            prop_iana.map(ComponentProperty::IanaProp),
            ))),
        tag("END:VEVENT\r\n"),
    ))(input)?;

    Ok((
        input,
        CalendarComponent::Event {
            properties,
        },
    ))
}
