/**
 * Frontend API layer.
 *
 * All Tauri command invocations go through here.
 * Wraps @tauri-apps/api invoke() with typed helpers.
 */

import { invoke, convertFileSrc } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

// ── Types (mirrors Rust DTOs) ──────────────────────────────

export interface LibraryBookCardDto {
  book_id: string
  title: string
  author: string | null
  format: string
  cover_url: string | null
  progress_percent: number
  chapter_count: number
  last_opened_at: string | null
  imported_at: string
  file_ok: boolean
}

export interface ReaderBookDto {
  book_id: string
  title: string
  author: string | null
  format: string
  chapter_count: number
  toc: TocItemDto[]
}

export interface TocItemDto {
  id: string
  title: string
  chapter_index: number | null
  href: string | null
  depth: number
  children: TocItemDto[]
}

export interface ReaderTextLinkDto {
  start: number
  end: number
  href: string
  title: string | null
}

export type ReaderBlockDto =
  | { type: 'paragraph'; index: number; block_id: string; text: string; kind: string; links?: ReaderTextLinkDto[] }
  | { type: 'heading'; index: number; block_id: string; text: string; kind: string; links?: ReaderTextLinkDto[] }
  | { type: 'quote'; index: number; block_id: string; text: string; links?: ReaderTextLinkDto[] }
  | { type: 'image'; index: number; block_id: string; asset_id: string; alt_text: string | null; caption: string | null }
  | { type: 'separator'; block_id: string }

export interface ReaderResolvedLinkDto {
  chapter_index: number
  paragraph_index: number | null
  block_index: number | null
  scroll_offset: number | null
}

export interface ReaderChapterDto {
  chapter_index: number
  title: string
  blocks: ReaderBlockDto[]
  char_count: number
}

export interface TtsConfigDto {
  enabled: boolean
  provider: string
  has_api_key: boolean
  api_key?: string | null
  base_url: string | null
  model: string | null
  voice_id: string | null
}

export interface ReaderAnchor {
  chapterId: string
  blockId: string
  charOffset: number
}

export type ReadingMode = 'ChapterScroll' | 'TwoPage'

export interface ReaderSettings {
  theme: string
  app_theme: string
  font_family: string
  font_size: number
  line_height: number
  paragraph_spacing: number
  content_width: number
  side_margin: number
  toc_width: number
  reading_mode: ReadingMode
  auto_save_progress: boolean
  show_status_bar: boolean
  show_chapter_progress: boolean
  smooth_scroll: boolean
  open_last_book_on_startup: boolean
  restore_last_position: boolean
  window_padding: number
  auto_page_turn: boolean
}

// ── Library ────────────────────────────────────────────────

export function libraryList(): Promise<LibraryBookCardDto[]> {
  return invoke('library_list')
}

export function libraryImport(paths: string[]): Promise<LibraryBookCardDto[]> {
  return invoke('library_import', { paths })
}

export function libraryRemove(bookId: string, deleteFiles: boolean = false): Promise<void> {
  return invoke('library_remove', { bookId, deleteFiles })
}

export function libraryRemoveBatch(bookIds: string[], deleteFiles: boolean = false): Promise<void> {
  return invoke('library_remove_batch', { bookIds, deleteFiles })
}

export function librarySearch(query: string): Promise<LibraryBookCardDto[]> {
  return invoke('library_search', { query })
}

export function libraryCover(bookId: string): Promise<string | null> {
  return invoke('library_cover', { bookId })
}

export function libraryFlushIndex(): Promise<void> {
  return invoke('library_flush_index')
}

// ── Reader ─────────────────────────────────────────────────

export function readerOpenBook(bookId: string): Promise<ReaderBookDto> {
  return invoke('reader_open_book', { bookId })
}

export function readerGetChapter(chapterIndex: number): Promise<ReaderChapterDto> {
  return invoke('reader_get_chapter', { chapterIndex })
}

export function readerChapterImage(bookId: string, assetId: string): Promise<string | null> {
  return invoke('reader_chapter_image', { bookId, assetId })
}

export function readerChapterImages(bookId: string, assetIds: string[]): Promise<Record<string, string>> {
  return invoke('reader_chapter_images', { bookId, assetIds })
}

export interface SaveProgressDto {
  book_id: string
  chapter_index: number
  progress_percent: number
  paragraph_index?: number | null
  scroll_offset?: number | null
  anchor?: ReaderAnchor | null
  clear_position?: boolean
  revision?: number
}

export function readerSaveProgress(progress: SaveProgressDto): Promise<void> {
  return invoke('reader_save_progress', { progress })
}

export function readerGetProgress(bookId: string): Promise<SaveProgressDto | null> {
  return invoke('reader_get_progress', { bookId })
}

export function readerFlushProgress(): Promise<void> {
  return invoke('reader_flush_progress')
}

export function readerResolveHref(
  href: string,
  fromChapterIndex?: number,
): Promise<ReaderResolvedLinkDto | null> {
  return invoke('reader_resolve_href', { href, fromChapterIndex: fromChapterIndex ?? null })
}

export interface LinkPreviewDto {
  chapter_index: number
  paragraph_index: number | null
  text: string
  title: string | null
}

