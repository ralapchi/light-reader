use crate::services::settings_service::SettingsService;
use crate::services::settings_service_impl::SettingsServiceImpl;

use super::super::dto::*;
use super::dto_convert::{dto_to_tts_config, tts_config_to_dto};

#[tauri::command]
pub fn settings_load() -> Result<serde_json::Value, String> {
    let svc = SettingsServiceImpl::new();
    let settings = svc.load_settings();
    serde_json::to_value(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn settings_save(settings: serde_json::Value) -> Result<(), String> {
    let parsed: crate::domain::reader_settings::ReaderSettings =
        serde_json::from_value(settings).map_err(|e| e.to_string())?;
    let svc = SettingsServiceImpl::new();
    svc.save_settings(&parsed).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn tts_config_load() -> Result<TtsConfigDto, String> {
    let svc = SettingsServiceImpl::new();
    let config = svc.load_tts_config();
    Ok(tts_config_to_dto(&config))
}

#[tauri::command]
pub fn tts_config_save(config: TtsConfigDto) -> Result<(), String> {
    let existing = SettingsServiceImpl::new().load_tts_config();
    let full = dto_to_tts_config(&config, existing.api_key);
    let svc = SettingsServiceImpl::new();
    svc.save_tts_config(&full).map_err(|e| e.to_string())
}
