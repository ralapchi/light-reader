import type { CSSProperties, RefObject } from 'react'
import type { ReaderChapterDto } from '../../services/api'
import SinglePageReaderContent from './SinglePageReaderContent'
import TwoPageReaderContent, { type TwoPageNav } from './TwoPageReaderContent'

export type { TwoPageNav }

interface ReaderContentProps {
  chapter: ReaderChapterDto | null
  chapterCount: number
  contentRef: RefObject<HTMLDivElement | null>
  contentStyle: CSSProperties
  contentWidth: number
  highlightedParagraphIndex?: number
  imageCache: Record<string, string>
  initialParagraphIndex?: number | null
  onNextChapter?: () => void
  onPreviousChapter?: () => void
  onScroll: () => void
  onLinkClick?: (href: string) => void
  onNavigate?: () => void
  paragraphStyle: CSSProperties
  readingMode?: string
}

export default function ReaderContent(props: ReaderContentProps) {
  if (props.readingMode === 'TwoPage') {
    return (
      <TwoPageReaderContent
        chapter={props.chapter}
        chapterCount={props.chapterCount}
        contentRef={props.contentRef}
        contentStyle={props.contentStyle}
        highlightedParagraphIndex={props.highlightedParagraphIndex}
        imageCache={props.imageCache}
        initialParagraphIndex={props.initialParagraphIndex}
        onNextChapter={props.onNextChapter}
        onPreviousChapter={props.onPreviousChapter}
        onLinkClick={props.onLinkClick}
        onNavigate={props.onNavigate}
        paragraphStyle={props.paragraphStyle}
      />
    )
  }

  return (
    <SinglePageReaderContent
      chapter={props.chapter}
      contentRef={props.contentRef}
      contentStyle={props.contentStyle}
      contentWidth={props.contentWidth}
      highlightedParagraphIndex={props.highlightedParagraphIndex}
      imageCache={props.imageCache}
      onScroll={props.onScroll}
      onLinkClick={props.onLinkClick}
      onNavigate={props.onNavigate}
      paragraphStyle={props.paragraphStyle}
    />
  )
}
