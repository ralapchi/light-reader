use std::path::PathBuf;
use std::sync::Arc;

use crate::tts::cache::TtsCache;
use crate::tts::config::TtsConfig;
use crate::tts::tts_provider::{TtsError, TtsProvider};
use crate::tts::types::{TtsProviderKind, TtsRequest, TtsResponse};

fn create_provider_from_config(config: &TtsConfig) -> Box<dyn TtsProvider> {
    match config.provider {
        TtsProviderKind::Xiaomi => Box::new(crate::tts::xiaomi_provider::XiaomiTtsProvider::new()),
    }
}

fn provider_cache_label(config: &TtsConfig) -> String {
    format!("{:?}", config.provider).to_lowercase()
}

pub struct TtsSynthesisService {
    providers: Vec<Box<dyn TtsProvider>>,
    #[allow(dead_code)] // instance-based synthesize() not yet wired up; currently using static synthesize_blocking()
    cache: Arc<TtsCache>,
}

impl TtsSynthesisService {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            providers: Vec::new(),
            cache: Arc::new(TtsCache::new(cache_dir)),
        }
    }

    // ── Provider management ──────────────────────────────────

    pub fn register_provider(&mut self, provider: Box<dyn TtsProvider>) {
        self.providers.push(provider);
    }

    pub fn get_provider(&self, kind: TtsProviderKind) -> Option<&dyn TtsProvider> {
        self.providers
            .iter()
            .find(|p| p.kind() == kind)
            .map(|p| p.as_ref())
    }

    // ── Config validation ────────────────────────────────────

    #[allow(dead_code)]
    pub fn validate_config(&self, config: &TtsConfig) -> Result<(), Vec<String>> {
        let provider = self
            .get_provider(config.provider.clone())
            .ok_or_else(|| vec!["不支持的 TTS Provider".to_string()])?;
        provider.validate_config(config)
    }

    // ── Connection testing ───────────────────────────────────

    pub fn test_connection(&self, config: &TtsConfig) -> Result<(), TtsError> {
        let provider = self
            .get_provider(config.provider.clone())
            .ok_or_else(|| TtsError::InvalidConfig("不支持的 TTS Provider".to_string()))?;
        provider.test_connection(config)
    }

    // ── Synthesis with cache ─────────────────────────────────

    /// Synthesize audio via the configured provider, write result to cache.
    /// Designed to be called from background threads — takes no `&self`,
    /// only explicit params so callers don't need a service handle.
    pub fn synthesize_blocking(
        request: &TtsRequest,
        config: &TtsConfig,
        voice_id: &str,
        cache: &TtsCache,
    ) -> Result<TtsResponse, TtsError> {
        let provider = create_provider_from_config(config);
        let resp = provider.synthesize(request, config)?;
        let path = cache.segment_path(
            &provider_cache_label(config),
            &request.book_id,
            request.chapter_index,
            request.segment_index,
            voice_id,
            "pcm16",
        );
        let _ = cache.write(&path, &resp.audio_bytes);
        Ok(resp)
    }

    #[allow(dead_code)] // not yet wired up; currently using static synthesize_blocking()
    pub fn synthesize(
        &self,
        request: &TtsRequest,
        config: &TtsConfig,
    ) -> Result<TtsResponse, TtsError> {
        let provider = self
            .get_provider(config.provider.clone())
            .ok_or_else(|| TtsError::InvalidConfig("不支持的 TTS Provider".to_string()))?;

        let voice_id = config.voice_id.as_deref().unwrap_or("default");
        let ext = "pcm16";

        let cache_path = self.cache.segment_path(
            &provider_cache_label(config),
            &request.book_id,
            request.chapter_index,
            request.segment_index,
            voice_id,
            ext,
        );

        // Check cache first
        if self.cache.exists(&cache_path) {
            match self.cache.read(&cache_path) {
                Ok(audio_bytes) => {
                    log::info!(
                        "TTS 缓存命中: book={} ch={} seg={}",
                        &request.book_id,
                        request.chapter_index,
                        request.segment_index
                    );
                    return Ok(TtsResponse {
                        audio_bytes,
                        media_type: format!("audio/{}", ext),
                        duration_ms: None,
                    });
                }
                Err(e) => {
                    log::warn!("TTS 缓存读取失败，重新合成: {}", e);
                }
            }
        }

        // Synthesize
        let response = provider.synthesize(request, config)?;

        // Write to cache (best-effort)
        if let Err(e) = self.cache.write(&cache_path, &response.audio_bytes) {
            log::warn!("TTS 缓存写入失败: {}", e);
        }

        Ok(response)
    }
}
