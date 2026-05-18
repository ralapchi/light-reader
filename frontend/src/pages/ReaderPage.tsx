import { useEffect, useMemo, useRef, useState, type CSSProperties } from 'react'
import { useParams } from 'react-router-dom'
import useAppStore from '../store/useAppStore'
import { findReaderTheme, readerFontFamily } from '../utils/readerOptions'
import { useSettingsPersistence } from '../hooks/useSettingsPersistence'
import { useReaderSearch } from './reader/useReaderSearch'
import { useChapterNavigation } from './reader/useChapterNavigation'
import { useReadingProgress } from './reader/useReadingProgress'
import { useBookmarks } from './reader/useBookmarks'
import { useTtsReader } from './reader/useTtsReader'
import { flattenToc } from './reader/readerUtils'
import ReaderContent from './reader/ReaderContent'
import ReaderSearchPanel from './reader/ReaderSearchPanel'
import ReaderSettingsControls, { type SettingsPanel } from './reader/ReaderSettingsControls'
import ReaderStatusBar from './reader/ReaderStatusBar'
import ReaderTocPanel from './reader/ReaderTocPanel'
import ReaderToolbar from './reader/ReaderToolbar'
import './ReaderPage.css'

function ReaderPage() {
  const { bookId } = useParams<{ bookId: string }>()
  const contentRef = useRef<HTMLDivElement>(null)
  const [activePanel, setActivePanel] = useState<SettingsPanel>(null)

  const { reader, toggleToc, closeToc } = useAppStore()
  const { book, currentChapterIndex, currentChapter, progressPercent, showToc, showSearch, settings, tts } = reader

  const search = useReaderSearch()
  const navigation = useChapterNavigation(bookId, book, contentRef, search.handleClose)
  const progress = useReadingProgress(bookId, book, currentChapterIndex, contentRef)
  const bookmarks = useBookmarks(bookId, currentChapterIndex, contentRef)
  const ttsReader = useTtsReader(contentRef)
  const updateAndSave = useSettingsPersistence()

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'b') {
        e.preventDefault()
        bookmarks.toggleBookmark()
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [bookmarks.toggleBookmark])

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
        contentRef={contentRef}
        contentStyle={contentStyle}
        highlightedParagraphIndex={tts.paragraph_indices[0]}
        imageCache={navigation.imageCache}
        onScroll={progress.handleScroll}
        paragraphStyle={paragraphStyle}
      />

      <ReaderStatusBar
        chapterTitle={chapterTitle}
        currentChapterIndex={currentChapterIndex}
        progressDisplay={progressDisplay}
        tts={tts}
      />

      <ReaderSettingsControls
        activePanel={activePanel}
        onPanelChange={setActivePanel}
        onUpdateSettings={updateAndSave}
        settings={settings}
      />
    </div>
  )
}

export default ReaderPage
