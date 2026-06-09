use crate::tts::cache::TtsCache;
use crate::tts::config::TtsConfig;
use crate::tts::segmenter::Segment;
use crate::tts::synthesis_service::TtsSynthesisService;
use crate::tts::types::PlaybackStatus;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;

use super::super::dto::*;
use super::dto_convert::dto_to_tts_config;
use super::{PlaybackCmd, ReaderSession};

/// Spawn a dedicated audio playback thread (rodio OutputStream is !Send).
fn spawn_playback_thread() -> (mpsc::Sender<PlaybackCmd>, Arc<AtomicBool>) {
    let (tx, rx) = mpsc::channel::<PlaybackCmd>();
    let is_playing = Arc::new(AtomicBool::new(false));

    let playing_flag = Arc::clone(&is_playing);
    std::thread::spawn(move || {
        use crate::tts::player::AudioPlayer;
        let mut player = match AudioPlayer::new() {
            Ok(p) => Some(p),
            Err(e) => {
                log::warn!("音频播放器初始化失败: {}", e);
                // Drain commands without panicking
                for cmd in rx {
                    if matches!(cmd, PlaybackCmd::Stop) {
                        break;
                    }
                }
                return;
            }
        };

        loop {
            let p = match player.as_mut() {
                Some(p) => p,
                None => break,
            };

            // Detect when sink finishes playing naturally
            if playing_flag.load(Ordering::Relaxed) && p.is_empty() && !p.is_paused() {
                playing_flag.store(false, Ordering::Relaxed);
            }

            match rx.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(cmd) => match cmd {
                    PlaybackCmd::Play(data, media_type) => {
                        let res = if media_type == "audio/pcm16" {
                            p.append_pcm16(data)
                        } else {
                            p.append(data)
                        };
                        if let Err(e) = res {
                            log::warn!("TTS 音频播放失败: {}", e);
                        }
                        playing_flag.store(true, Ordering::Relaxed);
                    }
                    PlaybackCmd::Pause => {
                        p.pause();
                    }
                    PlaybackCmd::Resume => {
                        p.play();
                        playing_flag.store(true, Ordering::Relaxed);
                    }
                    PlaybackCmd::Stop => {
                        p.stop();
                        playing_flag.store(false, Ordering::Relaxed);
                        break;
                    }
                },
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });

    (tx, is_playing)
}

/// Get voice_id from config, with default fallback.
fn tts_voice_id(config: &TtsConfig) -> String {
    config
        .voice_id
        .clone()
        .unwrap_or_else(|| "default_zh".to_string())
}

/// Get the max text length from a provider for the given config.
/// TODO: query provider dynamically when multiple providers are supported.
fn tts_max_text_length(_config: &TtsConfig) -> usize {
    200 // Xiaomi provider limit
}

fn synthesize_and_play(
    segment: &Segment,
    book_id: &str,
    chapter_index: usize,
    config: &TtsConfig,
    cache: &Arc<TtsCache>,
    playback_tx: &mpsc::Sender<PlaybackCmd>,
    app: &tauri::AppHandle,
) -> Result<(), String> {
    use crate::tauri_api::emitter::EventEmitter;
    use crate::tauri_api::events::{TtsBuffering, TtsError};
    let emitter = EventEmitter::new(app);

    emitter.tts_buffering(&TtsBuffering {
        book_id: book_id.to_string(),
        chapter_index,
        segment_index: segment.segment_index,
    });

    let voice_id = tts_voice_id(config);
    let request = crate::tts::types::TtsRequest {
        book_id: book_id.to_string(),
        chapter_index,
        segment_index: segment.segment_index,
        paragraph_indices: segment.paragraph_indices.clone(),
        text: segment.text.clone(),
        voice_id: Some(voice_id.clone()),
    };

    // Check cache first
    let segment_path = cache.segment_path(
        &format!("{:?}", config.provider).to_lowercase(),
        book_id,
        chapter_index,
        segment.segment_index,
        &voice_id,
        "pcm16",
    );
    let audio_result = if segment_path.exists() {
        std::fs::read(&segment_path)
            .ok()
            .map(|bytes| (bytes, "audio/pcm16".to_string()))
    } else {
        None
    };

    let (audio_bytes, media_type) = match audio_result {
        Some(r) => r,
        None => {
            // Synthesize on current thread (blocking)
            match TtsSynthesisService::synthesize_blocking(&request, config, &voice_id, cache) {
                Ok(resp) => (resp.audio_bytes, resp.media_type),
                Err(e) => {
                    let msg = format!("TTS 合成失败: {}", e);
                    log::error!("{}", msg);
                    emitter.tts_error(&TtsError {
                        book_id: Some(book_id.to_string()),
                        error_message: msg.clone(),
                    });
                    return Err(msg);
                }
            }
        }
    };

    // Send to playback thread
    playback_tx
        .send(PlaybackCmd::Play(audio_bytes, media_type))
        .map_err(|e| {
            let msg = format!("TTS 播放线程不可用: {}", e);
            emitter.tts_error(&TtsError {
                book_id: Some(book_id.to_string()),
                error_message: msg.clone(),
            });
            msg
        })?;
    Ok(())
}

#[tauri::command]
pub fn tts_test_connection(
    config: TtsConfigDto,
    state: tauri::State<'_, ReaderSession>,
) -> Result<bool, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    let full_config = dto_to_tts_config(&config, guard.tts_config.api_key.clone());
    let mut svc = TtsSynthesisService::new(crate::storage::paths::tts_cache_dir());
    svc.register_provider(Box::new(
        crate::tts::xiaomi_provider::XiaomiTtsProvider::new(),
    ));
    match svc.test_connection(&full_config) {
        Ok(()) => Ok(true),
        Err(e) => Err(format!("{}", e)),
    }
}

