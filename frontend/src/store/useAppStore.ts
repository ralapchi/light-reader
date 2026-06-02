import { create } from 'zustand'
import type { LibraryBookCardDto, ReaderBookDto, ReaderChapterDto, BookmarkDto, ReaderSettings, ReaderAnchor } from '../services/api'

// ── Opening state (for loading transition page) ─────────────

type OpeningStatus = 'idle' | 'loading' | 'error'

interface OpeningState {
  bookId: string
  title: string
  author: string | null
  coverUrl: string | null
  status: OpeningStatus
  errorMessage: string | null
}

// ── TTS state ───────────────────────────────────────────────

type TtsStatus = 'idle' | 'buffering' | 'playing' | 'paused' | 'finished' | 'error'

interface TtsState {
  status: TtsStatus
  paragraph_indices: number[]
  segment_index: number
  total_segments: number
  error: string | null
}

// ── Reader state (for reader page) ──────────────────────────

interface ReaderState {
  book: ReaderBookDto | null
  currentChapterIndex: number
  currentChapter: ReaderChapterDto | null
  progressPercent: number
  showToc: boolean
  showSearch: boolean
  bookmarks: BookmarkDto[]
  pendingNavTarget: { chapter_index: number; paragraph_index: number | null; scroll_offset: number | null; anchor?: ReaderAnchor | null } | null
  settings: ReaderSettings
  tts: TtsState
}

interface AppState {
  // Library
  books: LibraryBookCardDto[]

  // Sidebar
  sidebarFooter: string
  sidebarCollapsed: boolean
  setSidebarFooter: (text: string) => void
  toggleSidebar: () => void

  // Opening transition
  opening: OpeningState

  // Reader
  reader: ReaderState

  // Library actions
  setBooks: (books: LibraryBookCardDto[]) => void

  // Opening actions
  startOpening: (bookId: string, title: string, author: string | null, coverUrl: string | null) => void
  setOpeningError: (message: string) => void

  // Reader actions
  setReaderBook: (book: ReaderBookDto) => void
  setCurrentChapter: (index: number, chapter: ReaderChapterDto) => void
  setProgressPercent: (pct: number) => void
  toggleToc: () => void
  toggleSearch: () => void
  closeToc: () => void
  closeSearch: () => void
  setBookmarks: (bookmarks: BookmarkDto[]) => void
  setPendingNavTarget: (target: { chapter_index: number; paragraph_index: number | null; scroll_offset: number | null; anchor?: ReaderAnchor | null } | null) => void
  setSettings: (settings: Partial<ReaderSettings>) => void
  setTtsState: (partial: Partial<TtsState>) => void
  resetTts: () => void
}

const defaultOpening: OpeningState = {
  bookId: '',
  title: '',
  author: null,
  coverUrl: null,
  status: 'idle',
  errorMessage: null,
}

const defaultSettings: ReaderSettings = {
  theme: 'original',
  app_theme: 'system',
  font_family: 'sans-serif',
  font_size: 17,
  line_height: 1.85,
  paragraph_spacing: 1.2,
  content_width: 600,
  side_margin: 32,
  toc_width: 300,
  reading_mode: 'ChapterScroll',
  auto_save_progress: true,
  show_status_bar: true,
  show_chapter_progress: true,
  smooth_scroll: true,
  open_last_book_on_startup: false,
  restore_last_position: true,
  window_padding: 0,
  auto_page_turn: false,
}

const defaultTts: TtsState = {
  status: 'idle',
  paragraph_indices: [],
  segment_index: 0,
  total_segments: 0,
  error: null,
}

const defaultReader: ReaderState = {
  book: null,
  currentChapterIndex: 0,
  currentChapter: null,
  progressPercent: 0,
  showToc: false,
  showSearch: false,
  bookmarks: [],
  pendingNavTarget: null,
  settings: { ...defaultSettings },
  tts: { ...defaultTts },
}

/**
 * Global Zustand store for the reader frontend.
 */
const useAppStore = create<AppState>((set) => ({
  books: [],
  sidebarFooter: '',
  sidebarCollapsed: false,
  setSidebarFooter: (text) => set({ sidebarFooter: text }),
  toggleSidebar: () => set(s => ({ sidebarCollapsed: !s.sidebarCollapsed })),
  opening: { ...defaultOpening },
  reader: { ...defaultReader },

  setBooks: (books) => set({ books }),

  startOpening: (bookId, title, author, coverUrl) =>
    set({ opening: { bookId, title, author, coverUrl, status: 'loading', errorMessage: null } }),
  setOpeningError: (message) =>
    set((s) => ({ opening: { ...s.opening, status: 'error', errorMessage: message } })),

  setReaderBook: (book) =>
    set((s) => ({ reader: { ...s.reader, book } })),
  setCurrentChapter: (index, chapter) =>
    set((s) => ({ reader: { ...s.reader, currentChapterIndex: index, currentChapter: chapter } })),
  setProgressPercent: (pct) =>
    set((s) => ({ reader: { ...s.reader, progressPercent: pct } })),
  toggleToc: () =>
    set((s) => ({ reader: { ...s.reader, showToc: !s.reader.showToc, showSearch: false } })),
  toggleSearch: () =>
    set((s) => ({ reader: { ...s.reader, showSearch: !s.reader.showSearch, showToc: false } })),
  closeToc: () =>
    set((s) => ({ reader: { ...s.reader, showToc: false } })),
  closeSearch: () =>
    set((s) => ({ reader: { ...s.reader, showSearch: false } })),
  setBookmarks: (bookmarks) =>
    set((s) => ({ reader: { ...s.reader, bookmarks } })),
  setPendingNavTarget: (target) =>
    set((s) => ({ reader: { ...s.reader, pendingNavTarget: target } })),
  setSettings: (partial) =>
    set((s) => ({ reader: { ...s.reader, settings: { ...s.reader.settings, ...partial } } })),
  setTtsState: (partial) =>
    set((s) => ({ reader: { ...s.reader, tts: { ...s.reader.tts, ...partial } } })),
  resetTts: () =>
    set((s) => ({ reader: { ...s.reader, tts: { ...defaultTts } } })),
}))

export default useAppStore
