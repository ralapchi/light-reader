use serde::{Deserialize, Serialize};

use crate::tts::types::TtsProviderKind;

/// BYOK TTS configuration.
///
/// Mirrors the OpenAI Python client: api_key + base_url + model
/// are the only parameters needed to connect.
///
/// TRANSITIONAL: api_key is stored as plaintext in settings.json.
/// Migrate to OS keychain in a future iteration.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TtsConfig {
    pub enabled: bool,
    pub provider: TtsProviderKind,
    /// API key for the TTS service (MIMO_API_KEY for Xiaomi).
    pub api_key: Option<String>,
    /// Base URL of the TTS API (e.g. https://api.xiaomimimo.com/v1).
    pub base_url: Option<String>,
    /// Model name (e.g. "mimo-v2-tts").
    pub model: Option<String>,
    /// Voice / speaker identifier (e.g. "default_en").
    pub voice_id: Option<String>,
}

impl std::fmt::Debug for TtsConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TtsConfig")
            .field("enabled", &self.enabled)
            .field("provider", &self.provider)
            .field("api_key", &self.api_key.as_ref().map(|_| "***masked***"))
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .field("voice_id", &self.voice_id)
            .finish()
    }
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: TtsProviderKind::Xiaomi,
            api_key: None,
            base_url: None,
            model: None,
            voice_id: None,
        }
    }
}

impl TtsConfig {
    #[allow(dead_code)]
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if !self.enabled {
            return Ok(());
        }
        if self.api_key.is_none() || self.api_key.as_ref().map_or(true, |s| s.is_empty()) {
            errors.push("API Key 不能为空".to_string());
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
