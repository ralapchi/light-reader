import { useCallback, useEffect, useRef } from 'react'
import { readerSaveProgress } from '../../services/api'
import type { ReaderBookDto, ReadingMode } from '../../services/api'
import useAppStore from '../../store/useAppStore'
import type { TwoPageNav } from './TwoPageReaderContent'
import { chapterProgressPercent } from './readerProgressUtils'

export function useReadingProgress(
  bookId: string | undefined,
  book: ReaderBookDto | null,
  currentChapterIndex: number,
  contentRef: React.RefObject<HTMLDivElement | null>,
  readingMode?: ReadingMode,
  twoPageNavRef?: React.RefObject<TwoPageNav | null>,
) {
  const { setProgressPercent } = useAppStore()
  const progressPercent = useAppStore(s => s.reader.progressPercent)

  const saveProgress = useCallback((pct?: number, _force?: boolean, _paragraphIndex?: number | null, _scrollOffset?: number | null, chapterIndex?: number) => {
    if (!bookId) return
    readerSaveProgress({
      book_id: bookId,
      chapter_index: chapterIndex ?? currentChapterIndex,
      progress_percent: pct ?? progressPercent,
      paragraph_index: null,
      scroll_offset: null,
      anchor: null,
      clear_position: true,
    }).catch(() => { /* non-critical */ })
  }, [bookId, currentChapterIndex, progressPercent])

  const saveCurrentPosition = useCallback(() => {
    const el = contentRef.current
    if (!el || !book) return
    const twoPage = readingMode === 'TwoPage'
    let bookPct: number

    if (twoPage) {
      const nav = twoPageNavRef?.current ?? null
      const visibleChapterIndex = nav?.currentChapterIndex ?? currentChapterIndex
      bookPct = chapterProgressPercent(visibleChapterIndex, book.chapter_count)
      saveProgress(bookPct, true, null, null, visibleChapterIndex)
      return
    } else {
      bookPct = chapterProgressPercent(currentChapterIndex, book.chapter_count)
    }

    saveProgress(bookPct, true, null, null, currentChapterIndex)
  }, [book, currentChapterIndex, saveProgress, contentRef, readingMode])

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
      const bookPct = Math.min(1, (currentChapterIndex + chapterPct) / book.chapter_count)
      setProgressPercent(bookPct)
    }
  }, [setProgressPercent, book, currentChapterIndex, contentRef, readingMode])

  return { saveProgress, saveCurrentPosition, handleScroll }
}
