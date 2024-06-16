use crate::parser::Error;
use nom::bytes::streaming::{tag, take_until};
use nom::character::streaming::one_of;
use nom::combinator::opt;
use nom::error::ParseError;
use nom::sequence::tuple;
use nom::IResult;

pub fn content_line_first_pass<'a, E>(mut input: &'a [u8]) -> IResult<&'a [u8], Vec<u8>, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let mut out = Vec::new();

    loop {
        let (i, o) = take_until("\r\n")(input)?;
        out.extend_from_slice(o);
        input = i;

        if input.len() == 2 {
            break;
        }

        match tuple((tag("\r\n"), opt(one_of(" \t"))))(input) {
            Ok((i, (lb, sp))) => {
                if sp.is_none() {
                    out.extend_from_slice(lb);
                }
                input = i;
            }
            Err(e) => {
                if e.is_incomplete() {
                    return Err(e);
                }

                break;
            }
        }
    }

    let (input, v) = tag("\r\n")(input)?;
    out.extend_from_slice(v);

    Ok((input, out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::check_rem;

    #[test]
    fn general_line() {
        let (rem, line) = content_line_first_pass::<Error>(
            b"DESCRIP\r\n TION;BRE\r\n NT\r\n =\r\n sent\r\n :\r\n Meeting \"\r\n A\"\r\n",
        )
        .unwrap();
        check_rem(rem, 0);
        assert_eq!(line, b"DESCRIPTION;BRENT=sent:Meeting \"A\"\r\n");
    }
}
