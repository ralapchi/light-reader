import type { CSSProperties, RefObject } from 'react'
import type { ReaderBlockDto, ReaderChapterDto, ReadingMode } from '../../services/api'
import SinglePageReaderContent from './SinglePageReaderContent'
import TwoPageReaderContent, { type TwoPageNav, type TwoPageVisibleChapter } from './TwoPageReaderContent'

export type { TwoPageNav }

interface ReaderContentProps {
  chapter: ReaderChapterDto | null
  chapterCount: number
  contentRef: RefObject<HTMLDivElement | null>
  contentStyle: CSSProperties
  contentWidth: number
  highlightedParagraphIndex?: number
  imageCache: Record<string, string>
  loadChapterImages: (blocks: ReaderBlockDto[]) => Promise<void>
  initialParagraphIndex?: number | null
  twoPageNavRef: React.MutableRefObject<TwoPageNav | null>
  onNextChapter?: () => void
  onPreviousChapter?: () => void
  onScroll: () => void
  onLinkClick?: (href: string) => void
  onLinkHover?: (href: string, target: HTMLElement, title?: string | null) => void
  onLinkLeave?: () => void
  onNavigate?: () => void
  onVisibleChapterChange?: (visible: TwoPageVisibleChapter | null) => void
  paragraphStyle: CSSProperties
  readingMode?: ReadingMode
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
        loadChapterImages={props.loadChapterImages}
        initialParagraphIndex={props.initialParagraphIndex}
        twoPageNavRef={props.twoPageNavRef}
        onNextChapter={props.onNextChapter}
        onPreviousChapter={props.onPreviousChapter}
        onLinkClick={props.onLinkClick}
        onLinkHover={props.onLinkHover}
        onLinkLeave={props.onLinkLeave}
        onNavigate={props.onNavigate}
        onVisibleChapterChange={props.onVisibleChapterChange}
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
      onNextChapter={props.onNextChapter}
      onPreviousChapter={props.onPreviousChapter}
      onScroll={props.onScroll}
      onLinkClick={props.onLinkClick}
      onLinkHover={props.onLinkHover}
      onLinkLeave={props.onLinkLeave}
      onNavigate={props.onNavigate}
      paragraphStyle={props.paragraphStyle}
    />
  )
}
