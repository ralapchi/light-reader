import { useCallback, type CSSProperties, type RefObject } from 'react'
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
  onScroll: () => void
  onLinkClick?: (href: string) => void
  onNavigate?: () => void
  paragraphStyle: CSSProperties
}

export default function SinglePageReaderContent({
  chapter,
  contentRef,
  contentStyle,
  contentWidth,
  highlightedParagraphIndex,
  imageCache,
  onScroll,
  onLinkClick,
  onNavigate,
  paragraphStyle,
}: SinglePageReaderContentProps) {
  const handleScroll = useCallback(() => {
    onScroll()
    onNavigate?.()
  }, [onScroll, onNavigate])

  return (
    <div className="reader-content" ref={contentRef} onScroll={handleScroll}>
      <div className="book" style={{ maxWidth: `${contentWidth}px` }}>
        <div className="pg pg-l">
          <div className="tx" style={contentStyle}>
            {chapter && <h1 className="chapter-title">{chapter.title}</h1>}
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
}