export function readerGetLinkPreview(href: string, fromChapterIndex: number): Promise<LinkPreviewDto | null> {
  return invoke('reader_get_link_preview', { href, fromChapterIndex })
}

// ── Search / Bookmarks ─────────────────────────────────────

export interface SearchHitDto {
  chapter_index: number
  chapter_title: string
  context: string
  progress_hint: string
  paragraph_index: number
}

export function searchInBook(query: string): Promise<SearchHitDto[]> {
  return invoke('search_in_book', { query })
}

export interface BookmarkDto {
  id: string
  book_id: string
  chapter_index: number
  paragraph_index: number | null
  title: string
  snippet: string
  created_at: string
  note: string | null
}

export function bookmarkList(bookId: string): Promise<BookmarkDto[]> {
  return invoke('bookmark_list', { bookId })
}

export function bookmarkListAll(): Promise<BookmarkDto[]> {
  return invoke('bookmark_list_all')
}

export function bookmarkAdd(
  bookId: string,
  chapterIndex: number,
  paragraphIndex?: number,
  note?: string,
): Promise<BookmarkDto> {
  return invoke('bookmark_add', {
    bookId,
    chapterIndex,
    paragraphIndex: paragraphIndex ?? null,
    note: note ?? null,
  })
}

export function bookmarkRemove(bookId: string, bookmarkId: string): Promise<void> {
  return invoke('bookmark_remove', { bookId, bookmarkId })
}

export function settingsLoad(): Promise<ReaderSettings> {
  return invoke('settings_load')
}

export function settingsSave(settings: ReaderSettings): Promise<void> {
  return invoke('settings_save', { settings })
}

export function ttsConfigLoad(): Promise<TtsConfigDto> {
  return invoke('tts_config_load')
}

export function ttsConfigSave(config: TtsConfigDto): Promise<void> {
  return invoke('tts_config_save', { config })
}

// ── TTS ────────────────────────────────────────────────────

export function ttsTestConnection(config: TtsConfigDto): Promise<boolean> {
  return invoke('tts_test_connection', { config })
}

export function ttsStart(chapterIndex: number): Promise<void> {
  return invoke('tts_start', { chapterIndex })
}

export function ttsPause(): Promise<void> {
  return invoke('tts_pause')
}

export function ttsResume(): Promise<void> {
  return invoke('tts_resume')
}

export function ttsStop(): Promise<void> {
  return invoke('tts_stop')
}

export function ttsClearCache(): Promise<void> {
  return invoke('tts_clear_cache')
}

// ── TTS Events ──────────────────────────────────────────────

export interface TtsPlayingEvent {
  book_id: string
  chapter_index: number
  segment_index: number
  total_segments: number
  paragraph_indices: number[]
}

export interface TtsPausedEvent {
  book_id: string
  segment_index: number
}

export interface TtsBufferingEvent {
  book_id: string
  chapter_index: number
  segment_index: number
}

export interface TtsFinishedEvent {
  book_id: string
  chapter_index: number
}

export interface TtsErrorEvent {
  book_id: string | null
  error_message: string
}

export function onTtsPlaying(cb: (p: TtsPlayingEvent) => void): Promise<UnlistenFn> {
  return listen<TtsPlayingEvent>('tts-playing', e => cb(e.payload))
}

export function onTtsPaused(cb: (p: TtsPausedEvent) => void): Promise<UnlistenFn> {
  return listen<TtsPausedEvent>('tts-paused', e => cb(e.payload))
}

export function onTtsStopped(cb: () => void): Promise<UnlistenFn> {
  return listen('tts-stopped', () => cb())
}

export function onTtsBuffering(cb: (p: TtsBufferingEvent) => void): Promise<UnlistenFn> {
  return listen<TtsBufferingEvent>('tts-buffering', e => cb(e.payload))
}

export function onTtsFinished(cb: (p: TtsFinishedEvent) => void): Promise<UnlistenFn> {
  return listen<TtsFinishedEvent>('tts-finished', e => cb(e.payload))
}

export function onTtsError(cb: (p: TtsErrorEvent) => void): Promise<UnlistenFn> {
  return listen<TtsErrorEvent>('tts-error', e => cb(e.payload))
}

// ── Assets ─────────────────────────────────────────────────

/** Resolve a file path to a Tauri asset:// URL. */
export function assetUrl(path: string): string {
  if (!path) return ''
  return convertFileSrc(path)
}

// ── Block / Anchor Utilities ─────────────────────────────────

/** Derive a stable block identifier from its type and index. */
export function blockId(block: ReaderBlockDto): string {
  if (block.type === 'separator') return 'separator'
  return `${block.type}-${block.index}`
}

/** Serialize a ReaderAnchor to a compact cache key. */
export function anchorKey(anchor: ReaderAnchor): string {
  return `${anchor.chapterId}::${anchor.blockId}::${anchor.charOffset}`
}

/** Parse an anchor key back into a ReaderAnchor. */
export function parseAnchorKey(key: string): ReaderAnchor | null {
  const parts = key.split('::')
  if (parts.length !== 3) return null
  const charOffset = Number(parts[2])
  if (!Number.isInteger(charOffset) || charOffset < 0) return null
  return {
    chapterId: parts[0],
    blockId: parts[1],
    charOffset,
  }
}