#[tauri::command]
pub fn tts_start(
    chapter_index: usize,
    state: tauri::State<'_, ReaderSession>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    use crate::tauri_api::emitter::EventEmitter;
    use crate::tauri_api::events::TtsPlaying;
    let emitter = EventEmitter::new(&app);

    let mut guard = state.lock().map_err(|e| e.to_string())?;

    // Stop any existing playback first
    if let Some(tx) = &guard.playback_tx {
        let _ = tx.send(PlaybackCmd::Stop);
    }
    guard.stop_flag.store(true, Ordering::Relaxed);
    guard.playback_state = Default::default();
    guard.playback_state.current_chapter_index = Some(chapter_index);

    // Reset stop flag for new session
    guard.stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag = Arc::clone(&guard.stop_flag);

    // Get current chapter
    let book = guard.book.as_ref().ok_or("没有打开的书籍")?;
    let book_id = book.id.clone();
    let chapter = book
        .chapters
        .get(chapter_index)
        .ok_or_else(|| format!("章节 {} 不存在", chapter_index))?;
    let paragraphs: Vec<_> = chapter.text_paragraphs().cloned().collect();

    if paragraphs.is_empty() {
        return Err("当前章节没有内容".to_string());
    }

    let config = guard.tts_config.clone();
    let cache = Arc::clone(&guard.cache);

    // Segment the chapter
    let max_chars = tts_max_text_length(&config);
    let segments = crate::tts::segmenter::segment_chapter(chapter_index, &paragraphs, max_chars);
    if segments.is_empty() {
        return Err("章节分割结果为空".to_string());
    }

    let total_segments = segments.len();
    let segments_for_poll = segments.clone();
    guard.segments = segments.clone();

    // Spawn playback thread
    let (playback_tx, is_playing_flag) = spawn_playback_thread();
    guard.playback_tx = Some(playback_tx.clone());
    guard.is_playing_flag = Arc::clone(&is_playing_flag);

    // Synthesize and play segment 0
    let segment = segments[0].clone();
    let paragraph_indices = segment.paragraph_indices.clone();

    guard.playback_state.status = PlaybackStatus::Buffering;
    guard.playback_state.current_book_id = Some(book_id.clone());
    guard.playback_state.current_chapter_index = Some(chapter_index);
    guard.playback_state.total_segments = total_segments;
    drop(guard);

    if let Err(e) = synthesize_and_play(
        &segment,
        &book_id,
        chapter_index,
        &config,
        &cache,
        &playback_tx,
        &app,
    ) {
        let mut guard = state.lock().map_err(|lock_err| lock_err.to_string())?;
        guard.stop_flag.store(true, Ordering::Relaxed);
        guard.playback_state.status = PlaybackStatus::Error(e.clone());
        guard.playback_tx = None;
        guard.is_playing_flag.store(false, Ordering::Relaxed);
        return Err(e);
    }

    let mut guard = state.lock().map_err(|e| e.to_string())?;

    // Update playback state
    guard.playback_state.status = PlaybackStatus::Playing;
    guard.playback_state.current_book_id = Some(book_id.clone());
    guard.playback_state.current_chapter_index = Some(chapter_index);
    guard.playback_state.current_segment_index = Some(0);
    guard.playback_state.current_paragraph_indices = paragraph_indices.clone();
    guard.playback_state.total_segments = total_segments;

    emitter.tts_playing(&TtsPlaying {
        book_id: book_id.clone(),
        chapter_index,
        segment_index: 0,
        total_segments,
        paragraph_indices: paragraph_indices.clone(),
    });
    drop(guard);

    // Pre-fetch segment 1
    if segments.len() > 1 {
        let next = segments[1].clone();
        let cfg = config.clone();
        let c = Arc::clone(&cache);
        let bid = book_id.clone();
        let vid = tts_voice_id(&config);
        std::thread::spawn(move || {
            let req = crate::tts::types::TtsRequest {
                book_id: bid,
                chapter_index,
                segment_index: next.segment_index,
                paragraph_indices: next.paragraph_indices,
                text: next.text,
                voice_id: Some(vid.clone()),
            };
            let _ = TtsSynthesisService::synthesize_blocking(&req, &cfg, &vid, &c);
        });
    }

    // Start polling thread for auto-advance
    let poll_playing = Arc::clone(&is_playing_flag);
    let poll_stop = Arc::clone(&stop_flag);
    let poll_tx = playback_tx.clone();
    let poll_app = app.clone();
    let poll_segments = segments_for_poll;
    let poll_config = config.clone();
    let poll_cache = Arc::clone(&cache);
    std::thread::spawn(move || {
        let emitter = EventEmitter::new(&poll_app);
        let mut current_seg_idx: usize = 0;
        let mut was_playing = true;

        loop {
            if poll_stop.load(Ordering::Relaxed) {
                break;
            }

            // Auto-advance when playback finishes
            let currently_playing = poll_playing.load(Ordering::Relaxed);
            if was_playing && !currently_playing && !poll_stop.load(Ordering::Relaxed) {
                was_playing = false;
                current_seg_idx += 1;
                if current_seg_idx >= total_segments {
                    emitter.tts_finished(&crate::tauri_api::events::TtsFinished {
                        book_id: book_id.clone(),
                        chapter_index,
                    });
                    break;
                }
                if let Some(next_seg) = poll_segments.get(current_seg_idx) {
                    if synthesize_and_play(
                        next_seg,
                        &book_id,
                        chapter_index,
                        &poll_config,
                        &poll_cache,
                        &poll_tx,
                        &poll_app,
                    )
                    .is_err()
                    {
                        break;
                    }
                    emitter.tts_playing(&crate::tauri_api::events::TtsPlaying {
                        book_id: book_id.clone(),
                        chapter_index,
                        segment_index: current_seg_idx,
                        total_segments,
                        paragraph_indices: next_seg.paragraph_indices.clone(),
                    });
                    was_playing = true;
                }
            }

            if poll_stop.load(Ordering::Relaxed) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    });

    Ok(())
}

#[tauri::command]
pub fn tts_pause(
    state: tauri::State<'_, ReaderSession>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    use crate::tauri_api::emitter::EventEmitter;
    use crate::tauri_api::events::TtsPaused;
    let emitter = EventEmitter::new(&app);
    let mut guard = state.lock().map_err(|e| e.to_string())?;

    if let Some(tx) = &guard.playback_tx {
        tx.send(PlaybackCmd::Pause).ok();
    }
    guard.playback_state.status = PlaybackStatus::Paused;

    let book_id = guard
        .playback_state
        .current_book_id
        .clone()
        .unwrap_or_default();
    let seg_idx = guard.playback_state.current_segment_index.unwrap_or(0);
    emitter.tts_paused(&TtsPaused {
        book_id,
        segment_index: seg_idx,
    });
    Ok(())
}

#[tauri::command]
pub fn tts_resume(
    state: tauri::State<'_, ReaderSession>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    use crate::tauri_api::emitter::EventEmitter;
    use crate::tauri_api::events::TtsPlaying;
    let emitter = EventEmitter::new(&app);
    let mut guard = state.lock().map_err(|e| e.to_string())?;

    if let Some(tx) = &guard.playback_tx {
        tx.send(PlaybackCmd::Resume).ok();
    }
    guard.playback_state.status = PlaybackStatus::Playing;
    guard.is_playing_flag.store(true, Ordering::Relaxed);

    let book_id = guard
        .playback_state
        .current_book_id
        .clone()
        .unwrap_or_default();
    let chapter_index = guard.playback_state.current_chapter_index.unwrap_or(0);
    let seg_idx = guard.playback_state.current_segment_index.unwrap_or(0);
    let total = guard.playback_state.total_segments;
    let para = guard.playback_state.current_paragraph_indices.clone();
    emitter.tts_playing(&TtsPlaying {
        book_id,
        chapter_index,
        segment_index: seg_idx,
        total_segments: total,
        paragraph_indices: para,
    });
    Ok(())
}

#[tauri::command]
pub fn tts_stop(
    state: tauri::State<'_, ReaderSession>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    use crate::tauri_api::emitter::EventEmitter;
    use crate::tauri_api::events::TtsStopped;
    let emitter = EventEmitter::new(&app);
    let mut guard = state.lock().map_err(|e| e.to_string())?;

    guard.stop_flag.store(true, Ordering::Relaxed);
    if let Some(tx) = &guard.playback_tx {
        let _ = tx.send(PlaybackCmd::Stop);
    }
    guard.playback_state = Default::default();
    guard.segments.clear();
    guard.playback_tx = None;
    guard.is_playing_flag.store(false, Ordering::Relaxed);

    emitter.tts_stopped(&TtsStopped {});
    Ok(())
}

#[tauri::command]
pub fn tts_clear_cache(state: tauri::State<'_, ReaderSession>) -> Result<(), String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    guard.cache.clear_all().map_err(|e| e.to_string())
}
