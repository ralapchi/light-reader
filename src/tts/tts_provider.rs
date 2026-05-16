use crate::tts::config::TtsConfig;
use crate::tts::types::{TtsProviderKind, TtsRequest, TtsResponse, TtsVoice};

#[derive(Debug)]
#[allow(dead_code)]
pub enum TtsError {
    HttpError(String),
    AuthError(String),
    RateLimited { retry_after: Option<u64> },
    InvalidConfig(String),
    AudioDecodeError(String),
    TextTooLong { max_chars: usize },
    Unknown(String),
}

impl std::fmt::Display for TtsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TtsError::HttpError(msg) => write!(f, "网络请求失败: {}", msg),
            TtsError::AuthError(_) => write!(f, "鉴权失败，请检查 API Key 是否正确"),
            TtsError::RateLimited { .. } => write!(f, "请求过于频繁，请稍后重试"),
            TtsError::InvalidConfig(msg) => write!(f, "配置无效: {}", msg),
            TtsError::AudioDecodeError(msg) => write!(f, "音频解码失败: {}", msg),
            TtsError::TextTooLong { max_chars } => {
                write!(f, "文本过长，最多 {} 字", max_chars)
            }
            TtsError::Unknown(msg) => write!(f, "未知错误: {}", msg),
        }
    }
}

impl TtsError {
    #[allow(dead_code)]
    /// Whether this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(self, TtsError::HttpError(_) | TtsError::RateLimited { .. })
    }

    /// Convert to an error code string (safe for logging, no secrets exposed)
    pub fn error_code(&self) -> &str {
        match self {
            TtsError::HttpError(_) => "TTS_NETWORK_ERROR",
            TtsError::AuthError(_) => "TTS_AUTH_ERROR",
            TtsError::RateLimited { .. } => "TTS_RATE_LIMITED",
            TtsError::InvalidConfig(_) => "TTS_INVALID_CONFIG",
            TtsError::AudioDecodeError(_) => "TTS_AUDIO_DECODE_ERROR",
            TtsError::TextTooLong { .. } => "TTS_TEXT_TOO_LONG",
            TtsError::Unknown(_) => "TTS_UNKNOWN_ERROR",
        }
    }
}

#[allow(dead_code)]
pub trait TtsProvider: Send + Sync {
    fn kind(&self) -> TtsProviderKind;
    fn validate_config(&self, config: &TtsConfig) -> Result<(), Vec<String>>;
    fn synthesize(&self, request: &TtsRequest, config: &TtsConfig)
    -> Result<TtsResponse, TtsError>;
    fn test_connection(&self, config: &TtsConfig) -> Result<(), TtsError>;
    fn list_voices(&self, config: &TtsConfig) -> Result<Vec<TtsVoice>, TtsError>;
    fn max_text_length(&self) -> usize;
}
