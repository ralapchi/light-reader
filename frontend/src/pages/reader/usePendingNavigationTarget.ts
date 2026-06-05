import { useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import type { ReaderBookDto, ReadingMode } from '../../services/api'
import useAppStore from '../../store/useAppStore'
import type { TwoPageNav } from './TwoPageReaderContent'
import { afterNextPaint, afterLayoutSettled } from './rafUtils'
import { scrollToAnchor, scrollToOffset, scrollToParagraph, scrollToParagraphTwoPage } from './readerUtils'

export function usePendingNavigationTarget(
  bookId: string | undefined,
  book: ReaderBookDto | null,
  contentRef: React.RefObject<HTMLDivElement | null>,
  readingMode: ReadingMode | undefined,
  goToChapter: (index: number, scrollOffset?: number | null, options?: { saveProgress?: boolean }) => Promise<void>,
  twoPageNavRef?: React.RefObject<TwoPageNav | null>,
) {
  const navigate = useNavigate()
  const currentChapter = useAppStore(s => s.reader.currentChapter)
  const currentChapterIndex = useAppStore(s => s.reader.currentChapterIndex)

  useEffect(() => {
    const cancels: (() => void)[] = []
    const timer = window.setTimeout(() => {
      if (!book) {
        navigate(`/loading/${bookId}`)
        return
      }
      const pending = useAppStore.getState().reader.pendingNavTarget
      if (pending) {
        const targetChapter = Math.min(pending.chapter_index, (book.chapter_count ?? 1) - 1)
        useAppStore.getState().setPendingNavTarget(null)
        const alreadyLoaded = currentChapter && currentChapter.chapter_index === targetChapter
        if (alreadyLoaded) {
          if (readingMode === 'TwoPage') {
            if (pending.paragraph_index != null) {
              cancels.push(afterNextPaint(() => {
                const content = contentRef.current
                if (!content) return
                scrollToParagraphTwoPage(content, pending.paragraph_index!, twoPageNavRef?.current)
              }))
            }
          } else if (pending.anchor) {
            cancels.push(afterNextPaint(() => {
              const el = contentRef.current
              if (!el) return
              scrollToAnchor(el, pending.anchor!)
            }))
          } else if (pending.scroll_offset && pending.scroll_offset > 0) {
            cancels.push(afterLayoutSettled(() => {
              const el = contentRef.current
              if (!el) return
              scrollToOffset(el, pending.scroll_offset!)
            }))
          } else if (pending.paragraph_index != null) {
            cancels.push(afterNextPaint(() => {
              const content = contentRef.current
              if (!content) return
              scrollToParagraph(content, pending.paragraph_index!)
            }))
          }
        } else {
          goToChapter(targetChapter, pending.scroll_offset, { saveProgress: false }).then(() => {
            if (pending.anchor && readingMode !== 'TwoPage') {
              cancels.push(afterNextPaint(() => {
                const content = contentRef.current
                if (!content) return
                scrollToAnchor(content, pending.anchor!)
              }))
            } else if (pending.paragraph_index != null && (!pending.scroll_offset || pending.scroll_offset <= 0)) {
              cancels.push(afterNextPaint(() => {
                const content = contentRef.current
                if (!content) return
                if (readingMode === 'TwoPage') scrollToParagraphTwoPage(content, pending.paragraph_index!, twoPageNavRef?.current)
                else scrollToParagraph(content, pending.paragraph_index!)
              }))
            }
          })
        }
      } else if (!currentChapter) {
        goToChapter(currentChapterIndex)
      }
    }, 0)
    return () => {
      window.clearTimeout(timer)
      cancels.forEach(fn => fn())
    }
  }, [book, bookId, currentChapter, currentChapterIndex, goToChapter, navigate, contentRef, readingMode])
}
