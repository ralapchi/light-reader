import { useCallback, useState } from 'react'
import type { ReadingMode } from '../../services/api'
import useAppStore from '../../store/useAppStore'
import type { TwoPageNav } from './TwoPageReaderContent'
import { scrollToOffset, scrollToParagraphTwoPage } from './readerUtils'

export interface FootnoteReturnEntry {
  chapterIndex: number
  paragraphIndex: number | null
  scrollOffset: number
}

export function useFootnoteReturn(
  contentRef: React.RefObject<HTMLDivElement | null>,
  readingMode: ReadingMode | undefined,
  goToChapter: (index: number, scrollOffset?: number | null, options?: { saveProgress?: boolean }) => Promise<void>,
  twoPageNavRef?: React.RefObject<TwoPageNav | null>,
) {
  const currentChapterIndex = useAppStore(s => s.reader.currentChapterIndex)
  const [footnoteReturn, setFootnoteReturn] = useState<FootnoteReturnEntry | null>(null)

  const returnFromFootnote = useCallback(() => {
    if (!footnoteReturn) return
    const { chapterIndex, paragraphIndex, scrollOffset } = footnoteReturn
    setFootnoteReturn(null)
    if (chapterIndex !== currentChapterIndex) {
      useAppStore.getState().setPendingNavTarget({
        chapter_index: chapterIndex,
        paragraph_index: readingMode === 'TwoPage' ? paragraphIndex : null,
        scroll_offset: readingMode === 'TwoPage' ? null : scrollOffset,
      })
      goToChapter(chapterIndex, null, { saveProgress: false })
    } else {
      const el = contentRef.current
      if (!el) return
      if (readingMode === 'TwoPage') {
        if (paragraphIndex != null) scrollToParagraphTwoPage(el, paragraphIndex, twoPageNavRef?.current)
      } else {
        scrollToOffset(el, scrollOffset)
      }
    }
  }, [footnoteReturn, currentChapterIndex, goToChapter, contentRef, readingMode])

  const clearFootnoteReturn = useCallback(() => setFootnoteReturn(null), [])

  return { footnoteReturn, setFootnoteReturn, returnFromFootnote, clearFootnoteReturn }
}
