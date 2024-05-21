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

#[derive(Debug, Eq, PartialEq)]
pub struct CalendarScaleProperty<'a> {
    pub other_params: Vec<Param<'a>>,
    pub value: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct MethodProperty<'a> {
    pub other_params: Vec<Param<'a>>,
    pub value: &'a [u8],
}

#[derive(Debug, Eq, PartialEq)]
pub struct XProperty<'a> {
    pub name: &'a [u8],
    pub params: Vec<Param<'a>>,
    pub value: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct IanaProperty<'a> {
    pub name: &'a [u8],
    pub params: Vec<Param<'a>>,
    pub value: Vec<u8>,
}
