use std::fmt::{Display, Formatter};
use std::io::Error;

#[derive(Debug)]
pub enum AetoliaError {
    IO(Error),

    Other(String),
}

impl From<Error> for AetoliaError {
    fn from(value: Error) -> Self {
        Self::IO(value)
    }
}

impl AetoliaError {
    pub fn time(e: impl Into<time::error::Error>) -> Self {
        AetoliaError::Other(e.into().to_string())
    }

    pub fn other(msg: impl ToString) -> AetoliaError {
        AetoliaError::Other(msg.to_string())
    }
}

impl Display for AetoliaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AetoliaError::IO(e) => e.fmt(f),
            AetoliaError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for AetoliaError {}

pub type AetoliaResult<T> = Result<T, AetoliaError>;
