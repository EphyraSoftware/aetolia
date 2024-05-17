use crate::parser::{Error, InnerError};
use nom::branch::alt;
use nom::bytes::streaming::{tag, take_while, take_while1, take_while_m_n};
use nom::character::streaming::char;
use nom::character::{is_alphabetic, is_digit};
use nom::combinator::{map_res, opt, recognize, verify};
use nom::error::ParseError;
use nom::multi::{fold_many0, fold_many1, many0, many1, many_m_n, separated_list0};
use nom::sequence::tuple;
use nom::{IResult, InputIter, InputLength, InputTake, Parser};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::num::NonZeroUsize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpAddr {
    V4(Ipv4Addr),
    V6(Ipv6Addr),
    VFuture(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Host {
    IpAddr(IpAddr),
    RegName(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Authority {
    pub user_info: Option<Vec<u8>>,
    pub host: Host,
    pub port: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uri<'a> {
    pub scheme: &'a [u8],
    pub authority: Authority,
    pub path: Vec<u8>,
    pub query: Option<&'a [u8]>,
    pub fragment: Option<&'a [u8]>,
}

pub fn param_value_uri(input: &[u8]) -> IResult<&[u8], Uri, Error> {
    let (input, (scheme, _, authority, path, query, fragment)) = tuple((
        scheme,
        tag("://"),
        authority,
        opt(alt((path_absolute_empty, path_absolute, path_rootless))),
        opt(tuple((char('?'), query_or_fragment)).map(|(_, v)| v)),
        opt(tuple((char('#'), query_or_fragment)).map(|(_, v)| v)),
    ))(input)?;

    Ok((
        input,
        Uri {
            scheme,
            authority,
            path: path.unwrap_or_default().to_vec(),
            query,
            fragment,
        },
    ))
}

#[inline]
const fn is_scheme_char(b: u8) -> bool {
    matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'+' | b'-' | b'.')
}

fn scheme(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    verify(take_while1(is_scheme_char), |sch: &[u8]| {
        is_alphabetic(sch[0])
    })(input)
}

#[inline]
const fn is_hex_digit_upper(b: u8) -> bool {
    matches!(b, b'0'..=b'9' | b'A'..=b'F')
}

#[inline]
const fn is_hex_digit(b: u8) -> bool {
    matches!(b, b'0'..=b'9' | b'A'..=b'F' | b'a'..=b'f')
}

#[inline]
const fn is_unreserved(b: u8) -> bool {
    matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~')
}

fn pct_encoded(input: &[u8]) -> IResult<&[u8], Vec<u8>, Error> {
    tuple((
        char('%'),
        take_while_m_n(2, 2, is_hex_digit_upper).map(|v| {
            // TODO do without a dep here?
            hex::decode(v).unwrap()
        }),
    ))
    .map(|(_, v)| v)
    .parse(input)
}

#[inline]
const fn is_sub_delim(b: u8) -> bool {
    matches!(
        b,
        b'!' | b'$' | b'&' | b'\'' | b'(' | b')' | b'*' | b'+' | b',' | b';' | b'='
    )
}

fn authority(input: &[u8]) -> IResult<&[u8], Authority, Error> {
    tuple((
        opt(tuple((user_info, char('@'))).map(|(u, _)| u)),
        host,
        opt(tuple((char(':'), port)).map(|(_, p)| p)),
    ))
    .map(|(user_info, host, port)|
        Authority {
            user_info,
            host,
            port,
        }
    )
    .parse(input)
}

fn port(input: &[u8]) -> IResult<&[u8], u16, Error> {
    map_res(take_while(is_digit), |c| {
        std::str::from_utf8(c)
            .map_err(|e| {
                nom::Err::Error(Error::new(
                    input,
                    InnerError::EncodingError("Recur month list".to_string(), e),
                ))
            })?
            .parse::<u16>()
            .map_err(|_| nom::Err::Error(Error::new(input, InnerError::InvalidPort)))
    })
    .parse(input)
}

fn user_info(input: &[u8]) -> IResult<&[u8], Vec<u8>, Error> {
    fold_many1(
        alt((
            single(is_unreserved).map(|c| vec![c]),
            pct_encoded,
            single(is_sub_delim).map(|c| vec![c]),
            tag(":").map(|c: &[u8]| c.to_vec()),
        )),
        Vec::new,
        |mut acc, item| {
            acc.extend(item);
            acc
        },
    )(input)
}

fn host(input: &[u8]) -> IResult<&[u8], Host, Error> {
    alt((
        ip_literal.map(Host::IpAddr),
        ip_v4_addr
            .map(|ip| IpAddr::V4(Ipv4Addr::from(ip)))
            .map(Host::IpAddr),
        reg_name.map(Host::RegName),
    ))(input)
}

fn ip_literal(input: &[u8]) -> IResult<&[u8], IpAddr, Error> {
    tuple((
        tag("["),
        alt((
            ip_v6_addr.map(IpAddr::V6),
            ip_v_future_addr.map(|ip| IpAddr::VFuture(ip.to_vec())),
        )),
        tag("]"),
    ))
    .map(|(_, v, _)| v)
    .parse(input)
}

fn ip_v_future_addr(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    recognize(tuple((
        char('v').map(|a| a as u8),
        take_while1(is_hex_digit),
        char('.'),
        many1(alt((
            single(is_unreserved),
            single(is_sub_delim),
            char(':').map(|c| c as u8),
        ))),
    )))(input)
}

fn ip_v6_addr(input: &[u8]) -> IResult<&[u8], Ipv6Addr, Error> {
    let (input, prefix_parts) = separated_list0(char(':'), h_16)(input)?;

    println!("Took prefix parts: {:?}", prefix_parts);

    let prefix_len = prefix_parts.len();

    Ok(match prefix_len {
        7 => {
            let (input, _) = tag("::")(input)?;
            (
                input,
                Ipv6Addr::from([
                    prefix_parts[0][0],
                    prefix_parts[0][1],
                    prefix_parts[1][0],
                    prefix_parts[1][1],
                    prefix_parts[2][0],
                    prefix_parts[2][1],
                    prefix_parts[3][0],
                    prefix_parts[3][1],
                    prefix_parts[4][0],
                    prefix_parts[4][1],
                    prefix_parts[5][0],
                    prefix_parts[5][1],
                    prefix_parts[6][0],
                    prefix_parts[6][1],
                    0,
                    0,
                ]),
            )
        }
        6 => {
            let (input, (_, last)) = tuple((tag("::"), h_16))(input)?;

            (
                input,
                Ipv6Addr::from([
                    prefix_parts[0][0],
                    prefix_parts[0][1],
                    prefix_parts[1][0],
                    prefix_parts[1][1],
                    prefix_parts[2][0],
                    prefix_parts[2][1],
                    prefix_parts[3][0],
                    prefix_parts[3][1],
                    prefix_parts[4][0],
                    prefix_parts[4][1],
                    prefix_parts[5][0],
                    prefix_parts[5][1],
                    0,
                    0,
                    last[0],
                    last[1],
                ]),
            )
        }
        5 => {
            let (input, (_, last)) = tuple((tag("::"), ls_32))(input)?;

            (
                input,
                Ipv6Addr::from([
                    prefix_parts[0][0],
                    prefix_parts[0][1],
                    prefix_parts[1][0],
                    prefix_parts[1][1],
                    prefix_parts[2][0],
                    prefix_parts[2][1],
                    prefix_parts[3][0],
                    prefix_parts[3][1],
                    prefix_parts[4][0],
                    prefix_parts[4][1],
                    0,
                    0,
                    last[0],
                    last[1],
                    last[2],
                    last[3],
                ]),
            )
        }
        4 => {
            let (input, (_, lead, _, last)) = tuple((tag("::"), h_16, char(':'), ls_32))(input)?;

            (
                input,
                Ipv6Addr::from([
                    prefix_parts[0][0],
                    prefix_parts[0][1],
                    prefix_parts[1][0],
                    prefix_parts[1][1],
                    prefix_parts[2][0],
                    prefix_parts[2][1],
                    prefix_parts[3][0],
                    prefix_parts[3][1],
                    0,
                    0,
                    lead[0],
                    lead[1],
                    last[0],
                    last[1],
                    last[2],
                    last[3],
                ]),
            )
        }
        3 => {
            let (input, (_, lead, _, last)) = tuple((
                tag("::"),
                many_m_n(2, 2, tuple((h_16, char(':'))))
                    .map(|v| v.into_iter().map(|(v, _)| v).collect::<Vec<_>>()),
                char(':'),
                ls_32,
            ))(input)?;

            (
                input,
                Ipv6Addr::from([
                    prefix_parts[0][0],
                    prefix_parts[0][1],
                    prefix_parts[1][0],
                    prefix_parts[1][1],
                    prefix_parts[2][0],
                    prefix_parts[2][1],
                    0,
                    0,
                    lead[0][0],
                    lead[0][1],
                    lead[1][0],
                    lead[1][1],
                    last[0],
                    last[1],
                    last[2],
                    last[3],
                ]),
            )
        }
        2 => {
            let (input, (_, lead, _, last)) = tuple((
                tag("::"),
                many_m_n(3, 3, tuple((h_16, char(':'))))
                    .map(|v| v.into_iter().map(|(v, _)| v).collect::<Vec<_>>()),
                char(':'),
                ls_32,
            ))(input)?;

            (
                input,
                Ipv6Addr::from([
                    prefix_parts[0][0],
                    prefix_parts[0][1],
                    prefix_parts[1][0],
                    prefix_parts[1][1],
                    0,
                    0,
                    lead[0][0],
                    lead[0][1],
                    lead[1][0],
                    lead[1][1],
                    lead[2][0],
                    lead[2][1],
                    last[0],
                    last[1],
                    last[2],
                    last[3],
                ]),
            )
        }
        1 => {
            let (input, (_, lead, _, last)) = tuple((
                tag("::"),
                many_m_n(4, 4, tuple((h_16, char(':'))))
                    .map(|v| v.into_iter().map(|(v, _)| v).collect::<Vec<_>>()),
                char(':'),
                ls_32,
            ))(input)?;

            (
                input,
                Ipv6Addr::from([
                    prefix_parts[0][0],
                    prefix_parts[0][1],
                    0,
                    0,
                    lead[0][0],
                    lead[0][1],
                    lead[1][0],
                    lead[1][1],
                    lead[2][0],
                    lead[2][1],
                    lead[3][0],
                    lead[3][1],
                    last[0],
                    last[1],
                    last[2],
                    last[3],
                ]),
            )
        }
        0 => {
            let (input, v) = opt(tag("::"))(input)?;

            match v {
                Some(_) => {
                    let (input, (lead, _, last)) = tuple((
                        many_m_n(5, 5, tuple((h_16, char(':'))))
                            .map(|v| v.into_iter().map(|(v, _)| v).collect::<Vec<_>>()),
                        char(':'),
                        ls_32,
                    ))(input)?;

                    (
                        input,
                        Ipv6Addr::from([
                            0, 0, lead[0][0], lead[0][1], lead[1][0], lead[1][1], lead[2][0],
                            lead[2][1], lead[3][0], lead[3][1], lead[4][0], lead[4][1], last[0],
                            last[1], last[2], last[3],
                        ]),
                    )
                }
                None => {
                    let (input, (lead, _, last)) = tuple((
                        many_m_n(6, 6, tuple((h_16, char(':'))))
                            .map(|v| v.into_iter().map(|(v, _)| v).collect::<Vec<_>>()),
                        char(':'),
                        ls_32,
                    ))(input)?;

                    (
                        input,
                        Ipv6Addr::from([
                            lead[0][0], lead[0][1], lead[1][0], lead[1][1], lead[2][0], lead[2][1],
                            lead[3][0], lead[3][1], lead[4][0], lead[4][1], lead[5][0], lead[5][1],
                            last[0], last[1], last[2], last[3],
                        ]),
                    )
                }
            }
        }
        _ => {
            return Err(nom::Err::Error(Error::new(input, InnerError::InvalidIpv6)));
        }
    })
}

fn h_16(input: &[u8]) -> IResult<&[u8], [u8; 2], Error> {
    take_while_m_n(1, 4, is_hex_digit).map(|c: &[u8]| {
        let mut src = c.to_vec();
        while src.len() < 4 {
            src.insert(0, 0);
        }
        let mut dst = [0, 0];
        println!("Hex decode: {:?}", src);
        hex::decode_to_slice(src, &mut dst).unwrap();
        dst
    }).parse(input)
}

fn ls_32(input: &[u8]) -> IResult<&[u8], Vec<u8>, Error> {
    alt((
        tuple((h_16, char(':'), h_16)).map(|(a, _, b)| {
            let mut r = Vec::with_capacity(4);
            r.extend(hex::decode(a).unwrap());
            r.extend(hex::decode(b).unwrap());
            r
        }),
        ip_v4_addr.map(|a| a.to_vec()),
    ))(input)
}

fn ip_v4_addr(input: &[u8]) -> IResult<&[u8], [u8; 4], Error> {
    tuple((
        dec_octet,
        char('.'),
        dec_octet,
        char('.'),
        dec_octet,
        char('.'),
        dec_octet,
    ))
    .map(|(a, _, b, _, c, _, d)| [a, b, c, d])
    .parse(input)
}

fn dec_octet(input: &[u8]) -> IResult<&[u8], u8, Error> {
    map_res(
        verify(take_while_m_n(1, 3, is_digit), |b: &[u8]| {
            // May not have a 0 prefix
            if b.len() == 2 {
                b[0] != b'0'
            } else if b.len() == 3 {
                if b[0] == b'0' && b[1] == b'0' {
                    false
                } else {
                    b[0] != b'0'
                }
            } else {
                true
            }
        }),
        |b| {
            std::str::from_utf8(b)
                .unwrap()
                .parse::<u8>()
                .map_err(|_| nom::Err::Error(Error::new(input, InnerError::InvalidOctet)))
        },
    )(input)
}

fn reg_name(input: &[u8]) -> IResult<&[u8], Vec<u8>, Error> {
    fold_many0(
        alt((
            single(is_unreserved).map(|c| vec![c]),
            pct_encoded,
            single(is_sub_delim).map(|c| vec![c]),
        )),
        Vec::new,
        |mut acc, item| {
            acc.extend(item);
            acc
        },
    )(input)
}

/// Streaming, single character matching the predicate
fn single<F, Input, Output, Error: ParseError<Input>>(
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

fn path_absolute_empty(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    recognize(many0(tuple((char('/'), segment))))(input)
}

fn path_absolute(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    recognize(tuple((segment_nz, path_absolute_empty)))(input)
}

fn path_rootless(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    recognize(tuple((segment_nz, many0(tuple((char('/'), segment))))))(input)
}

fn segment(input: &[u8]) -> IResult<&[u8], Vec<u8>, Error> {
    fold_many0(p_char, Vec::new, |mut acc, item| {
        acc.extend(item);
        acc
    })(input)
}

fn segment_nz(input: &[u8]) -> IResult<&[u8], Vec<u8>, Error> {
    fold_many1(p_char, Vec::new, |mut acc, item| {
        acc.extend(item);
        acc
    })(input)
}

fn query_or_fragment(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
    recognize(many0(alt((
        p_char,
        tag("/").map(|c: &[u8]| c.to_vec()),
        tag("?").map(|c: &[u8]| c.to_vec()),
    ))))(input)
}

fn p_char(input: &[u8]) -> IResult<&[u8], Vec<u8>, Error> {
    alt((
        single(is_unreserved).map(|c| vec![c]),
        pct_encoded,
        single(is_sub_delim).map(|c| vec![c]),
        tag(":").map(|c: &[u8]| c.to_vec()),
        tag("@").map(|c: &[u8]| c.to_vec()),
    ))(input)
}

// ftp://ftp.is.co.za/rfc/rfc1808.txt
//
//       http://www.ietf.org/rfc/rfc2396.txt
//
//       ldap://[2001:db8::7]/c=GB?objectClass?one
//
//       mailto:John.Doe@example.com
//
//       news:comp.infosystems.www.servers.unix
//
//       tel:+1-816-555-1212
//
//       telnet://192.0.2.16:80/
//
//       urn:oasis:names:specification:docbook:dtd:xml:4.1.2

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::check_rem;

    #[test]
    fn ftp() {
        let (input, uri) = param_value_uri(b"ftp://ftp.is.co.za/rfc/rfc1808.txt`").unwrap();
        check_rem(input, 1);
        assert_eq!(uri.scheme, b"ftp");
        assert_eq!(uri.authority.host, Host::RegName(b"ftp.is.co.za".to_vec()));
        assert_eq!(uri.path, b"/rfc/rfc1808.txt");
    }

    #[test]
    fn http() {
        let (input, uri) = param_value_uri(b"http://www.ietf.org/rfc/rfc2396.txt`").unwrap();
        check_rem(input, 1);
        assert_eq!(uri.scheme, b"http");
        assert_eq!(uri.authority.host, Host::RegName(b"www.ietf.org".to_vec()));
        assert_eq!(uri.path, b"/rfc/rfc2396.txt");
    }

    #[test]
    fn ip_v6() {
        let (input, ipv6) = ip_v6_addr(b"2001:db8::7`").unwrap();
        check_rem(input, 1);
    }

    #[test]
    fn ldap() {
        let (input, uri) = param_value_uri(b"ldap://[2001:db8::7]/c=GB?objectClass?one`").unwrap();
        check_rem(input, 1);
        assert_eq!(uri.scheme, b"ldap");
    }
}
