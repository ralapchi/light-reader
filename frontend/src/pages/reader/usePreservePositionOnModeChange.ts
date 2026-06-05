import { useEffect, useRef, useState } from 'react'
import type { RefObject } from 'react'
import type { ReadingMode } from '../../services/api'
import type { TwoPageNav } from './TwoPageReaderContent'
import { afterLayoutSettled } from './rafUtils'
import { captureVisibleParagraph, scrollToParagraph, scrollToParagraphTwoPage } from './readerUtils'

export function usePreservePositionOnModeChange(
  effectiveReadingMode: ReadingMode,
  contentRef: RefObject<HTMLDivElement | null>,
  twoPageNavRef: RefObject<TwoPageNav | null>,
) {
  const [layoutAnchorParagraph, setLayoutAnchorParagraph] = useState<number | null>(null)
  const prevModeRef = useRef(effectiveReadingMode)
  const anchorRef = useRef(layoutAnchorParagraph)
  useEffect(() => { anchorRef.current = layoutAnchorParagraph }, [layoutAnchorParagraph])

  useEffect(() => {
    const prev = prevModeRef.current
    prevModeRef.current = effectiveReadingMode
    if (prev === effectiveReadingMode) return
    const el = contentRef.current
    if (!el) return
    const captured = anchorRef.current ?? captureVisibleParagraph(el, prev === 'TwoPage')
    if (captured == null) return
    if (!anchorRef.current) setLayoutAnchorParagraph(captured)
    const cancel = afterLayoutSettled(() => {
      const el2 = contentRef.current
      if (!el2) return
      if (effectiveReadingMode === 'TwoPage') scrollToParagraphTwoPage(el2, captured, twoPageNavRef.current)
      else scrollToParagraph(el2, captured)
    })
    return cancel
  }, [effectiveReadingMode, contentRef, twoPageNavRef])

  return { layoutAnchorParagraph, setLayoutAnchorParagraph }
}
