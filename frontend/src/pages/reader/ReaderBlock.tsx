import { memo, useMemo, type CSSProperties } from 'react'
import type { ReaderBlockDto, ReaderTextLinkDto } from '../../services/api'
import { forEachInlineImage } from '../../utils/inlineImage'

interface ReaderBlockProps {
  block: ReaderBlockDto
  imageCache: Record<string, string>
  paragraphStyle?: CSSProperties
  highlight?: boolean
  onLinkClick?: (href: string) => void
  onLinkHover?: (href: string, target: HTMLElement, title?: string | null) => void
  onLinkLeave?: () => void
}


function renderTextWithInlineImages(
  text: string,
  sortedLinks: ReaderTextLinkDto[],
  imageCache: Record<string, string>,
  onLinkClick?: (href: string) => void,
  onLinkHover?: (href: string, target: HTMLElement, title?: string | null) => void,
  onLinkLeave?: () => void,
) {
  const parts: React.ReactNode[] = []
  let lastIdx = 0
  forEachInlineImage(text, (m) => {
    // Text before this inline image — render links
    if (m.index > lastIdx) {
      const segment = text.slice(lastIdx, m.index)
      const segLinks = sortedLinks
        .filter(l => l.start >= lastIdx && l.end <= m.index)
        .map(l => ({ ...l, start: l.start - lastIdx, end: l.end - lastIdx }))
      parts.push(renderLinkedText(segment, segLinks, onLinkClick, onLinkHover, onLinkLeave))
    }
    const assetId = m[1]
    if (imageCache[assetId]) {
      parts.push(
        <img key={`iimg-${assetId}`} className="reader-inline-image" src={imageCache[assetId]} alt="" />,
      )
    }
    lastIdx = m.index + m[0].length
  })
  // Remaining text
  if (lastIdx < text.length) {
    const segment = text.slice(lastIdx)
    const segLinks = sortedLinks
      .filter(l => l.start >= lastIdx && l.end <= text.length)
      .map(l => ({ ...l, start: l.start - lastIdx, end: l.end - lastIdx }))
    parts.push(renderLinkedText(segment, segLinks, onLinkClick, onLinkHover, onLinkLeave))
  }
  return <>{parts}</>
}

function renderLinkedText(
  text: string,
  sortedLinks: ReaderTextLinkDto[],
  onLinkClick?: (href: string) => void,
  onLinkHover?: (href: string, target: HTMLElement, title?: string | null) => void,
  onLinkLeave?: () => void,
) {
  if (!sortedLinks || sortedLinks.length === 0) return text

  const parts: React.ReactNode[] = []
  let cursor = 0

  for (const link of sortedLinks) {
    if (link.start < cursor || link.start >= text.length || link.end <= link.start) continue

    if (link.start > cursor) {
      parts.push(text.slice(cursor, link.start))
    }

    parts.push(
      <button
        key={`link-${link.start}`}
        className={link.is_footnote ? 'reader-link reader-link-noteref' : 'reader-link'}
        onClick={(e) => {
          e.stopPropagation()
          onLinkClick?.(link.href)
        }}
        onMouseEnter={(e) => onLinkHover?.(link.href, e.currentTarget, link.title)}
        onMouseLeave={onLinkLeave}
        title={link.title || undefined}
      >
        {text.slice(link.start, link.end)}
      </button>,
    )
    cursor = link.end
  }

  if (cursor < text.length) {
    parts.push(text.slice(cursor))
  }

  return <>{parts}</>
}

export default memo(function ReaderBlock({ block, imageCache, paragraphStyle, highlight, onLinkClick, onLinkHover, onLinkLeave }: ReaderBlockProps) {
  const blockLinks = 'links' in block ? block.links : undefined
  const sortedLinks = useMemo(
    () => blockLinks?.length ? [...blockLinks].sort((a, b) => a.start - b.start) : undefined,
    [blockLinks]
  )

  if (block.type === 'separator') {
    return <p className="reader-paragraph separator" style={paragraphStyle}>***</p>
  }

  if (block.type === 'image') {
    return (
      <figure className="reader-image">
        {imageCache[block.asset_id] && (
          <img src={imageCache[block.asset_id]} alt={block.alt_text ?? ''} />
        )}
        {block.caption && <figcaption>{block.caption}</figcaption>}
      </figure>
    )
  }

  if (block.type === 'heading') {
    return (
      <h2 className={`reader-heading${highlight ? ' tts-highlight' : ''}`} style={paragraphStyle} data-para-index={block.index}>
        {renderTextWithInlineImages(block.text, sortedLinks ?? [], imageCache, onLinkClick, onLinkHover, onLinkLeave)}
      </h2>
    )
  }

  if (block.type === 'quote') {
    return (
      <blockquote className={`reader-paragraph quote${highlight ? ' tts-highlight' : ''}`} style={paragraphStyle} data-para-index={block.index}>
        {renderTextWithInlineImages(block.text, sortedLinks ?? [], imageCache, onLinkClick, onLinkHover, onLinkLeave)}
      </blockquote>
    )
  }

  const cls = `reader-paragraph indent${highlight ? ' tts-highlight' : ''}`
  return (
    <p className={cls} style={paragraphStyle} data-para-index={block.index}>
      {renderTextWithInlineImages(block.text, sortedLinks ?? [], imageCache, onLinkClick, onLinkHover, onLinkLeave)}
    </p>
  )
})
