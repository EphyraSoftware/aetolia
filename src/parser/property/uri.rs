use crate::parser::{Error, InnerError};
use nom::branch::alt;
use nom::bytes::streaming::{tag, take_while, take_while1, take_while_m_n};
use nom::character::streaming::char;
use nom::character::{is_alphabetic, is_digit};
use nom::combinator::{map_res, opt, recognize, verify};
use nom::error::ParseError;
use nom::multi::{fold_many0, fold_many1, many0, many1, separated_list0};
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
    pub authority: Option<Authority>,
    pub path: Vec<u8>,
    pub query: Option<&'a [u8]>,
    pub fragment: Option<&'a [u8]>,
}

pub fn param_value_uri(input: &[u8]) -> IResult<&[u8], Uri, Error> {
    let (input, (scheme, _, (authority, path), query, fragment)) = tuple((
        scheme,
        char(':'),
        alt((
            tuple((tag("//"), authority, opt(path_absolute_empty))).map(|(_, a, b)| (Some(a), b)),
            path_absolute.map(|p| (None, Some(p))),
            path_rootless.map(|p| (None, Some(p))),
        )),
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
    .map(|(user_info, host, port)| Authority {
        user_info,
        host,
        port,
    })
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

    if prefix_parts.len() > 7 {
        return Err(nom::Err::Error(Error::new(input, InnerError::InvalidIpv6)));
    }

    let (input, found_collapse) = opt(tag("::"))(input)?;
    let fill_zeroes = found_collapse.is_some();

    let (input, suffix_parts) = separated_list0(char(':'), h_16)(input)?;

    if suffix_parts.len() > 8 {
        return Err(nom::Err::Error(Error::new(input, InnerError::InvalidIpv6)));
    }

    let (input, ipv4_post) = opt(tuple((char(':'), ip_v4_addr)))(input)?;

    let mut content = [0u8; 16];

    let provided_len =
        prefix_parts.len() * 2 + suffix_parts.len() * 2 + if ipv4_post.is_some() { 4 } else { 0 };

    if provided_len > 16 {
        return Err(nom::Err::Error(Error::new(input, InnerError::InvalidIpv6)));
    } else if provided_len < 16 && !fill_zeroes {
        return Err(nom::Err::Error(Error::new(input, InnerError::InvalidIpv6)));
    }

    let mut i = 0;
    for [a, b] in prefix_parts {
        content[i] = a;
        content[i + 1] = b;
        i += 2;
    }

    if fill_zeroes {
        let zeroes = 16 - provided_len;
        i += zeroes;
    }

    for [a, b] in suffix_parts {
        content[i] = a;
        content[i + 1] = b;
        i += 2;
    }

    if let Some((_, ipv4)) = ipv4_post {
        content[12] = ipv4[0];
        content[13] = ipv4[1];
        content[14] = ipv4[2];
        content[15] = ipv4[3];
    }

    Ok((input, Ipv6Addr::from(content)))
}

fn h_16(input: &[u8]) -> IResult<&[u8], [u8; 2], Error> {
    take_while_m_n(1, 4, is_hex_digit)
        .map(|c: &[u8]| {
            let mut src = c.to_vec();
            while src.len() < 4 {
                src.insert(0, b'0');
            }
            let mut dst = [0, 0];
            hex::decode_to_slice(src, &mut dst).unwrap();
            dst
        })
        .parse(input)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::check_rem;

    #[test]
    fn ftp() {
        let (input, uri) = param_value_uri(b"ftp://ftp.is.co.za/rfc/rfc1808.txt`").unwrap();
        check_rem(input, 1);
        assert_eq!(uri.scheme, b"ftp");
        assert_eq!(
            uri.authority.unwrap().host,
            Host::RegName(b"ftp.is.co.za".to_vec())
        );
        assert_eq!(uri.path, b"/rfc/rfc1808.txt");
    }

    #[test]
    fn http() {
        let (input, uri) = param_value_uri(b"http://www.ietf.org/rfc/rfc2396.txt`").unwrap();
        check_rem(input, 1);
        assert_eq!(uri.scheme, b"http");
        assert_eq!(
            uri.authority.unwrap().host,
            Host::RegName(b"www.ietf.org".to_vec())
        );
        assert_eq!(uri.path, b"/rfc/rfc2396.txt");
    }

    #[test]
    fn ip_v6() {
        let (input, ipv6) = ip_v6_addr(b"2001:db8::7`").unwrap();
        check_rem(input, 1);
        assert_eq!(ipv6, Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 7));
    }

    #[test]
    fn ldap() {
        let (input, uri) = param_value_uri(b"ldap://[2001:db8::7]/c=GB?objectClass?one`").unwrap();
        check_rem(input, 1);
        assert_eq!(uri.scheme, b"ldap");
        assert_eq!(
            uri.authority.unwrap().host,
            Host::IpAddr(IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 7)))
        );
        assert_eq!(uri.path, b"/c=GB");
        assert_eq!(uri.query.unwrap(), b"objectClass?one");
    }

    #[test]
    fn mailto() {
        let (input, uri) = param_value_uri(b"mailto:John.Doe@example.com`").unwrap();
        check_rem(input, 1);
        assert_eq!(uri.scheme, b"mailto");
        assert_eq!(uri.path, b"John.Doe@example.com".to_vec());
    }

    #[test]
    fn news() {
        let (input, uri) = param_value_uri(b"news:comp.infosystems.www.servers.unix`").unwrap();
        check_rem(input, 1);
        assert_eq!(uri.scheme, b"news");
        assert_eq!(uri.path, b"comp.infosystems.www.servers.unix".to_vec());
    }

    #[test]
    fn tel() {
        let (input, uri) = param_value_uri(b"tel:+1-816-555-1212`").unwrap();
        check_rem(input, 1);
        assert_eq!(uri.scheme, b"tel");
        assert_eq!(uri.path, b"+1-816-555-1212".to_vec());
    }

    #[test]
    fn telnet() {
        let (input, uri) = param_value_uri(b"telnet://192.0.2.16:80/`").unwrap();
        check_rem(input, 1);
        assert_eq!(uri.scheme, b"telnet");
        let authority = uri.authority.unwrap();
        assert_eq!(
            authority.host,
            Host::IpAddr(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 16)))
        );
        assert_eq!(authority.port.unwrap(), 80);
    }

    #[test]
    fn urn() {
        let (input, uri) =
            param_value_uri(b"urn:oasis:names:specification:docbook:dtd:xml:4.1.2`").unwrap();
        check_rem(input, 1);
        assert_eq!(uri.scheme, b"urn");
        assert_eq!(
            uri.path,
            b"oasis:names:specification:docbook:dtd:xml:4.1.2".to_vec()
        );
    }
}
