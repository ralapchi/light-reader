import { useCallback, useEffect, useRef } from 'react'
import { readerSaveProgress } from '../../services/api'
import type { ReaderBookDto, ReadingMode } from '../../services/api'
import useAppStore from '../../store/useAppStore'
import type { TwoPageNav } from './TwoPageReaderContent'
import {
  createReadingPosition,
  readingPositionProgressPercent,
  readingPositionToSaveProgress,
  type ReadingPosition,
} from './readerProgressUtils'

export function useReadingProgress(
  bookId: string | undefined,
  book: ReaderBookDto | null,
  currentChapterIndex: number,
  contentRef: React.RefObject<HTMLDivElement | null>,
  readingMode?: ReadingMode,
  twoPageNavRef?: React.RefObject<TwoPageNav | null>,
) {
  const setProgressPercent = useAppStore(s => s.setProgressPercent)
  const scrollSaveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const readingModeRef = useRef(readingMode)
  useEffect(() => { readingModeRef.current = readingMode }, [readingMode])

  useEffect(() => {
    if (readingMode !== 'TwoPage') return
    if (scrollSaveTimerRef.current != null) {
      clearTimeout(scrollSaveTimerRef.current)
      scrollSaveTimerRef.current = null
    }
  }, [readingMode])

  const savePosition = useCallback((position: ReadingPosition) => {
    if (!book) return
    const progress = readingPositionToSaveProgress(position, book.chapter_count)
    readerSaveProgress(progress).catch(() => { /* non-critical */ })
  }, [book])

  const saveCurrentPosition = useCallback(() => {
    const el = contentRef.current
    if (!el || !book) return
    const twoPage = readingMode === 'TwoPage'
    if (twoPage) {
      const nav = twoPageNavRef?.current ?? null
      const visibleChapterIndex = nav?.currentChapterIndex ?? currentChapterIndex
      const chapterOffset = nav?.currentChapterOffset ?? 0
      const position = createReadingPosition(bookId ?? book.book_id, visibleChapterIndex, chapterOffset, 'two-page')
      savePosition(position)
      return
    } else {
      const maxScroll = Math.max(0, el.scrollHeight - el.clientHeight)
      const chapterOffset = maxScroll > 0 ? el.scrollTop / maxScroll : 0
      const position = createReadingPosition(bookId ?? book.book_id, currentChapterIndex, chapterOffset, 'single')
      savePosition(position)
      return
    }
  }, [book, bookId, currentChapterIndex, savePosition, contentRef, readingMode, twoPageNavRef])

  const saveRef = useRef(saveCurrentPosition)
  useEffect(() => { saveRef.current = saveCurrentPosition }, [saveCurrentPosition])

  useEffect(() => {
    const onVisibility = () => {
      if (document.hidden) {
        saveRef.current()
      }
    }
    const onBeforeUnload = () => {
      saveRef.current()
    }
    document.addEventListener('visibilitychange', onVisibility)
    window.addEventListener('beforeunload', onBeforeUnload)
    return () => {
      document.removeEventListener('visibilitychange', onVisibility)
      window.removeEventListener('beforeunload', onBeforeUnload)
      if (scrollSaveTimerRef.current != null) {
        clearTimeout(scrollSaveTimerRef.current)
        scrollSaveTimerRef.current = null
      }
      saveRef.current()
    }
  }, [])

  const handleScroll = useCallback(() => {
    if (readingMode === 'TwoPage') return
    const el = contentRef.current
    if (!el) return
    const scrollTop = el.scrollTop
    const scrollHeight = el.scrollHeight - el.clientHeight
    if (scrollHeight > 0 && book) {
      const chapterPct = scrollTop / scrollHeight
      const position = createReadingPosition(book.book_id, currentChapterIndex, chapterPct, 'single')
      const bookPct = readingPositionProgressPercent(position, book.chapter_count)
      setProgressPercent(bookPct)
      if (scrollSaveTimerRef.current != null) clearTimeout(scrollSaveTimerRef.current)
      scrollSaveTimerRef.current = setTimeout(() => {
        scrollSaveTimerRef.current = null
        if (readingModeRef.current === 'TwoPage') return
        savePosition(position)
      }, 250)
    }
  }, [setProgressPercent, book, currentChapterIndex, contentRef, readingMode, savePosition])

  return { saveCurrentPosition, handleScroll }
}
