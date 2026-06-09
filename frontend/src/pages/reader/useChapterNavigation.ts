import { useCallback, useEffect, useRef } from 'react'
import { flushSync } from 'react-dom'
import { useNavigate } from 'react-router-dom'
import { libraryFlushIndex, readerFlushProgress, readerGetChapter, readerSaveProgress } from '../../services/api'
import type { ReaderBookDto, ReadingMode, SearchHitDto } from '../../services/api'
import useAppStore from '../../store/useAppStore'
import type { TwoPageNav } from './TwoPageReaderContent'
import { buildChapterOnlyProgress } from './readerProgressUtils'
import { afterNextPaint } from './rafUtils'
import { scrollToOffset, scrollToParagraph, scrollToParagraphTwoPage } from './readerUtils'
import { useChapterImages } from './useChapterImages'
import { useFootnoteReturn } from './useFootnoteReturn'
import { useHrefNavigation } from './useHrefNavigation'
import { usePendingNavigationTarget } from './usePendingNavigationTarget'

export function useChapterNavigation(
  bookId: string | undefined,
  book: ReaderBookDto | null,
  contentRef: React.RefObject<HTMLDivElement | null>,
  handleCloseSearch: () => void,
  readingMode?: ReadingMode,
  twoPageNavRef?: React.RefObject<TwoPageNav | null>,
) {
  const navigate = useNavigate()
  const {
    setCurrentChapter,
    setProgressPercent,
    closeToc,
  } = useAppStore()
  const currentChapterIndex = useAppStore(s => s.reader.currentChapterIndex)
  const rafCancelsRef = useRef<(() => void)[]>([])
  useEffect(() => () => { rafCancelsRef.current.forEach(fn => fn()); rafCancelsRef.current = [] }, [])
  const getNavigationChapterIndex = useCallback(() => {
    if (readingMode === 'TwoPage') {
      return twoPageNavRef?.current?.currentChapterIndex ?? currentChapterIndex
    }
    return currentChapterIndex
  }, [currentChapterIndex, readingMode, twoPageNavRef])

  // ── Chapter image loading ──────────────────────────────
  const { imageCache, loadChapterImages } = useChapterImages(bookId)

  // ── Chapter navigation ─────────────────────────────────
  const goToChapter = useCallback(async (
    index: number,
    scrollOffset?: number | null,
    options?: { saveProgress?: boolean },
  ) => {
    try {
      const chapter = await readerGetChapter(index)
      flushSync(() => {
        setCurrentChapter(index, chapter)
      })
      closeToc()
      handleCloseSearch()
      loadChapterImages(chapter.blocks)
      const bookPct = book ? Math.min(1, index / book.chapter_count) : 0
      setProgressPercent(bookPct)
      if (bookId && options?.saveProgress !== false) {
        readerSaveProgress(
          buildChapterOnlyProgress(bookId, index, book?.chapter_count ?? 0),
        ).catch(() => { /* non-critical */ })
      }
      rafCancelsRef.current.push(afterNextPaint(() => {
        const el = contentRef.current
        if (!el) return
        if (readingMode === 'TwoPage' && scrollOffset == null) {
          // In two-page mode, no scroll offset - start at first spread
        } else if (scrollOffset && scrollOffset > 0) {
          scrollToOffset(el, scrollOffset)
        } else {
          scrollToOffset(el, 0)
        }
      }))
    } catch (e) {
      console.error('加载章节失败:', e)
    }
  }, [setCurrentChapter, closeToc, handleCloseSearch, loadChapterImages, bookId, book, setProgressPercent, contentRef, readingMode])

  // ── Search result click ────────────────────────────────
  const handleSearchResultClick = useCallback((hit: SearchHitDto) => {
    handleCloseSearch()
    goToChapter(hit.chapter_index, null, { saveProgress: false }).then(() => {
      if (hit.paragraph_index != null) {
        rafCancelsRef.current.push(afterNextPaint(() => {
          const content = contentRef.current
          if (!content) return
          if (readingMode === 'TwoPage') scrollToParagraphTwoPage(content, hit.paragraph_index, twoPageNavRef?.current)
          else scrollToParagraph(content, hit.paragraph_index)
        }))
      }
    })
  }, [handleCloseSearch, goToChapter, contentRef, readingMode])

  // ── Footnote return stack ──────────────────────────────
  const { footnoteReturn, setFootnoteReturn, returnFromFootnote, clearFootnoteReturn } =
    useFootnoteReturn(contentRef, readingMode, goToChapter, twoPageNavRef)

  // ── Href navigation ────────────────────────────────────
  const { navigateToHref } = useHrefNavigation(contentRef, readingMode, goToChapter, setFootnoteReturn, twoPageNavRef)

  // ── Pending navigation target ──────────────────────────
  usePendingNavigationTarget(bookId, book, contentRef, readingMode, goToChapter, twoPageNavRef)

  const goBackToLibrary = useCallback(() => {
    Promise.allSettled([libraryFlushIndex(), readerFlushProgress()])
      .finally(() => navigate('/'))
  }, [navigate])
  const goToPreviousChapter = useCallback(() => {
    const index = getNavigationChapterIndex()
    if (index > 0) goToChapter(index - 1)
  }, [getNavigationChapterIndex, goToChapter])
  const goToNextChapter = useCallback(() => {
    const index = getNavigationChapterIndex()
    if (book && index < book.chapter_count - 1) goToChapter(index + 1)
  }, [book, getNavigationChapterIndex, goToChapter])

  return {
    goToChapter,
    navigateToHref,
    returnFromFootnote,
    footnoteReturn,
    clearFootnoteReturn,
    handleSearchResultClick,
    imageCache,
    goBackToLibrary,
    goToPreviousChapter,
    goToNextChapter,
  }
}
