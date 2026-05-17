use crate::domain::app_error::AppResult;
use crate::domain::reader_settings::ReaderSettings;
use crate::tts::config::TtsConfig;

/// Service trait for settings and TTS configuration persistence.
pub trait SettingsService {
    /// Load reader settings from disk.
    fn load_settings(&self) -> ReaderSettings;

    /// Save reader settings to disk.
    fn save_settings(&self, settings: &ReaderSettings) -> AppResult<()>;

    /// Load TTS config from disk.
    fn load_tts_config(&self) -> TtsConfig;

    /// Save TTS config to disk.
    fn save_tts_config(&self, config: &TtsConfig) -> AppResult<()>;
}
