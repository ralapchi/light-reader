use log::warn;
use serde::{Deserialize, Serialize};

use crate::domain::reader_settings::ReaderSettings;
use crate::storage::paths;
use crate::tts::config::TtsConfig;

const SETTINGS_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
pub struct SettingsFile {
    pub version: u32,
    pub reader_settings: ReaderSettings,
    pub window_size: Option<(f32, f32)>,
    pub window_pos: Option<(f32, f32)>,
    pub last_opened_book_id: Option<String>,
    /// TTS configuration (BYOK). TRANSITIONAL: API key stored as plaintext.
    /// Migrate to OS keychain in a future iteration.
    pub tts_config: Option<TtsConfig>,
}

impl Default for SettingsFile {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            reader_settings: ReaderSettings::default(),
            window_size: None,
            window_pos: None,
            last_opened_book_id: None,
            tts_config: None,
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
            tts_config: None,
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
            Ok(file) => {
                // T12: 版本检查
                if file.version != SETTINGS_VERSION {
                    warn!(
                        "设置文件版本不匹配: 期望 {}，实际 {}，尝试兼容读取",
                        SETTINGS_VERSION, file.version
                    );
                }
                file
            }
            Err(e) => {
                warn!("设置文件解析失败: {}，回退到默认设置", e);
                SettingsFile::default()
            }
        },
        Err(e) => {
            warn!("设置文件读取失败: {}，回退到默认设置", e);
            SettingsFile::default()
        }
    }
}

pub fn save(settings: &SettingsFile) -> Result<(), String> {
    let path = paths::settings_path();
    let data = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, data).map_err(|e| e.to_string())
}
