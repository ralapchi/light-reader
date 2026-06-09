import { memo, type CSSProperties } from 'react'
import type { ReaderBlockDto, ReaderTextLinkDto } from '../../services/api'

interface ReaderBlockProps {
  block: ReaderBlockDto
  imageCache: Record<string, string>
  paragraphStyle?: CSSProperties
  highlight?: boolean
  onLinkClick?: (href: string) => void
}

function renderLinkedText(text: string, links: ReaderTextLinkDto[], onLinkClick?: (href: string) => void) {
  if (!links || links.length === 0) return text

  const parts: React.ReactNode[] = []
  const chars = Array.from(text)
  let cursor = 0

  const sorted = [...links].sort((a, b) => a.start - b.start)

  for (const link of sorted) {
    if (link.start < cursor || link.start >= chars.length || link.end <= link.start) continue

    if (link.start > cursor) {
      parts.push(chars.slice(cursor, link.start).join(''))
    }

    const linkText = chars.slice(link.start, link.end).join('')
    parts.push(
      <button
        key={`link-${link.start}`}
        className="reader-link"
        onClick={(e) => {
          e.stopPropagation()
          onLinkClick?.(link.href)
        }}
        title={link.title || link.href}
      >
        {linkText}
      </button>,
    )
    cursor = link.end
  }

  if (cursor < chars.length) {
    parts.push(chars.slice(cursor).join(''))
  }

  return <>{parts}</>
}

export default memo(function ReaderBlock({ block, imageCache, paragraphStyle, highlight, onLinkClick }: ReaderBlockProps) {
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
        {renderLinkedText(block.text, block.links ?? [], onLinkClick)}
      </h2>
    )
  }

  if (block.type === 'quote') {
    return (
      <blockquote className={`reader-paragraph quote${highlight ? ' tts-highlight' : ''}`} style={paragraphStyle} data-para-index={block.index}>
        {renderLinkedText(block.text, block.links ?? [], onLinkClick)}
      </blockquote>
    )
  }

  const cls = `reader-paragraph indent${highlight ? ' tts-highlight' : ''}`
  return (
    <p className={cls} style={paragraphStyle} data-para-index={block.index}>
      {renderLinkedText(block.text, block.links ?? [], onLinkClick)}
    </p>
  )
})
