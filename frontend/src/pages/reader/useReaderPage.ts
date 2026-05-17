import { useEffect, useRef, useState, type CSSProperties } from 'react'
import { useParams } from 'react-router-dom'
import useAppStore from '../../store/useAppStore'
import { findReaderTheme, readerFontFamily } from '../../utils/readerOptions'
import { flattenToc } from './readerUtils'
import { useReaderSearch } from './useReaderSearch'
import { useChapterImages } from './useChapterImages'
import { useChapterNavigation } from './useChapterNavigation'
import { useReadingProgress } from './useReadingProgress'
import { useBookmarks } from './useBookmarks'
import { useTtsReader } from './useTtsReader'
import { useSettingsPersistence } from '../../hooks/useSettingsPersistence'

export type ReaderSettingsPanel = 'theme' | 'font' | 'format' | null

export function useReaderPage() {
  const { bookId } = useParams<{ bookId: string }>()
  const contentRef = useRef<HTMLDivElement>(null)
  const [activePanel, setActivePanel] = useState<ReaderSettingsPanel>(null)

  const { reader, toggleToc, closeToc } = useAppStore()
  const { book, currentChapterIndex, currentChapter, progressPercent, showToc, showSearch, settings, tts } = reader

  const search = useReaderSearch()
  const images = useChapterImages(bookId)
  const navigation = useChapterNavigation(bookId, book, contentRef, search.handleClose, images.loadChapterImages)
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

  const flatToc = book ? flattenToc(book.toc) : []
  const chapterTitle = currentChapter?.title ?? ''
  const progressDisplay = `${Math.round(progressPercent * 100)}%`

  const isOriginal = settings.theme === 'original'
  const currentTheme = findReaderTheme(settings.theme)
  const readerStyle = {
    '--bg': currentTheme.bg,
    '--text-primary': currentTheme.text,
    '--accent': currentTheme.accent,
    '--border': currentTheme.border,
    '--surface': currentTheme.surface,
    '--text-secondary': currentTheme.textSec,
    '--text-tertiary': currentTheme.textTer,
    '--surface-hover': currentTheme.hover,
    '--accent-soft': currentTheme.accentSoft,
  } as CSSProperties

  const contentStyle: CSSProperties = isOriginal ? {} : {
    fontFamily: readerFontFamily(settings.font_family),
    fontSize: `${settings.font_size}px`,
    lineHeight: settings.line_height,
  }

  const paragraphStyle: CSSProperties = isOriginal ? {} : {
    marginBottom: `${settings.paragraph_spacing}em`,
  }

  return {
    activePanel,
    book,
    chapterTitle,
    closeToc,
    contentRef,
    contentStyle,
    currentBookmark: bookmarks.currentBookmark,
    currentChapter,
    currentChapterIndex,
    flatToc,
    goToChapter: navigation.goToChapter,
    handleCloseSearch: search.handleClose,
    handleScroll: progress.handleScroll,
    handleSearchInput: search.handleInput,
    handleSearchResultClick: navigation.handleSearchResultClick,
    handleToggleSearch: search.handleToggle,
    handleTtsStop: ttsReader.handleTtsStop,
    handleTtsToggle: ttsReader.handleTtsToggle,
    imageCache: images.imageCache,
    paragraphStyle,
    progressDisplay,
    progressPercent,
    readerStyle,
    searchQuery: search.searchQuery,
    searchResults: search.searchResults,
    setActivePanel,
    settings,
    showSearch,
    showToc,
    toggleBookmark: bookmarks.toggleBookmark,
    toggleToc,
    tts,
    updateAndSave,
    goBackToLibrary: navigation.goBackToLibrary,
    goToPreviousChapter: navigation.goToPreviousChapter,
    goToNextChapter: navigation.goToNextChapter,
  }
}
