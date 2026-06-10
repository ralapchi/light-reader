import { memo, useMemo, type CSSProperties } from 'react'
import type { ReaderBlockDto, ReaderTextLinkDto } from '../../services/api'

interface ReaderBlockProps {
  block: ReaderBlockDto
  imageCache: Record<string, string>
  paragraphStyle?: CSSProperties
  highlight?: boolean
  onLinkClick?: (href: string) => void
  onLinkHover?: (href: string, target: HTMLElement, title?: string | null) => void
  onLinkLeave?: () => void
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
        {renderLinkedText(block.text, sortedLinks ?? [], onLinkClick, onLinkHover, onLinkLeave)}
      </h2>
    )
  }

  if (block.type === 'quote') {
    return (
      <blockquote className={`reader-paragraph quote${highlight ? ' tts-highlight' : ''}`} style={paragraphStyle} data-para-index={block.index}>
        {renderLinkedText(block.text, sortedLinks ?? [], onLinkClick, onLinkHover, onLinkLeave)}
      </blockquote>
    )
  }

  const cls = `reader-paragraph indent${highlight ? ' tts-highlight' : ''}`
  return (
    <p className={cls} style={paragraphStyle} data-para-index={block.index}>
      {renderLinkedText(block.text, sortedLinks ?? [], onLinkClick, onLinkHover, onLinkLeave)}
    </p>
  )
})
