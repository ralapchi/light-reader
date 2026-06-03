import { useCallback } from 'react'
import { readerResolveHref, type ReadingMode } from '../../services/api'
import useAppStore from '../../store/useAppStore'
import type { TwoPageNav } from './TwoPageReaderContent'
import { captureVisibleParagraph, scrollToParagraph, scrollToParagraphTwoPage } from './readerUtils'
import type { FootnoteReturnEntry } from './useFootnoteReturn'

interface NavigateToHrefOptions {
  showReturn?: boolean
}

export function useHrefNavigation(
  contentRef: React.RefObject<HTMLDivElement | null>,
  readingMode: ReadingMode | undefined,
  goToChapter: (index: number, scrollOffset?: number | null, options?: { saveProgress?: boolean }) => Promise<void>,
  setFootnoteReturn: (entry: FootnoteReturnEntry) => void,
  twoPageNavRef?: React.RefObject<TwoPageNav | null>,
) {
  const currentChapterIndex = useAppStore(s => s.reader.currentChapterIndex)

  const navigateToHref = useCallback(async (
    href: string,
    fallbackIndex?: number | null,
    options?: NavigateToHrefOptions,
  ) => {
    const currentEl = contentRef.current
    const twoPage = readingMode === 'TwoPage'
    const shouldShowReturn = options?.showReturn !== false
    const savedReturn: FootnoteReturnEntry = {
      chapterIndex: currentChapterIndex,
      paragraphIndex: currentEl ? captureVisibleParagraph(currentEl, twoPage) : null,
      scrollOffset: currentEl ? (currentEl.scrollTop || 0) : 0,
    }

    try {
      const resolved = await readerResolveHref(href, currentChapterIndex)
      if (!resolved) {
        if (fallbackIndex != null) goToChapter(fallbackIndex)
        return
      }

      const targetChapter = resolved.chapter_index
      if (targetChapter !== currentChapterIndex) {
        if (shouldShowReturn) setFootnoteReturn(savedReturn)
        if (resolved.paragraph_index != null) {
          useAppStore.getState().setPendingNavTarget({
            chapter_index: targetChapter,
            paragraph_index: resolved.paragraph_index,
            scroll_offset: null,
          })
        }
        goToChapter(targetChapter, null, { saveProgress: false }).then(() => {
          if (resolved.paragraph_index != null) {
            requestAnimationFrame(() => {
              const content = contentRef.current
              if (!content) return
              if (readingMode === 'TwoPage') scrollToParagraphTwoPage(content, resolved.paragraph_index!, twoPageNavRef?.current)
              else scrollToParagraph(content, resolved.paragraph_index!)
            })
          }
        })
      } else {
        if (shouldShowReturn) setFootnoteReturn(savedReturn)
        const targetPara = resolved.paragraph_index
        if (targetPara != null) {
          requestAnimationFrame(() => {
            const content = contentRef.current
            if (!content) return
            if (readingMode === 'TwoPage') scrollToParagraphTwoPage(content, targetPara, twoPageNavRef?.current)
            else scrollToParagraph(content, targetPara)
          })
        }
      }
    } catch {
      if (fallbackIndex != null) goToChapter(fallbackIndex)
    }
  }, [currentChapterIndex, goToChapter, contentRef, readingMode, setFootnoteReturn])

  return { navigateToHref }
}
