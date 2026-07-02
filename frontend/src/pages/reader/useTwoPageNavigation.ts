import type { RefObject } from 'react'
import type { ReaderChapterDto } from '../../services/api'
import type { TwoPageNav, TwoPageVisibleChapter } from './TwoPageReaderContent'
import { useSpreadNavigation } from './useSpreadNavigation'
import { useSpreadKeyboard } from './useSpreadKeyboard'
import { useSpreadPreload } from './useSpreadPreload'
import { useVisibleChapterSync } from './useVisibleChapterSync'

/**
 * Two-page navigation hook — composes spread management, keyboard handling,
 * chapter preload, and visible chapter sync.
 */
export function useTwoPageNavigation(
  contentRef: RefObject<HTMLDivElement | null>,
  scrollRef: RefObject<HTMLDivElement | null>,
  totalSpreadsRef: React.MutableRefObject<number>,
  pageWidth: number,
  spineGap: number,
  totalSpreads: number,
  flowChapters: ReaderChapterDto[],
  chapter: ReaderChapterDto | null,
  chapterSpreadStarts: number[],
  chapterContentPageCounts: number[],
  hasNextChapter: boolean,
  loadNextChapter: () => Promise<boolean>,
  setExtraChapters: (chapters: ReaderChapterDto[]) => void,
  twoPageNavRef: React.MutableRefObject<TwoPageNav | null>,
  onNextChapter: (() => void) | undefined,
  onPreviousChapter: (() => void) | undefined,
  onNavigate: (() => void) | undefined,
  initialParagraphIndex: number | null | undefined,
  onVisibleChapterChange: ((visible: TwoPageVisibleChapter | null) => void) | undefined,
) {
  const {
    spreadIndex,
    activeSpreadIndex,
    innerRef,
    goToSpread,
    turnSpread,
    nextSpread,
    prevSpread,
    findSpreadByParagraph,
    recalcSpreads,
    currentChapterFlowIndex,
    getChapterOffsetForSpread,
  } = useSpreadNavigation(
    scrollRef, totalSpreadsRef,
    pageWidth, spineGap, totalSpreads,
    flowChapters, chapter, chapterSpreadStarts, chapterContentPageCounts,
    hasNextChapter, loadNextChapter, setExtraChapters,
    twoPageNavRef, onNextChapter, onPreviousChapter, onNavigate,
    initialParagraphIndex,
  )

  useSpreadKeyboard(contentRef, nextSpread, prevSpread)

  useSpreadPreload(spreadIndex, totalSpreads, hasNextChapter, loadNextChapter)

  const { visibleChapterIndex } = useVisibleChapterSync(
    flowChapters, chapter, currentChapterFlowIndex,
    activeSpreadIndex, getChapterOffsetForSpread,
    onVisibleChapterChange,
  )

  return {
    spreadIndex,
    activeSpreadIndex,
    innerRef,
    goToSpread,
    turnSpread,
    nextSpread,
    prevSpread,
    findSpreadByParagraph,
    recalcSpreads,
    visibleChapterIndex,
  }
}
