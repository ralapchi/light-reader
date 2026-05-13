use std::path::PathBuf;
use std::sync::Arc;

use crate::domain::paragraph::Paragraph;
use crate::tts::cache::TtsCache;
use crate::tts::config::TtsConfig;
use crate::tts::player::AudioPlayer;
use crate::tts::segmenter::{segment_chapter, Segment};
use crate::tts::tts_provider::{TtsError, TtsProvider};
use crate::tts::types::{TtsProviderKind, TtsRequest, TtsResponse};

pub struct TtsService {
    providers: Vec<Box<dyn TtsProvider>>,
    cache: Arc<TtsCache>,
    player: Option<AudioPlayer>,
}

fn create_provider_from_config(config: &TtsConfig) -> Box<dyn TtsProvider> {
    match config.provider {
        TtsProviderKind::Xiaomi => Box::new(crate::tts::xiaomi_provider::XiaomiTtsProvider::new()),
    }
}

fn provider_cache_label(config: &TtsConfig) -> String {
    format!("{:?}", config.provider).to_lowercase()
}

#[allow(dead_code)]
impl TtsService {
    pub fn new(cache_dir: PathBuf) -> Self {
        let player = AudioPlayer::new().ok();
        if player.is_none() {
            log::warn!("TTS: 音频播放器初始化失败（无声模式）");
        }
        Self {
            providers: Vec::new(),
            cache: Arc::new(TtsCache::new(cache_dir)),
            player,
        }
    }

    /// Cloneable handle to the cache for background threads.
    pub fn cache_arc(&self) -> Arc<TtsCache> {
        Arc::clone(&self.cache)
    }

    // ── Provider management ──────────────────────────────────

    pub fn register_provider(&mut self, provider: Box<dyn TtsProvider>) {
        self.providers.push(provider);
    }

    pub fn get_provider(&self, kind: TtsProviderKind) -> Option<&dyn TtsProvider> {
        self.providers.iter().find(|p| p.kind() == kind).map(|p| p.as_ref())
    }

    // ── Config validation ────────────────────────────────────

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
    /// only explicit params so callers don't need a TtsService handle.
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

    // ── Text segmentation ────────────────────────────────────

    pub fn segment_chapter(
        &self,
        chapter_index: usize,
        paragraphs: &[Paragraph],
        config: &TtsConfig,
    ) -> Vec<Segment> {
        let max_chars = self
            .get_provider(config.provider.clone())
            .map(|p| p.max_text_length())
            .unwrap_or(2000);
        segment_chapter(chapter_index, paragraphs, max_chars)
    }

    // ── Playback control ─────────────────────────────────────

    pub fn play_audio(&self, data: Vec<u8>, media_type: &str) -> Result<(), String> {
        let player = self.player.as_ref().ok_or("音频播放器不可用（无声模式）")?;
        if media_type == "audio/pcm16" {
            player.append_pcm16(data)
        } else {
            player.append(data)
        }
    }

    pub fn stop_playback(&self) {
        if let Some(player) = &self.player {
            player.stop();
        }
    }

    pub fn pause_playback(&self) {
        if let Some(player) = &self.player {
            player.pause();
        }
    }

    pub fn resume_playback(&self) {
        if let Some(player) = &self.player {
            player.play();
        }
    }

    pub fn is_playing(&self) -> bool {
        self.player
            .as_ref()
            .map(|p| !p.is_paused() && !p.is_empty())
            .unwrap_or(false)
    }

    /// Whether the current audio segment has finished playing.
    /// Returns true when the rodio sink has consumed all queued audio.
    pub fn is_playback_done(&self) -> bool {
        self.player
            .as_ref()
            .map(|p| p.is_empty())
            .unwrap_or(false)
    }

    // ── Cache management ─────────────────────────────────────

    pub fn clear_cache(&self) -> std::io::Result<()> {
        self.cache.clear_all()
    }

    pub fn clear_book_cache(&self, book_id: &str) -> std::io::Result<()> {
        self.cache.clear_book(book_id)
    }
}
