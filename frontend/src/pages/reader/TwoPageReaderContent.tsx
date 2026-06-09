import { memo, useCallback, type CSSProperties, type RefObject } from 'react'
import type { ReaderChapterDto } from '../../services/api'
import ReaderBlock from './ReaderBlock'
import { blockKey, blockParagraphIndex } from './readerUtils'
import { useAdjacentChapterPreload } from './useAdjacentChapterPreload'
import { useTwoPageLayout } from './useTwoPageLayout'
import { useTwoPageNavigation } from './useTwoPageNavigation'

export interface TwoPageNav {
  findSpreadByParagraph: (paragraphIndex: number) => number | null
  goToSpread: (index: number) => void
  recalcSpreads: () => void
  spreadIndex: number
  spreadCount: number
  currentChapterIndex: number
  innerRef: React.RefObject<HTMLDivElement | null>
}

export interface TwoPageVisibleChapter {
  chapterIndex: number
  title: string
}

const columnFillAuto = { columnFill: 'auto' } as CSSProperties
const chapterGroupStyle = { height: '100%', overflow: 'hidden', flexShrink: 0 } as CSSProperties

interface TwoPageReaderContentProps {
  chapter: ReaderChapterDto | null
  chapterCount: number
  contentRef: RefObject<HTMLDivElement | null>
  contentStyle: CSSProperties
  highlightedParagraphIndex?: number
  imageCache: Record<string, string>
  initialParagraphIndex?: number | null
  twoPageNavRef: React.MutableRefObject<TwoPageNav | null>
  onNextChapter?: () => void
  onPreviousChapter?: () => void
  onLinkClick?: (href: string) => void
  onNavigate?: () => void
  onVisibleChapterChange?: (visible: TwoPageVisibleChapter | null) => void
  paragraphStyle: CSSProperties
}

export default memo(function TwoPageReaderContent({
  chapter,
  chapterCount,
  contentRef,
  contentStyle,
  highlightedParagraphIndex,
  imageCache,
  initialParagraphIndex,
  twoPageNavRef,
  onNextChapter,
  onPreviousChapter,
  onLinkClick,
  onNavigate,
  onVisibleChapterChange,
  paragraphStyle,
}: TwoPageReaderContentProps) {
  // ── Hooks ──────────────────────────────────────────────────

  const { flowChapters, loadNextChapter, hasNextChapter, setExtraChapters } =
    useAdjacentChapterPreload(chapter, chapterCount)

  const { scrollRef, chapterRefs, pageHeight, pageWidth, spineGap, chapterPageCounts, chapterContentPageCounts, chapterSpreadStarts, totalSpreads, totalSpreadsRef, isReady } =
    useTwoPageLayout(contentRef, contentStyle, flowChapters)

  const { nextSpread, prevSpread } = useTwoPageNavigation(
    contentRef, scrollRef, totalSpreadsRef, pageWidth, spineGap, totalSpreads,
    flowChapters, chapter, chapterSpreadStarts, chapterContentPageCounts, hasNextChapter, loadNextChapter,
    setExtraChapters, twoPageNavRef,
    onNextChapter, onPreviousChapter, onNavigate, initialParagraphIndex, onVisibleChapterChange,
  )
  const spreadViewportWidth = pageWidth * 2 + spineGap
  const spreadStep = (pageWidth + spineGap) * 2

  const handleSpreadClick = useCallback((e: React.MouseEvent) => {
    if ((e.target as HTMLElement).closest('.reader-link')) return
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
    const x = e.clientX - rect.left
    if (x < rect.width * 0.2) prevSpread()
    else if (x > rect.width * 0.8) nextSpread()
  }, [prevSpread, nextSpread])

  if (!isReady) return (
    <div className="reader-content two-page" ref={contentRef} onClick={handleSpreadClick}>
      <div ref={scrollRef} style={{ width: `${spreadViewportWidth}px`, maxWidth: '100%', alignSelf: 'stretch', overflow: 'hidden' }} />
    </div>
  )

  return (
    <div className="reader-content two-page" ref={contentRef} onClick={handleSpreadClick}>
      <div
        ref={scrollRef}
        style={{ width: `${spreadViewportWidth}px`, maxWidth: '100%', alignSelf: 'stretch', overflow: 'hidden' }}
      >
        <div
          className="reader-page-flow"
          style={{
            display: 'flex',
            alignItems: 'stretch',
            height: pageHeight,
            width: `${Math.max(1, totalSpreads) * spreadStep}px`,
          }}
        >
          {flowChapters.map((ch, ci) => {
            const pageCount = chapterPageCounts[ci] ?? 2
            const chapterWidth = Math.max(1, pageCount) * (pageWidth + spineGap)
            const chapterTextWidth = Math.max(1, pageCount) * pageWidth + Math.max(0, pageCount - 1) * spineGap
            return (
              <div
                key={ch.chapter_index}
                ref={(el) => { chapterRefs.current[ci] = el }}
                data-chapter-index={ch.chapter_index}
                style={{
                  ...chapterGroupStyle,
                  width: `${chapterWidth}px`,
                }}
              >
                <div
                  className="reader-page-text"
                  style={{
                    ...contentStyle,
                    columnWidth: `${pageWidth}px`,
                    columnGap: `${spineGap}px`,
                    ...columnFillAuto,
                    height: pageHeight,
                    width: `${chapterTextWidth}px`,
                    overflow: 'visible',
                  }}
                >
                {ch.blocks.map((block, i) => (
                  <div className="reader-block-shell" key={blockKey(block, i)}>
                    <ReaderBlock
                      block={block}
                      imageCache={imageCache}
                      paragraphStyle={paragraphStyle}
                      highlight={blockParagraphIndex(block) === highlightedParagraphIndex}
                      onLinkClick={onLinkClick}
                    />
                  </div>
                ))}
                </div>
              </div>
            )
          })}
        </div>
      </div>

    </div>
  )
})
