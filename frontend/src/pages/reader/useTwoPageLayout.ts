import { useCallback, useEffect, useRef, useState, type CSSProperties } from 'react'
import type { ReaderChapterDto } from '../../services/api'

export function useTwoPageLayout(
  contentRef: React.RefObject<HTMLDivElement | null>,
  contentStyle: CSSProperties,
  flowChapters: ReaderChapterDto[],
) {
  const [pageHeight, setPageHeight] = useState(600)
  const [pageWidth, setPageWidth] = useState(400)
  const [spineGap, setSpineGap] = useState(32)
  const [chapterGaps, setChapterGaps] = useState<number[]>([])
  const [totalPages, setTotalPages] = useState(2)
  const scrollRef = useRef<HTMLDivElement | null>(null)
  const chapterRefs = useRef<(HTMLDivElement | null)[]>([])

  // ── measureLayout ──────────────────────────────────────────

  const measureLayout = useCallback(() => {
    const el = scrollRef.current
    if (!el) return
    const rect = el.getBoundingClientRect()
    if (rect.height <= 0 || rect.width <= 0) return

    const fontSize = Number.parseFloat(String(contentStyle.fontSize ?? '18')) || 18
    const lh = typeof contentStyle.lineHeight === 'number'
      ? contentStyle.lineHeight
      : Number.parseFloat(String(contentStyle.lineHeight ?? '1.85')) || 1.85
    const linePx = fontSize * lh
    const alignedH = Math.max(linePx * 4, Math.floor(rect.height / linePx) * linePx)

    let gap = Math.floor(rect.width / 12)
    if (gap % 2 !== 0) gap -= 1
    if (gap < 16) gap = 16

    const colW = (rect.width - gap) / 2

    setPageHeight(alignedH)
    setPageWidth(colW)
    setSpineGap(gap)
  }, [contentStyle])

  // ── ResizeObserver + window.resize (debounced) ──────────────

  useEffect(() => {
    let debounceTimer: ReturnType<typeof setTimeout> | null = null
    const debouncedMeasure = () => {
      if (debounceTimer) clearTimeout(debounceTimer)
      debounceTimer = setTimeout(measureLayout, 100)
    }

    const t = window.setTimeout(measureLayout, 0)

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
  }, [contentRef, measureLayout])

  // ── Chapter gap alignment ──────────────────────────────────

  const adjustChapterGaps = useCallback(() => {
    if (flowChapters.length <= 1) {
      setChapterGaps([])
      return
    }
    const step = pageWidth + spineGap
    if (step <= 0) return
    const refs = chapterRefs.current
    const gaps: number[] = []
    for (let i = 1; i < flowChapters.length; i++) {
      const el = refs[i]
      let gap = pageHeight
      if (el) {
        const rect = el.getBoundingClientRect()
        const col = Math.round(rect.left / step)
        if (col % 2 !== 0) gap = pageHeight * 2
      }
      gaps.push(gap)
    }
    setChapterGaps(prev => {
      if (prev.length !== gaps.length) return gaps
      if (prev.every((g, i) => g === gaps[i])) return prev
      return gaps
    })
  }, [flowChapters, pageWidth, spineGap, pageHeight])

  useEffect(() => {
    const t = window.setTimeout(adjustChapterGaps, 0)
    return () => window.clearTimeout(t)
  }, [adjustChapterGaps])

  // ── Total page count ───────────────────────────────────────

  const updatePageCount = useCallback(() => {
    const el = scrollRef.current
    if (!el) return
    const step = el.clientWidth + spineGap
    if (step <= 0) return
    setTotalPages(2 * Math.max(1, Math.round(el.scrollWidth / step)))
  }, [spineGap])

  useEffect(() => {
    const t = window.setTimeout(updatePageCount, 0)
    return () => window.clearTimeout(t)
  }, [updatePageCount, pageHeight, flowChapters])

  const totalSpreads = Math.max(1, Math.ceil(totalPages / 2))
  const totalSpreadsRef = useRef(totalSpreads)
  useEffect(() => { totalSpreadsRef.current = totalSpreads }, [totalSpreads])

  return {
    scrollRef,
    chapterRefs,
    pageHeight,
    pageWidth,
    spineGap,
    chapterGaps,
    totalSpreads,
    totalSpreadsRef,
  }
}
