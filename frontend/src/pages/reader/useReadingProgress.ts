import { useCallback, useEffect, useRef } from 'react'
import { readerSaveProgress } from '../../services/api'
import type { ReaderBookDto, ReaderAnchor, ReadingMode } from '../../services/api'
import useAppStore from '../../store/useAppStore'
import { findVisibleParagraphIndex, captureReaderAnchor } from './readerUtils'
import type { TwoPageNav } from './TwoPageReaderContent'

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

  const saveProgress = useCallback((pct?: number, _force?: boolean, paragraphIndex?: number | null, scrollOffset?: number | null, chapterIndex?: number, anchor?: ReaderAnchor | null) => {
    if (!bookId) return
    readerSaveProgress({
      book_id: bookId,
      chapter_index: chapterIndex ?? currentChapterIndex,
      progress_percent: pct ?? progressPercent,
      paragraph_index: paragraphIndex ?? null,
      scroll_offset: scrollOffset ?? null,
      anchor: anchor ?? null,
    }).catch(() => { /* non-critical */ })
  }, [bookId, currentChapterIndex, progressPercent])

  const saveCurrentPosition = useCallback(() => {
    const el = contentRef.current
    if (!el || !book) return
    const twoPage = readingMode === 'TwoPage'
    let bookPct: number
    let paraIndex: number | null
    let scrollOffset: number | null

    if (twoPage) {
      const nav = twoPageNavRef?.current ?? null
      const spreadIdx = nav ? nav.spreadIndex : 0
      const totalSpreads = nav ? nav.spreadCount : 1
      const visibleChapterIndex = nav?.currentChapterIndex ?? currentChapterIndex
      const chapterPct = totalSpreads > 1 ? spreadIdx / (totalSpreads - 1) : 0
      bookPct = Math.min(1, (visibleChapterIndex + chapterPct) / book.chapter_count)
      console.log(`[saveCurrentPos] TwoPage visCh=${visibleChapterIndex} spread=${spreadIdx}/${totalSpreads} bookPct=${bookPct.toFixed(4)}`)
      saveProgress(bookPct, true, null, null, visibleChapterIndex)
      return
    } else {
      const scrollTop = el.scrollTop
      const scrollHeight = el.scrollHeight - el.clientHeight
      if (scrollHeight <= 0) return
      const chapterPct = scrollTop / scrollHeight
      bookPct = Math.min(1, (currentChapterIndex + chapterPct) / book.chapter_count)
      paraIndex = findVisibleParagraphIndex(el)
      scrollOffset = scrollTop
    }

    const anchor = captureReaderAnchor(el, currentChapterIndex)
    saveProgress(bookPct, true, paraIndex, scrollOffset, currentChapterIndex, anchor)
  }, [book, currentChapterIndex, saveProgress, contentRef, readingMode])

  const saveRef = useRef(saveCurrentPosition)
  useEffect(() => { saveRef.current = saveCurrentPosition }, [saveCurrentPosition])

  useEffect(() => {
    const onVisibility = () => {
      if (document.hidden) {
        console.log(`[visibilitychange] hidden → save`)
        saveRef.current()
      }
    }
    const onBeforeUnload = () => {
      console.log(`[beforeunload] → save`)
      saveRef.current()
    }
    document.addEventListener('visibilitychange', onVisibility)
    window.addEventListener('beforeunload', onBeforeUnload)
    return () => {
      console.log(`[cleanup] unmount → save`)
      document.removeEventListener('visibilitychange', onVisibility)
      window.removeEventListener('beforeunload', onBeforeUnload)
      saveRef.current()
    }
  }, [])

  const handleScroll = useCallback(() => {
    if (readingMode === 'TwoPage') return
    console.log(`[handleScroll] called in ${readingMode} mode — this should NOT happen in TwoPage`)
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
