import type { CSSProperties, RefObject } from 'react'
import type { ReaderChapterDto } from '../../services/api'
import ReaderBlock from './ReaderBlock'
import { blockKey, blockParagraphIndex } from './readerUtils'

interface ReaderContentProps {
  chapter: ReaderChapterDto | null
  contentRef: RefObject<HTMLDivElement | null>
  contentStyle: CSSProperties
  highlightedParagraphIndex?: number
  imageCache: Record<string, string>
  onScroll: () => void
  paragraphStyle: CSSProperties
}

export default function ReaderContent({
  chapter,
  contentRef,
  contentStyle,
  highlightedParagraphIndex,
  imageCache,
  onScroll,
  paragraphStyle,
}: ReaderContentProps) {
  return (
    <div className="reader-content" ref={contentRef} onScroll={onScroll}>
      <div className="reader-column" style={contentStyle}>
        {chapter && (
          <>
            <h1 className="chapter-title">{chapter.title}</h1>
            {chapter.blocks.map((block, i) => (
              <ReaderBlock
                key={blockKey(block, i)}
                block={block}
                imageCache={imageCache}
                paragraphStyle={paragraphStyle}
                highlight={blockParagraphIndex(block) === highlightedParagraphIndex}
              />
            ))}
          </>
        )}
      </div>
    </div>
  )
}
