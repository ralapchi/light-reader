import type { CSSProperties } from 'react'
import type { ReaderBlockDto } from '../../services/api'

interface ReaderBlockProps {
  block: ReaderBlockDto
  imageCache: Record<string, string>
  paragraphStyle?: CSSProperties
  highlight?: boolean
}

export default function ReaderBlock({ block, imageCache, paragraphStyle, highlight }: ReaderBlockProps) {
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

  const cls = `reader-paragraph ${block.kind === 'quote' ? 'quote' : 'indent'}${highlight ? ' tts-highlight' : ''}`
  return <p className={cls} style={paragraphStyle} data-para-index={block.index}>{block.text}</p>
}
