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
    config.provider.as_str().to_string()
}

pub struct TtsSynthesisService {
    providers: Vec<Box<dyn TtsProvider>>,
}

impl TtsSynthesisService {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
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
    /// Checks cache first; only calls the provider API on cache miss.
    pub fn synthesize_blocking(
        request: &TtsRequest,
        config: &TtsConfig,
        voice_id: &str,
        cache: &TtsCache,
    ) -> Result<TtsResponse, TtsError> {
        let provider_label = provider_cache_label(config);
        let path = cache.segment_path(
            &provider_label,
            &request.book_id,
            request.chapter_index,
            request.segment_index,
            voice_id,
            "pcm16",
        );

        // Check cache first — avoids redundant API calls during prefetch
        if cache.exists(&path) {
            match cache.read(&path) {
                Ok(audio_bytes) => {
                    log::info!(
                        "TTS 缓存命中: book={} ch={} seg={}",
                        &request.book_id,
                        request.chapter_index,
                        request.segment_index
                    );
                    return Ok(TtsResponse {
                        audio_bytes,
                        media_type: "audio/pcm16".to_string(),
                        duration_ms: None,
                    });
                }
                Err(e) => {
                    log::warn!("TTS 缓存读取失败，重新合成: {}", e);
                }
            }
        }

        // Cache miss — synthesize via provider
        let provider = create_provider_from_config(config);
        let resp = provider.synthesize(request, config)?;

        // Write to cache (best-effort)
        if let Err(e) = cache.write(&path, &resp.audio_bytes) {
            log::warn!("TTS 缓存写入失败: {}", e);
        }
        cache.prune_if_over_limit();
        Ok(resp)
    }
}
