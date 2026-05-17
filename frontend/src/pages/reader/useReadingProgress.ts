import { useCallback, useEffect, useRef } from 'react'
import { readerSaveProgress } from '../../services/api'
import type { ReaderBookDto } from '../../services/api'
import useAppStore from '../../store/useAppStore'
import { findVisibleParagraphIndex } from './readerUtils'

export function useReadingProgress(
  bookId: string | undefined,
  book: ReaderBookDto | null,
  currentChapterIndex: number,
  contentRef: React.RefObject<HTMLDivElement | null>,
) {
  const { setProgressPercent } = useAppStore()
  const progressPercent = useAppStore(s => s.reader.progressPercent)

  const saveProgress = useCallback((pct?: number, _force?: boolean, paragraphIndex?: number | null, scrollOffset?: number) => {
    if (!bookId) return
    readerSaveProgress({
      book_id: bookId,
      chapter_index: currentChapterIndex,
      progress_percent: pct ?? progressPercent,
      paragraph_index: paragraphIndex ?? null,
      scroll_offset: scrollOffset ?? null,
    }).catch(() => { /* non-critical */ })
  }, [bookId, currentChapterIndex, progressPercent])

  const saveCurrentPosition = useCallback(() => {
    const el = contentRef.current
    if (!el || !book) return
    const scrollTop = el.scrollTop
    const scrollHeight = el.scrollHeight - el.clientHeight
    if (scrollHeight <= 0) return
    const chapterPct = scrollTop / scrollHeight
    const bookPct = Math.min(1, (currentChapterIndex + chapterPct) / book.chapter_count)
    const paraIndex = findVisibleParagraphIndex(el)
    saveProgress(bookPct, true, paraIndex, scrollTop)
  }, [book, currentChapterIndex, saveProgress, contentRef])

  const saveRef = useRef(saveCurrentPosition)
  useEffect(() => { saveRef.current = saveCurrentPosition }, [saveCurrentPosition])

  useEffect(() => {
    const onVisibility = () => { if (document.hidden) saveRef.current() }
    const onBeforeUnload = () => saveRef.current()
    document.addEventListener('visibilitychange', onVisibility)
    window.addEventListener('beforeunload', onBeforeUnload)
    return () => {
      document.removeEventListener('visibilitychange', onVisibility)
      window.removeEventListener('beforeunload', onBeforeUnload)
      saveRef.current()
    }
  }, [])

  const handleScroll = useCallback(() => {
    const el = contentRef.current
    if (!el) return
    const scrollTop = el.scrollTop
    const scrollHeight = el.scrollHeight - el.clientHeight
    if (scrollHeight > 0 && book) {
      const chapterPct = scrollTop / scrollHeight
      const bookPct = Math.min(1, (currentChapterIndex + chapterPct) / book.chapter_count)
      setProgressPercent(bookPct)
    }
  }, [setProgressPercent, book, currentChapterIndex, contentRef])

  return { saveProgress, saveCurrentPosition, handleScroll }
}
