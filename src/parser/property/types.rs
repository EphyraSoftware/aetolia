use crate::parser::param::Param;

#[derive(Debug, Eq, PartialEq)]
pub struct ProductId<'a> {
    pub other_params: Vec<Param<'a>>,
    pub value: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VersionProperty<'a> {
    pub other_params: Vec<Param<'a>>,
    pub min_version: Option<&'a [u8]>,
    pub max_version: &'a [u8],
}
