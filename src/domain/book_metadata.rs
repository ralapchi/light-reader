use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BookMetadata {
    pub title: String,
    pub author: Option<String>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub description: Option<String>,
    pub identifier: Option<String>,
    pub series: Option<String>,
    pub cover_title: Option<String>,
    pub created_at: Option<String>,
    pub modified_at: Option<String>,
}
