//! Event emission helpers.
//!
//! Wraps `tauri::AppHandle::emit` so callers don't need to know event names.
//! Each method emits a typed event that the frontend can listen for.

use tauri::Emitter;

use super::events::*;

/// Thin wrapper around Tauri's event system.
///
/// Create one per command invocation (cheap clone of AppHandle).
/// Methods are no-ops if the Tauri window is not yet ready.
pub struct EventEmitter {
    app: tauri::AppHandle,
}

impl EventEmitter {
    pub fn new(app: &tauri::AppHandle) -> Self {
        Self { app: app.clone() }
    }

    // ── Book Opening ───────────────────────────────────────

    pub fn book_opening_started(&self, event: &BookOpeningStarted) {
        let _ = self.app.emit("book-opening-started", event);
    }

    pub fn book_opening_progress(&self, event: &BookOpeningProgress) {
        let _ = self.app.emit("book-opening-progress", event);
    }

    pub fn book_opening_finished(&self, event: &BookOpeningFinished) {
        let _ = self.app.emit("book-opening-finished", event);
    }

    pub fn book_opening_failed(&self, event: &BookOpeningFailed) {
        let _ = self.app.emit("book-opening-failed", event);
    }

    // ── TTS ────────────────────────────────────────────────

    pub fn tts_buffering(&self, event: &TtsBuffering) {
        let _ = self.app.emit("tts-buffering", event);
    }

    pub fn tts_playing(&self, event: &TtsPlaying) {
        let _ = self.app.emit("tts-playing", event);
    }

    pub fn tts_finished(&self, event: &TtsFinished) {
        let _ = self.app.emit("tts-finished", event);
    }

    pub fn tts_paused(&self, event: &TtsPaused) {
        let _ = self.app.emit("tts-paused", event);
    }

    pub fn tts_stopped(&self, event: &TtsStopped) {
        let _ = self.app.emit("tts-stopped", event);
    }

    pub fn tts_error(&self, event: &TtsError) {
        let _ = self.app.emit("tts-error", event);
    }
}
