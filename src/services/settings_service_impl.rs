use crate::domain::app_error::{AppError, AppResult};
use crate::domain::reader_settings::ReaderSettings;
use crate::services::settings_service::SettingsService;
use crate::storage::settings_store;
use crate::tts::config::TtsConfig;

/// Concrete implementation of SettingsService.
///
/// Wraps `storage::settings_store` load/save with version-check logic intact.
pub struct SettingsServiceImpl;

impl SettingsServiceImpl {
    pub fn new() -> Self {
        Self
    }
}

impl SettingsService for SettingsServiceImpl {
    fn load_settings(&self) -> ReaderSettings {
        settings_store::load().reader_settings
    }

    fn save_settings(&self, settings: &ReaderSettings) -> AppResult<()> {
        let settings = settings.clone();
        settings_store::update(|file| file.reader_settings = settings)
            .map_err(|e| AppError::new("SETTINGS_SAVE_FAILED", &e))
    }

    fn load_tts_config(&self) -> TtsConfig {
        settings_store::load().tts_config.unwrap_or_default()
    }

    fn save_tts_config(&self, config: &TtsConfig) -> AppResult<()> {
        let config = config.clone();
        settings_store::update(|file| file.tts_config = Some(config))
            .map_err(|e| AppError::new("TTS_CONFIG_SAVE_FAILED", &e))
    }
}
