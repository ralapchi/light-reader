import { useEffect, useMemo } from 'react'
import type { ReaderChapterDto } from '../../services/api'
import type { TwoPageVisibleChapter } from './TwoPageReaderContent'

/**
 * Computes the visible chapter index/title from the current flow index
 * and notifies the parent via onVisibleChapterChange.
 */
export function useVisibleChapterSync(
  flowChapters: ReaderChapterDto[],
  chapter: ReaderChapterDto | null,
  currentChapterFlowIndex: number,
  activeSpreadIndex: number,
  getChapterOffsetForSpread: (spread: number, flowIdx: number) => number,
  onVisibleChapterChange: ((visible: TwoPageVisibleChapter | null) => void) | undefined,
) {
  const visibleChapterIndex = useMemo(() => {
    if (flowChapters.length === 0) return chapter?.chapter_index ?? 0
    const idx = currentChapterFlowIndex
    return flowChapters[idx]?.chapter_index ?? chapter?.chapter_index ?? 0
  }, [flowChapters, chapter, currentChapterFlowIndex])

  const visibleChapterTitle = useMemo(() => {
    const idx = currentChapterFlowIndex
    return flowChapters[idx]?.title ?? chapter?.title ?? ''
  }, [flowChapters, chapter, currentChapterFlowIndex])

  const visibleChapterOffset = useMemo(
    () => getChapterOffsetForSpread(activeSpreadIndex, currentChapterFlowIndex),
    [activeSpreadIndex, currentChapterFlowIndex, getChapterOffsetForSpread],
  )

  useEffect(() => {
    if (flowChapters.length === 0) {
      onVisibleChapterChange?.(null)
      return
    }
    onVisibleChapterChange?.({
      chapterIndex: visibleChapterIndex,
      title: visibleChapterTitle,
    })
  }, [flowChapters.length, onVisibleChapterChange, visibleChapterIndex, visibleChapterTitle])

  return { visibleChapterIndex, visibleChapterTitle, visibleChapterOffset }
}
