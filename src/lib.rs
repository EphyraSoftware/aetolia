use crate::parser::Error;
use nom::branch::alt;
use nom::combinator::recognize;
use nom::error::ParseError;
use nom::sequence::tuple;
use nom::{IResult, InputIter, InputLength, InputTake};
use std::num::NonZeroUsize;

mod parser;

#[cfg(test)]
mod test_utils;

/// Streaming, single character matching the predicate
pub fn single<F, Input, Output, Error: ParseError<Input>>(
    cond: F,
) -> impl Fn(Input) -> IResult<Input, Output, Error>
where
    Input: InputIter<Item = Output> + InputLength + InputTake,
    F: Fn(<Input as InputIter>::Item) -> bool,
    Output: Copy,
{
    move |i: Input| {
        match i.iter_elements().next() {
            Some(c) if cond(c) => {
                let (input, v) = i.take_split(1);
                Ok((input, v.iter_elements().next().unwrap()))
            }
            // Closest error I can get, can't add to the Nom enum!
            Some(_) => Err(nom::Err::Error(Error::from_error_kind(
                i,
                nom::error::ErrorKind::OneOf,
            ))),
            None => Err(nom::Err::Incomplete(nom::Needed::Size(
                NonZeroUsize::new(1).unwrap(),
            ))),
        }
    }
}

fn utf8_seq(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    let (input, seq) = alt((
        // Utf-8 2-byte sequence
        recognize(tuple((
            single(|b| matches!(b, b'\xC2'..=b'\xDF')),
            single(|b| matches!(b, b'\x80'..=b'\xBF')),
        ))),
        // Utf-8 3-byte sequence
        alt((
            recognize(tuple((
                single(|b| b == b'\xE0'),
                single(|b| matches!(b, b'\xA0'..=b'\xBF')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
            ))),
            recognize(tuple((
                single(|b| matches!(b, b'\xE1'..=b'\xEC')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
            ))),
            recognize(tuple((
                single(|b| b == b'\xED'),
                single(|b| matches!(b, b'\x80'..=b'\x9F')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
            ))),
            recognize(tuple((
                single(|b| matches!(b, b'\xEE'..=b'\xEF')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
            ))),
        )),
        // Utf-8 4-byte sequence
        alt((
            recognize(tuple((
                single(|b| b == b'\xF0'),
                single(|b| matches!(b, b'\x90'..=b'\xBF')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
            ))),
            recognize(tuple((
                single(|b| matches!(b, b'\xF1'..=b'\xF3')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
            ))),
            recognize(tuple((
                single(|b| b == b'\xF4'),
                single(|b| matches!(b, b'\x80'..=b'\x8F')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
                single(|b| matches!(b, b'\x80'..=b'\xBF')),
            ))),
        )),
    ))(input)?;

    Ok((input, seq))
}
