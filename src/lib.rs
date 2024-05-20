use nom::error::ParseError;
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
