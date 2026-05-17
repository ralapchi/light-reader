use std::sync::OnceLock;

use base64::Engine as _;

use crate::tts::config::TtsConfig;
use crate::tts::tts_provider::{TtsError, TtsProvider};
use crate::tts::types::{TtsProviderKind, TtsRequest, TtsResponse};

const DEFAULT_BASE_URL: &str = "https://api.xiaomimimo.com/v1";
const DEFAULT_MODEL: &str = "mimo-v2-tts";
const DEFAULT_VOICE: &str = "default_en";

fn http_client() -> &'static reqwest::blocking::Client {
    static CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client")
    })
}

pub struct XiaomiTtsProvider;

impl XiaomiTtsProvider {
    pub fn new() -> Self {
        Self
    }

    fn chat_url(config: &TtsConfig) -> String {
        let base = config
            .base_url
            .as_deref()
            .unwrap_or(DEFAULT_BASE_URL)
            .trim_end_matches('/');
        format!("{}/chat/completions", base)
    }
}

impl TtsProvider for XiaomiTtsProvider {
    fn kind(&self) -> TtsProviderKind {
        TtsProviderKind::Xiaomi
    }

    fn validate_config(&self, config: &TtsConfig) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if config.api_key.is_none() || config.api_key.as_ref().map_or(true, |s| s.is_empty()) {
            errors.push("Xiaomi TTS: API Key 不能为空".to_string());
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn synthesize(
        &self,
        request: &TtsRequest,
        config: &TtsConfig,
    ) -> Result<TtsResponse, TtsError> {
        let url = Self::chat_url(config);
        let api_key = config.api_key.as_deref().unwrap_or("");
        let model = config.model.as_deref().unwrap_or(DEFAULT_MODEL);
        let voice = request
            .voice_id
            .as_deref()
            .or(config.voice_id.as_deref())
            .unwrap_or(DEFAULT_VOICE);

        let body = serde_json::json!({
            "model": model,
            "messages": [
                {"role": "assistant", "content": request.text}
            ],
            "audio": {
                "format": "pcm16",
                "voice": voice
            },
            "stream": true
        });

        let resp = http_client()
            .post(&url)
            .header("api-key", api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| {
                log::warn!(
                    "Xiaomi TTS request failed: {}",
                    TtsError::error_code(&TtsError::HttpError(e.to_string()))
                );
                TtsError::HttpError(e.to_string())
            })?;

        let status = resp.status();
        if status.is_success() {
            let text = resp
                .text()
                .map_err(|e| TtsError::HttpError(format!("读取响应失败: {}", e)))?;

            let audio_bytes = parse_sse_response(&text)?;

            Ok(TtsResponse {
                audio_bytes,
                media_type: "audio/pcm16".to_string(),
                duration_ms: None,
            })
        } else {
            let status_code = status.as_u16();
            let body_text = resp.text().unwrap_or_default();
            log::warn!(
                "Xiaomi TTS API error: status={} body={}",
                status_code,
                &body_text.chars().take(300).collect::<String>()
            );

            let err_detail = serde_json::from_str::<serde_json::Value>(&body_text)
                .ok()
                .and_then(|v| {
                    v.get("error")
                        .and_then(|e| e.get("message"))
                        .and_then(|m| m.as_str())
                        .map(|s| s.to_string())
                });

            match status_code {
                401 | 403 => Err(TtsError::AuthError(
                    err_detail.unwrap_or_else(|| "鉴权失败，请检查 API Key".to_string()),
                )),
                429 => Err(TtsError::RateLimited { retry_after: None }),
                code if code >= 500 => Err(TtsError::HttpError(format!(
                    "服务端错误({}): {}",
                    code,
                    err_detail.unwrap_or_else(|| "请稍后重试".to_string())
                ))),
                _ => {
                    let detail = err_detail.unwrap_or_else(|| format!("HTTP {}", status_code));
                    Err(TtsError::Unknown(format!(
                        "{} (voice={}, model={})",
                        detail, voice, model
                    )))
                }
            }
        }
    }

    fn test_connection(&self, config: &TtsConfig) -> Result<(), TtsError> {
        let request = TtsRequest {
            book_id: String::new(),
            chapter_index: 0,
            segment_index: 0,
            paragraph_indices: vec![0],
            text: "你好，这是一条测试语音。".to_string(),
            voice_id: config.voice_id.clone(),
        };
        self.synthesize(&request, config).map(|_| ())
    }

    fn max_text_length(&self) -> usize {
        500
    }
}

/// Parse an SSE (Server-Sent Events) response to extract PCM16 audio chunks.
///
/// Expected format:
///   data: {"choices":[{"index":0,"delta":{"audio":{"data":"<base64>"}}}]}
///   data: [DONE]
///
/// Each SSE `data:` line is parsed as JSON. Audio data from `choices[0].delta.audio.data`
/// is base64-decoded and accumulated. Parsing stops at `[DONE]` or end of stream.
fn parse_sse_response(text: &str) -> Result<Vec<u8>, TtsError> {
    let engine = base64::engine::general_purpose::STANDARD;
    let mut all_audio = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("data: ") {
            continue;
        }
        let data = &trimmed[6..];
        if data == "[DONE]" {
            break;
        }

        let Ok(json) = serde_json::from_str::<serde_json::Value>(data) else {
            continue;
        };

        // Extract audio.data base64 field from choices[0].delta.audio.data
        let audio_chunk = json
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|ch| ch.get("delta"))
            .and_then(|d| d.get("audio"))
            .and_then(|a| a.get("data"))
            .and_then(|d| d.as_str());

        if let Some(b64) = audio_chunk {
            match engine.decode(b64) {
                Ok(decoded) => all_audio.extend_from_slice(&decoded),
                Err(e) => {
                    log::warn!("Xiaomi TTS: base64 解码片段失败: {}", e);
                }
            }
        }
    }

    if all_audio.is_empty() {
        Err(TtsError::Unknown(
            "TTS 响应中未找到音频数据，请检查 API Key 或 Voice 是否正确".to_string(),
        ))
    } else {
        Ok(all_audio)
    }
}
