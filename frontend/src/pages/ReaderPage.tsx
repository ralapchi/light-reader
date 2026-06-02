import { useEffect, useMemo, useRef, useState, type CSSProperties } from 'react'
import { useParams } from 'react-router-dom'
import useAppStore from '../store/useAppStore'
import { findReaderTheme, readerFontFamily } from '../utils/readerOptions'
import { useSettingsPersistence } from '../hooks/useSettingsPersistence'
import type { ReaderSettings } from '../services/api'
import { useReaderSearch } from './reader/useReaderSearch'
import { useChapterNavigation } from './reader/useChapterNavigation'
import { useReadingProgress } from './reader/useReadingProgress'
import { useBookmarks } from './reader/useBookmarks'
import { useTtsReader } from './reader/useTtsReader'
import { captureVisibleParagraph, flattenToc, scrollToParagraph, scrollToParagraphTwoPage } from './reader/readerUtils'
import ReaderContent from './reader/ReaderContent'
import ReaderSearchPanel from './reader/ReaderSearchPanel'
import ReaderSettingsControls, { type SettingsPanel } from './reader/ReaderSettingsControls'
import ReaderStatusBar from './reader/ReaderStatusBar'
import ReaderTocPanel from './reader/ReaderTocPanel'
import ReaderToolbar from './reader/ReaderToolbar'
import './ReaderPage.css'

const TWO_PAGE_MIN_WIDTH = 960

