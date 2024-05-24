use crate::parser::Error;
use nom::bytes::streaming::{tag, take_until};
use nom::character::streaming::one_of;
use nom::sequence::tuple;
use nom::IResult;

pub fn content_line_first_pass(mut input: &[u8]) -> IResult<&[u8], Vec<u8>, Error> {
    let mut out = Vec::new();

    loop {
        let (i, o) = take_until("\r\n")(input)?;
        out.extend_from_slice(o);
        input = i;

        match tuple((tag("\r\n"), one_of(" \t")))(input) {
            Ok((i, _)) => {
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
        let (rem, line) = content_line_first_pass(
            b"DESCRIP\r\n TION;BRE\r\n NT\r\n =\r\n sent\r\n :\r\n Meeting \"\r\n A\"\r\n;",
        )
        .unwrap();
        check_rem(rem, 1);
        assert_eq!(line, b"DESCRIPTION;BRENT=sent:Meeting \"A\"\r\n");
    }
}
