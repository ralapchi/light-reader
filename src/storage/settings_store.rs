use log::warn;
use serde::{Deserialize, Serialize};

use crate::domain::reader_settings::ReaderSettings;
use crate::storage::paths;
use crate::tts::config::TtsConfig;
use crate::tts::secure_store;

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
        tts_config: Option<TtsConfig>,
    ) -> Self {
        Self {
            version: SETTINGS_VERSION,
            reader_settings: reader_settings.clone(),
            window_size: None,
            window_pos: None,
            last_opened_book_id,
            tts_config,
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
            Ok(mut file) => {
                // T12: 版本检查
                if file.version != SETTINGS_VERSION {
                    warn!(
                        "设置文件版本不匹配: 期望 {}，实际 {}，尝试兼容读取",
                        SETTINGS_VERSION, file.version
                    );
                }
                // Restore API key from keyring if marker is present
                if let Some(ref mut config) = file.tts_config {
                    if config.api_key.as_deref() == Some(secure_store::KEYRING_MARKER) {
                        let provider_label = format!("{:?}", config.provider).to_lowercase();
                        match secure_store::load_api_key(&provider_label) {
                            Ok(key) => config.api_key = Some(key),
                            Err(e) => {
                                warn!("从 keyring 读取 API Key 失败: {}", e);
                                config.api_key = None;
                            }
                        }
                    }
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

    // Clone settings so we can replace api_key with keyring marker
    let mut to_save = SettingsFile {
        version: settings.version,
        reader_settings: settings.reader_settings.clone(),
        window_size: settings.window_size,
        window_pos: settings.window_pos,
        last_opened_book_id: settings.last_opened_book_id.clone(),
        tts_config: settings.tts_config.clone(),
    };

    if let Some(ref mut config) = to_save.tts_config {
        if let Some(ref key) = config.api_key {
            if key != secure_store::KEYRING_MARKER {
                let provider_label = format!("{:?}", config.provider).to_lowercase();
                match secure_store::save_api_key(&provider_label, key) {
                    Ok(()) => config.api_key = Some(secure_store::KEYRING_MARKER.to_string()),
                    Err(e) => {
                        warn!("keyring 保存 API Key 失败，回退到明文存储: {}", e);
                        // Keep plaintext as fallback
                    }
                }
            }
        }
    }

    let data = serde_json::to_string_pretty(&to_save).map_err(|e| e.to_string())?;
    std::fs::write(&path, data).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::reader_settings::ReaderSettings;
    use crate::tts::config::TtsConfig;
    use crate::tts::types::TtsProviderKind;

    #[test]
    fn from_reader_settings_preserves_tts_config() {
        let tts = TtsConfig {
            enabled: true,
            provider: TtsProviderKind::Xiaomi,
            api_key: Some("test-key".to_string()),
            base_url: Some("https://api.example.com".to_string()),
            model: Some("mimo-v2".to_string()),
            voice_id: Some("female".to_string()),
        };
        let file = SettingsFile::from_reader_settings(
            &ReaderSettings::default(),
            Some("book-1".to_string()),
            Some(tts.clone()),
        );
        assert_eq!(file.tts_config, Some(tts));
        assert_eq!(file.last_opened_book_id, Some("book-1".to_string()));
    }

    #[test]
    fn settings_file_json_roundtrip() {
        let tts = TtsConfig {
            enabled: true,
            provider: TtsProviderKind::Xiaomi,
            api_key: Some("key-123".to_string()),
            base_url: None,
            model: None,
            voice_id: None,
        };
        let original = SettingsFile::from_reader_settings(
            &ReaderSettings::default(),
            None,
            Some(tts),
        );
        let json = serde_json::to_string(&original).unwrap();
        let restored: SettingsFile = serde_json::from_str(&json).unwrap();
        assert_eq!(original.tts_config, restored.tts_config);
        assert_eq!(original.version, restored.version);
    }
}