function ReaderPage() {
  const { bookId } = useParams<{ bookId: string }>()
  const contentRef = useRef<HTMLDivElement>(null)
  const [activePanel, setActivePanel] = useState<SettingsPanel>(null)
  const [layoutAnchorParagraph, setLayoutAnchorParagraph] = useState<number | null>(null)
  const [winW, setWinW] = useState(() => window.innerWidth)

  const { reader, toggleToc, closeToc } = useAppStore()
  const { book, currentChapterIndex, currentChapter, progressPercent, showToc, showSearch, settings, tts } = reader


  useEffect(() => {
    const onResize = () => setWinW(window.innerWidth)
    window.addEventListener('resize', onResize)
    return () => window.removeEventListener('resize', onResize)
  }, [])

  const isTwoPageAvailable = winW >= TWO_PAGE_MIN_WIDTH
  const effectiveReadingMode = isTwoPageAvailable ? settings.reading_mode : 'ChapterScroll'

  // Preserve reading position when layout mode changes due to resize
  const prevModeRef = useRef(effectiveReadingMode)
  useEffect(() => {
    const prev = prevModeRef.current
    prevModeRef.current = effectiveReadingMode
    if (prev === effectiveReadingMode) return
    const el = contentRef.current
    if (!el) return
    const captured = layoutAnchorParagraph ?? captureVisibleParagraph(el, prev === 'TwoPage')
    if (captured == null) return
    if (!layoutAnchorParagraph) setLayoutAnchorParagraph(captured)
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        const el2 = contentRef.current
        if (!el2) return
        if (effectiveReadingMode === 'TwoPage') scrollToParagraphTwoPage(el2, captured)
        else scrollToParagraph(el2, captured)
      })
    })
  }, [effectiveReadingMode, layoutAnchorParagraph])

  const search = useReaderSearch()
  const navigation = useChapterNavigation(bookId, book, contentRef, search.handleClose, effectiveReadingMode)
  const progress = useReadingProgress(bookId, book, currentChapterIndex, contentRef, effectiveReadingMode)
  const bookmarks = useBookmarks(bookId, currentChapterIndex, contentRef)
  const { toggleBookmark } = bookmarks
  const ttsReader = useTtsReader(contentRef)
  const updateAndSave = useSettingsPersistence()

  const handleUpdateSettings = (partial: Partial<ReaderSettings>) => {
    if (partial.reading_mode && partial.reading_mode !== effectiveReadingMode) {
      const el = contentRef.current
      setLayoutAnchorParagraph(el ? captureVisibleParagraph(el, effectiveReadingMode === 'TwoPage') : null)
      progress.saveCurrentPosition()
    }
    updateAndSave(partial)
  }

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'b') {
        e.preventDefault()
        toggleBookmark()
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [toggleBookmark])

  const flatToc = useMemo(() => book ? flattenToc(book.toc) : [], [book])
  const chapterTitle = currentChapter?.title ?? ''
  const progressDisplay = `${Math.round(progressPercent * 100)}%`

  const isOriginal = settings.theme === 'original'
  const currentTheme = findReaderTheme(settings.theme)
  const readerStyle = useMemo(() => ({
    '--bg': currentTheme.bg,
    '--text-primary': currentTheme.text,
    '--accent': currentTheme.accent,
    '--border': currentTheme.border,
    '--surface': currentTheme.surface,
    '--text-secondary': currentTheme.textSec,
    '--text-tertiary': currentTheme.textTer,
    '--surface-hover': currentTheme.hover,
    '--accent-soft': currentTheme.accentSoft,
  } as CSSProperties), [currentTheme])

  const contentStyle = useMemo<CSSProperties>(() => isOriginal ? {} : {
    fontFamily: readerFontFamily(settings.font_family),
    fontSize: `${settings.font_size}px`,
    lineHeight: settings.line_height,
  }, [isOriginal, settings.font_family, settings.font_size, settings.line_height])

  const paragraphStyle = useMemo<CSSProperties>(() => isOriginal ? {} : {
    marginBottom: `${settings.paragraph_spacing}em`,
  }, [isOriginal, settings.paragraph_spacing])

  const handleLinkClick = (href: string) => {
    navigation.navigateToHref(href)
  }

  return (
    <div className="reader-app" style={readerStyle}>
      <div className="toolbar-trigger" />

      <ReaderToolbar
        book={book}
        chapterTitle={chapterTitle}
        currentBookmark={bookmarks.currentBookmark}
        onBack={navigation.goBackToLibrary}
        onNextChapter={navigation.goToNextChapter}
        onPreviousChapter={navigation.goToPreviousChapter}
        onToggleBookmark={bookmarks.toggleBookmark}
        onToggleSearch={search.handleToggle}
        onToggleToc={toggleToc}
        onTtsStop={ttsReader.handleTtsStop}
        onTtsToggle={ttsReader.handleTtsToggle}
        ttsStatus={tts.status}
      />

      <ReaderTocPanel
        currentChapterIndex={currentChapterIndex}
        items={flatToc}
        onClose={closeToc}
        onGoToChapter={navigation.goToChapter}
        onNavigateHref={(href, fallback) => navigation.navigateToHref(href, fallback, { showReturn: false })}
        progressDisplay={progressDisplay}
        progressPercent={progressPercent}
        visible={showToc}
      />

      <ReaderSearchPanel
        onClose={search.handleClose}
        onInput={search.handleInput}
        onResultClick={navigation.handleSearchResultClick}
        query={search.searchQuery}
        results={search.searchResults}
        visible={showSearch}
      />

      <div className="reading-progress-track">
        <div className="reading-progress-fill" style={{ width: `${Math.round(progressPercent * 100)}%` }} />
      </div>

      <ReaderContent
        chapter={currentChapter}
        chapterCount={book?.chapter_count ?? 1}
        contentRef={contentRef}
        contentStyle={contentStyle}
        contentWidth={settings.content_width}
        highlightedParagraphIndex={tts.paragraph_indices[0]}
        imageCache={navigation.imageCache}
        initialParagraphIndex={layoutAnchorParagraph}
        onScroll={progress.handleScroll}
        onLinkClick={handleLinkClick}
        paragraphStyle={paragraphStyle}
        readingMode={effectiveReadingMode}
        onNextChapter={navigation.goToNextChapter}
        onPreviousChapter={navigation.goToPreviousChapter}
        onNavigate={navigation.clearFootnoteReturn}
      />

      {navigation.footnoteReturn && (
        <button
          className="footnote-return-btn"
          onClick={navigation.returnFromFootnote}
          title="返回原文"
          aria-label="返回原文"
        >
          <svg width="19" height="19" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round">
            <path d="m9 14-5-5 5-5" />
            <path d="M4 9h10a6 6 0 0 1 0 12h-1" />
          </svg>
        </button>
      )}

      <ReaderStatusBar
        chapterTitle={chapterTitle}
        currentChapterIndex={currentChapterIndex}
        progressDisplay={progressDisplay}
        tts={tts}
      />

      <ReaderSettingsControls
        activePanel={activePanel}
        isTwoPageAvailable={isTwoPageAvailable}
        onPanelChange={setActivePanel}
        onUpdateSettings={handleUpdateSettings}
        settings={settings}
      />
    </div>
  )
}

export default ReaderPage
