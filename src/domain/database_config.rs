use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseBackendType {
    Sqlite,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub backend: DatabaseBackendType,
    pub path: Option<String>,
    pub connection_string: Option<String>,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

fn default_max_connections() -> u32 {
    5
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            backend: DatabaseBackendType::Sqlite,
            path: None,
            connection_string: None,
            max_connections: 5,
        }
    }
}
