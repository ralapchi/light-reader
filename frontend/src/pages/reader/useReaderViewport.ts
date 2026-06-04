import { useCallback, useEffect, useRef, useState, type CSSProperties, type RefObject } from 'react'

export interface ReaderViewport {
  viewportHeight: number
  viewportWidth: number
  lineHeightPx: number
}

/**
 * Computes the usable content area dimensions for the reader.
 * Observes the content container via ResizeObserver and aligns height to line-height multiples
 * so that CSS columns never clip a line mid-height.
 */
export function useReaderViewport(
  contentRef: RefObject<HTMLDivElement | null>,
  contentStyle: CSSProperties,
): ReaderViewport {
  const [viewportHeight, setViewportHeight] = useState(600)
  const [viewportWidth, setViewportWidth] = useState(400)
  const [lineHeightPx, setLineHeightPx] = useState(30)

  const contentStyleRef = useRef(contentStyle)
  useEffect(() => { contentStyleRef.current = contentStyle }, [contentStyle])

  const measure = useCallback(() => {
    const el = contentRef.current
    if (!el) return
    const rect = el.getBoundingClientRect()
    if (rect.height <= 0 || rect.width <= 0) return

    const style = contentStyleRef.current
    const fontSize = Number.parseFloat(String(style.fontSize ?? '18')) || 18
    const lh = typeof style.lineHeight === 'number'
      ? style.lineHeight
      : Number.parseFloat(String(style.lineHeight ?? '1.85')) || 1.85
    const linePx = fontSize * lh

    const alignedH = Math.max(linePx * 4, Math.floor(rect.height / linePx) * linePx)

    setViewportHeight(alignedH)
    setViewportWidth(rect.width)
    setLineHeightPx(linePx)
  }, [contentRef])

  useEffect(() => {
    let debounceTimer: ReturnType<typeof setTimeout> | null = null
    const debouncedMeasure = () => {
      if (debounceTimer) clearTimeout(debounceTimer)
      debounceTimer = setTimeout(measure, 100)
    }

    const t = window.setTimeout(measure, 0)

    const el = contentRef.current
    const observer = el ? new ResizeObserver(debouncedMeasure) : null
    if (el) observer?.observe(el)
    window.addEventListener('resize', debouncedMeasure)
    return () => {
      window.clearTimeout(t)
      if (debounceTimer) clearTimeout(debounceTimer)
      observer?.disconnect()
      window.removeEventListener('resize', debouncedMeasure)
    }
  /* eslint-disable react-hooks/refs -- contentRef.current must trigger re-attach on reading mode switch */
  }, [contentRef, measure, contentRef.current])
  /* eslint-enable react-hooks/refs */

  return { viewportHeight, viewportWidth, lineHeightPx }
}
