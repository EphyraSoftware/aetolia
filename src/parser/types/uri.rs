use std::net::{Ipv4Addr, Ipv6Addr};

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
