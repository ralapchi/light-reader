use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BookFormat {
    Epub,
    Txt,
}

impl BookFormat {
    /// 根据文件路径扩展名推断格式（大小写不敏感）
    pub fn from_path(path: &str) -> Option<Self> {
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase())?;
        match ext.as_str() {
            "epub" => Some(Self::Epub),
            "txt" => Some(Self::Txt),
            _ => None,
        }
    }
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
