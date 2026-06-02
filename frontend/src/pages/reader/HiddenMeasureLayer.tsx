import type { CSSProperties } from 'react'
import type { ReaderBlockDto } from '../../services/api'
import ReaderBlock from './ReaderBlock'

interface HiddenMeasureLayerProps {
  block: ReaderBlockDto
  contentStyle: CSSProperties
  paragraphStyle: CSSProperties
  textLayerWidth: number
  imageCache: Record<string, string>
}

/**
 * Off-screen measurement container that mirrors the real text layer's CSS.
 * NOT display:none — browsers report 0 height for hidden elements.
 * Uses position:absolute + visibility:hidden so layout is computed normally.
 */
export default function HiddenMeasureLayer({
  block,
  contentStyle,
  paragraphStyle,
  textLayerWidth,
  imageCache,
}: HiddenMeasureLayerProps) {
  return (
    <div
      className="tx"
      style={{
        position: 'absolute',
        left: '-9999px',
        top: 0,
        width: `${textLayerWidth}px`,
        visibility: 'hidden',
        ...contentStyle,
      }}
    >
      <ReaderBlock
        block={block}
        imageCache={imageCache}
        paragraphStyle={paragraphStyle}
      />
    </div>
  )
}
