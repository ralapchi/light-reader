import { useCallback, type CSSProperties, type RefObject } from 'react'
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

const breakBeforeColumn = { breakBefore: 'column' } as CSSProperties
const columnFillAuto = { columnFill: 'auto' } as CSSProperties

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
  paragraphStyle: CSSProperties
}

export default function TwoPageReaderContent({
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
  paragraphStyle,
}: TwoPageReaderContentProps) {
  // ── Hooks ──────────────────────────────────────────────────

  const { flowChapters, loadNextChapter, hasNextChapter, setExtraChapters } =
    useAdjacentChapterPreload(chapter, chapterCount)

  const { scrollRef, chapterRefs, pageHeight, pageWidth, spineGap, chapterGaps, totalSpreads, totalSpreadsRef } =
    useTwoPageLayout(contentRef, contentStyle, flowChapters)

  const { nextSpread, prevSpread } = useTwoPageNavigation(
    contentRef, scrollRef, totalSpreadsRef, pageWidth, spineGap, totalSpreads,
    flowChapters, chapter, hasNextChapter, loadNextChapter,
    setExtraChapters, twoPageNavRef,
    onNextChapter, onPreviousChapter, onNavigate, initialParagraphIndex,
  )

  // ── Render ─────────────────────────────────────────────────

  const handleSpreadClick = useCallback((e: React.MouseEvent) => {
    if ((e.target as HTMLElement).closest('.reader-link')) return
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
    const x = e.clientX - rect.left
    if (x < rect.width * 0.2) prevSpread()
    else if (x > rect.width * 0.8) nextSpread()
  }, [prevSpread, nextSpread])

  return (
    <div className="reader-content two-page" ref={contentRef} onClick={handleSpreadClick}>
      <div
        ref={scrollRef}
        style={{ flex: 1, alignSelf: 'stretch', overflow: 'hidden' }}
      >
        <div
          className="reader-page-text"
          style={{
            ...contentStyle,
            columnWidth: `${pageWidth}px`,
            columnGap: `${spineGap}px`,
            ...columnFillAuto,
            height: pageHeight,
            overflow: 'visible',
          }}
        >
          {flowChapters.map((ch, ci) => {
            const needSpacer = ci > 0 && chapterGaps[ci - 1] != null && chapterGaps[ci - 1] > pageHeight
            return (
            <div
              key={ch.chapter_index}
              ref={(el) => { chapterRefs.current[ci] = el }}
              style={ci > 0 ? breakBeforeColumn : undefined}
            >
              {needSpacer && <div style={{ height: pageHeight }} />}
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
            )
          })}
        </div>
      </div>

    </div>
  )
}
