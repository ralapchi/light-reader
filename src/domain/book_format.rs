use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BookFormat {
    Epub,
    Txt,
}

impl fmt::Display for BookFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BookFormat::Epub => write!(f, "Epub"),
            BookFormat::Txt => write!(f, "Txt"),
        }
    }
}

impl FromStr for BookFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Epub" => Ok(BookFormat::Epub),
            "Txt" => Ok(BookFormat::Txt),
            other => Err(format!("Unknown BookFormat: {}", other)),
        }
    }
}
