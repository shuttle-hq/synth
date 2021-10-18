use std::fmt::{Debug, Display};

use crate::value::{IntoToken, Special, Token};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Error {
    Type { expected: String, got: String },
    Deserialize { msg: String },
    Serialize { msg: String },
    Custom { msg: String },
}

impl Error {
    pub fn r#type<T1: ToString, T2: Debug>(expected: T1, got: T2) -> Self {
        Self::Type {
            expected: expected.to_string(),
            got: format!("{:?}", got),
        }
    }

    pub fn custom<T: Display>(msg: T) -> Self {
        Self::Custom {
            msg: msg.to_string(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Serialize {
            msg: msg.to_string(),
        }
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Deserialize {
            msg: msg.to_string(),
        }
    }
}

impl IntoToken for Error {
    fn into_token(self) -> Token {
        Token::Special(Special::Error(self))
    }
}
