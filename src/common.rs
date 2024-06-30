#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum CalendarUserType {
    #[default]
    Individual,
    Group,
    Resource,
    Room,
    Unknown,
    XName(String),
    IanaToken(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum Encoding {
    #[default]
    EightBit,
    Base64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FreeBusyTimeType {
    Free,
    Busy,
    BusyUnavailable,
    BusyTentative,
    XName(String),
    IanaToken(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageTag {
    pub language: String,
    pub ext_lang: Option<String>,
    pub script: Option<String>,
    pub region: Option<String>,
    pub variants: Vec<String>,
    pub extensions: Vec<String>,
    pub private_use: Option<String>,
}

impl Default for LanguageTag {
    fn default() -> Self {
        Self {
            language: String::new(),
            ext_lang: None,
            script: None,
            region: None,
            variants: Vec::with_capacity(0),
            extensions: Vec::with_capacity(0),
            private_use: None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Range {
    ThisAndFuture,
}