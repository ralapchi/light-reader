import { memo, useCallback, type CSSProperties, type RefObject } from 'react'
import type { ReaderChapterDto } from '../../services/api'
import ReaderBlock from './ReaderBlock'
import { blockKey, blockParagraphIndex } from './readerUtils'

interface SinglePageReaderContentProps {
  chapter: ReaderChapterDto | null
  contentRef: RefObject<HTMLDivElement | null>
  contentStyle: CSSProperties
  contentWidth: number
  highlightedParagraphIndex?: number
  imageCache: Record<string, string>
  onNextChapter?: () => void
  onPreviousChapter?: () => void
  onScroll: () => void
  onLinkClick?: (href: string) => void
  onNavigate?: () => void
  paragraphStyle: CSSProperties
}

export default memo(function SinglePageReaderContent({
  chapter,
  contentRef,
  contentStyle,
  contentWidth,
  highlightedParagraphIndex,
  imageCache,
  onNextChapter,
  onPreviousChapter,
  onScroll,
  onLinkClick,
  onNavigate,
  paragraphStyle,
}: SinglePageReaderContentProps) {
  const handleScroll = useCallback(() => {
    onScroll()
    onNavigate?.()
  }, [onScroll, onNavigate])

  const handleClick = useCallback((e: React.MouseEvent) => {
    if ((e.target as HTMLElement).closest('.reader-link')) return
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
    const x = e.clientX - rect.left
    if (x < rect.width * 0.2) onPreviousChapter?.()
    else if (x > rect.width * 0.8) onNextChapter?.()
  }, [onNextChapter, onPreviousChapter])

  return (
    <div className="reader-content" ref={contentRef} onScroll={handleScroll} onClick={handleClick}>
      <div className="reader-book" style={{ maxWidth: `${contentWidth}px` }}>
        <div className="reader-page reader-page-l">
          <div className="reader-page-text" style={contentStyle}>
            {(chapter?.blocks ?? []).map((block, i) => (
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
      </div>
    </div>
  )
})
