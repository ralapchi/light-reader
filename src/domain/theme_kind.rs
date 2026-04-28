use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ThemeKind {
    Light,
    Dark,
    Sepia,
    Paper,
    Custom,
}
