use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TagGroup {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub sort_order: i32,
}
