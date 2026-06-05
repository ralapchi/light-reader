import { useEffect, useMemo, useRef, useState } from 'react'
import { useParams } from 'react-router-dom'
import useAppStore from '../store/useAppStore'
import { useSettingsPersistence } from '../hooks/useSettingsPersistence'
import type { ReaderSettings } from '../services/api'
import { useReaderSearch } from './reader/useReaderSearch'
import { useChapterNavigation } from './reader/useChapterNavigation'
import { useReadingProgress } from './reader/useReadingProgress'
import { useBookmarks } from './reader/useBookmarks'
import { useTtsReader } from './reader/useTtsReader'
import { captureVisibleParagraph, flattenToc } from './reader/readerUtils'
import { useReaderKeyboard } from './reader/useReaderKeyboard'
import { useReaderStyles } from './reader/useReaderStyles'
import { usePreservePositionOnModeChange } from './reader/usePreservePositionOnModeChange'
import ReaderContent from './reader/ReaderContent'
import type { TwoPageNav, TwoPageVisibleChapter } from './reader/TwoPageReaderContent'
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
  const twoPageNavRef = useRef<TwoPageNav | null>(null)
  const [activePanel, setActivePanel] = useState<SettingsPanel>(null)
  const [winW, setWinW] = useState(() => window.innerWidth)
  const [twoPageVisibleChapter, setTwoPageVisibleChapter] = useState<TwoPageVisibleChapter | null>(null)

  const { reader, toggleToc, closeToc } = useAppStore()
  const { book, currentChapterIndex, currentChapter, progressPercent, showToc, showSearch, settings, tts } = reader


  useEffect(() => {
    const onResize = () => setWinW(window.innerWidth)
    window.addEventListener('resize', onResize)
    return () => window.removeEventListener('resize', onResize)
  }, [])

  const isTwoPageAvailable = winW >= TWO_PAGE_MIN_WIDTH
  const effectiveReadingMode = isTwoPageAvailable ? settings.reading_mode : 'ChapterScroll'

  useEffect(() => {
    if (effectiveReadingMode !== 'TwoPage') setTwoPageVisibleChapter(null)
  }, [effectiveReadingMode])

  const { layoutAnchorParagraph, setLayoutAnchorParagraph } = usePreservePositionOnModeChange(effectiveReadingMode, contentRef, twoPageNavRef)

  const search = useReaderSearch()
  const navigation = useChapterNavigation(bookId, book, contentRef, search.handleClose, effectiveReadingMode, twoPageNavRef)
  const progress = useReadingProgress(bookId, book, currentChapterIndex, contentRef, effectiveReadingMode, twoPageNavRef)
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

  useReaderKeyboard(toggleBookmark)

  const flatToc = useMemo(() => book ? flattenToc(book.toc) : [], [book])
  const displayChapterIndex = effectiveReadingMode === 'TwoPage'
    ? (twoPageVisibleChapter?.chapterIndex ?? currentChapterIndex)
    : currentChapterIndex
  const chapterTitle = effectiveReadingMode === 'TwoPage'
    ? (twoPageVisibleChapter?.title ?? currentChapter?.title ?? '')
    : (currentChapter?.title ?? '')
  const progressDisplay = `${Math.round(progressPercent * 100)}%`

  const isOriginal = settings.theme === 'original'
  const { readerStyle, contentStyle, paragraphStyle } = useReaderStyles(settings, isOriginal)

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
        twoPageNavRef={twoPageNavRef}
        onScroll={progress.handleScroll}
        onLinkClick={handleLinkClick}
        paragraphStyle={paragraphStyle}
        readingMode={effectiveReadingMode}
        onNextChapter={navigation.goToNextChapter}
        onPreviousChapter={navigation.goToPreviousChapter}
        onNavigate={navigation.clearFootnoteReturn}
        onVisibleChapterChange={setTwoPageVisibleChapter}
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
        currentChapterIndex={displayChapterIndex}
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
