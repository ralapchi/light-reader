import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState, type CSSProperties } from 'react'
import type { ReaderChapterDto } from '../../services/api'

const DEFAULT_STATUS_BAR_HEIGHT = 32
const MIN_CHAPTER_PAGE_COUNT = 2

interface PageModel {
  key: string
  pageCounts: number[]
  contentPageCounts: number[]
}

function evenPageCount(pageCount: number) {
  const safeCount = Math.max(1, pageCount)
  return safeCount % 2 === 0 ? safeCount : safeCount + 1
}

export function useTwoPageLayout(
  contentRef: React.RefObject<HTMLDivElement | null>,
  contentStyle: CSSProperties,
  flowChapters: ReaderChapterDto[],
) {
  const [pageHeight, setPageHeight] = useState(600)
  const [pageWidth, setPageWidth] = useState(400)
  const [spineGap, setSpineGap] = useState(32)
  const [statusBarHeight, setStatusBarHeight] = useState(DEFAULT_STATUS_BAR_HEIGHT)
  const [isReady, setIsReady] = useState(false)
  const [pageModel, setPageModel] = useState<PageModel>({ key: '', pageCounts: [], contentPageCounts: [] })
  const scrollRef = useRef<HTMLDivElement | null>(null)
  const chapterRefs = useRef<(HTMLDivElement | null)[]>([])

  // ── measureLayout ──────────────────────────────────────────

  const measureLayout = useCallback(() => {
    const el = contentRef.current
    if (!el) return
    const rect = el.getBoundingClientRect()
    if (rect.height <= 0 || rect.width <= 0) return

    const fontSize = Number.parseFloat(String(contentStyle.fontSize ?? '18')) || 18
    const lh = typeof contentStyle.lineHeight === 'number'
      ? contentStyle.lineHeight
      : Number.parseFloat(String(contentStyle.lineHeight ?? '1.85')) || 1.85
    const linePx = fontSize * lh

    const cs = getComputedStyle(el)
    const totalPadding = parseFloat(cs.paddingTop) + parseFloat(cs.paddingBottom)
    const horizontalPadding = parseFloat(cs.paddingLeft) + parseFloat(cs.paddingRight)
    const available = window.innerHeight - totalPadding
    const minStatusBar = DEFAULT_STATUS_BAR_HEIGHT
    const N = Math.max(4, Math.floor((available - minStatusBar) / linePx))
    const newStatusBarHeight = available - N * linePx
    const alignedH = N * linePx

    const usableWidth = Math.max(360, rect.width - horizontalPadding)
    let gap = Math.floor(usableWidth / 12)
    if (gap % 2 !== 0) gap -= 1
    if (gap < 16) gap = 16

    const colW = (usableWidth - gap * 2) / 2

    setStatusBarHeight(prev => prev === newStatusBarHeight ? prev : newStatusBarHeight)
    setPageHeight(prev => prev === alignedH ? prev : alignedH)
    setPageWidth(prev => prev === colW ? prev : colW)
    setSpineGap(prev => prev === gap ? prev : gap)
    setIsReady(true)
  }, [contentStyle, contentRef])

  // ── ResizeObserver + window.resize (debounced) ──────────────

  useEffect(() => {
    let debounceTimer: ReturnType<typeof setTimeout> | null = null
    const debouncedMeasure = () => {
      if (debounceTimer) clearTimeout(debounceTimer)
      debounceTimer = setTimeout(measureLayout, 100)
    }

    const t = window.setTimeout(() => {
      document.fonts.ready.then(() => {
        requestAnimationFrame(measureLayout)
      })
    }, 0)

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

  // ── Chapter-level page model ───────────────────────────────

  const chapterKey = useMemo(
    () => flowChapters.map(ch => ch.chapter_index).join(','),
    [flowChapters],
  )
  const layoutKey = `${chapterKey}|${pageHeight}|${pageWidth}|${spineGap}`
  const renderedPageCounts = pageModel.key === layoutKey
    ? pageModel.pageCounts
    : flowChapters.map(() => MIN_CHAPTER_PAGE_COUNT)
  const renderedContentPageCounts = pageModel.key === layoutKey
    ? pageModel.contentPageCounts
    : flowChapters.map(() => 1)

  useLayoutEffect(() => {
    if (flowChapters.length === 0 || pageWidth <= 0) {
      setPageModel({ key: layoutKey, pageCounts: [], contentPageCounts: [] })
      return
    }

    const colStep = pageWidth + spineGap
    if (colStep <= 0) return

    const contentPageCounts = flowChapters.map((_, index) => {
      const el = chapterRefs.current[index]
      if (!el) return 1
      const textEl = el.querySelector('.reader-page-text') as HTMLElement | null
      const blocks = el.querySelectorAll('.reader-block-shell')
      const lastBlock = blocks.length > 0 ? (blocks[blocks.length - 1] as HTMLElement) : null
      const naturalRight = lastBlock
        ? lastBlock.getBoundingClientRect().right - (textEl ?? el).getBoundingClientRect().left
        : (textEl?.scrollWidth ?? el.scrollWidth)
      return Math.max(1, Math.ceil(Math.max(pageWidth, naturalRight) / colStep))
    })
    const pageCounts = contentPageCounts.map(evenPageCount)

    setPageModel(prev => {
      if (
        prev.key === layoutKey &&
        prev.pageCounts.length === pageCounts.length &&
        prev.pageCounts.every((count, index) => count === pageCounts[index]) &&
        prev.contentPageCounts.every((count, index) => count === contentPageCounts[index])
      ) {
        return prev
      }
      return { key: layoutKey, pageCounts, contentPageCounts }
    })
  }, [flowChapters, layoutKey, pageWidth, spineGap])

  const chapterSpreadStarts = useMemo(() => {
    const starts: number[] = []
    let spread = 0
    for (const pageCount of renderedPageCounts) {
      starts.push(spread)
      spread += Math.max(1, pageCount / 2)
    }
    return starts
  }, [renderedPageCounts])

  const totalSpreads = Math.max(
    1,
    renderedPageCounts.reduce((sum, pageCount) => sum + Math.max(1, pageCount / 2), 0),
  )
  const totalSpreadsRef = useRef(totalSpreads)
  useEffect(() => { totalSpreadsRef.current = totalSpreads }, [totalSpreads])

  return {
    scrollRef,
    chapterRefs,
    pageHeight,
    pageWidth,
    spineGap,
    chapterPageCounts: renderedPageCounts,
    chapterContentPageCounts: renderedContentPageCounts,
    chapterSpreadStarts,
    totalSpreads,
    totalSpreadsRef,
    statusBarHeight,
    isReady,
  }
}
