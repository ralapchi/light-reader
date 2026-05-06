use log::warn;
use serde::{Deserialize, Serialize};

use crate::domain::reader_settings::ReaderSettings;
use crate::storage::paths;

const SETTINGS_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
pub struct SettingsFile {
    pub version: u32,
    pub reader_settings: ReaderSettings,
    pub window_size: Option<(f32, f32)>,
    pub window_pos: Option<(f32, f32)>,
    pub last_opened_book_id: Option<String>,
}

impl Default for SettingsFile {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            reader_settings: ReaderSettings::default(),
            window_size: None,
            window_pos: None,
            last_opened_book_id: None,
        }
    }
}

impl SettingsFile {
    pub fn from_reader_settings(
        reader_settings: &ReaderSettings,
        last_opened_book_id: Option<String>,
    ) -> Self {
        Self {
            version: SETTINGS_VERSION,
            reader_settings: reader_settings.clone(),
            window_size: None,
            window_pos: None,
            last_opened_book_id,
        }
    }
}

pub fn load() -> SettingsFile {
    let path = paths::settings_path();
    if !path.exists() {
        return SettingsFile::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(data) => match serde_json::from_str::<SettingsFile>(&data) {
            Ok(file) => file,
            Err(e) => {
                warn!("设置文件解析失败，使用默认设置: {}", e);
                SettingsFile::default()
            }
        },
        Err(e) => {
            warn!("设置文件读取失败，使用默认设置: {}", e);
            SettingsFile::default()
        }
    }
}

pub fn save(settings: &SettingsFile) -> Result<(), String> {
    let path = paths::settings_path();
    let data = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, data).map_err(|e| e.to_string())
}
