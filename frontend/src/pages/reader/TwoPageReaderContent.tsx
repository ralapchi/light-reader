import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState, type CSSProperties, type RefObject } from 'react'
import { readerGetChapter } from '../../services/api'
import type { ReaderChapterDto } from '../../services/api'
import ReaderBlock from './ReaderBlock'
import { blockKey, blockParagraphIndex } from './readerUtils'

export interface TwoPageNav {
  findSpreadByParagraph: (paragraphIndex: number) => number | null
  goToSpread: (index: number) => void
  recalcSpreads: () => void
  spreadIndex: number
  spreadCount: number
  currentChapterIndex: number
  innerRef: React.RefObject<HTMLDivElement | null>
}

const breakBeforeColumn = { breakBefore: 'column' } as CSSProperties
const columnFillAuto = { columnFill: 'auto' } as CSSProperties

interface TwoPageReaderContentProps {
  chapter: ReaderChapterDto | null
  chapterCount: number
  contentRef: RefObject<HTMLDivElement | null>
  contentStyle: CSSProperties
  highlightedParagraphIndex?: number
  imageCache: Record<string, string>
  initialParagraphIndex?: number | null
  onNextChapter?: () => void
  onPreviousChapter?: () => void
  onLinkClick?: (href: string) => void
  onNavigate?: () => void
  paragraphStyle: CSSProperties
}

export default function TwoPageReaderContent({
  chapter,
  chapterCount,
  contentRef,
  contentStyle,
  highlightedParagraphIndex,
  imageCache,
  initialParagraphIndex,
  onNextChapter,
  onPreviousChapter,
  onLinkClick,
  onNavigate,
  paragraphStyle,
}: TwoPageReaderContentProps) {
  const [spreadIndex, setSpreadIndex] = useState(0)
  const [extraChapters, setExtraChapters] = useState<ReaderChapterDto[]>([])
  const [loadingChapterIndex, setLoadingChapterIndex] = useState<number | null>(null)
  const [pageHeight, setPageHeight] = useState(600)
  const [pageWidth, setPageWidth] = useState(400)
  const [spineGap, setSpineGap] = useState(32)
  const [chapterGaps, setChapterGaps] = useState<number[]>([])
  const scrollRef = useRef<HTMLDivElement | null>(null)
  const innerRef = useRef<HTMLDivElement | null>(null)
  const chapterRefs = useRef<(HTMLDivElement | null)[]>([])

  const flowChapters = useMemo(() => chapter ? [chapter, ...extraChapters] : [], [chapter, extraChapters])
  const flowChaptersRef = useRef(flowChapters)
  useEffect(() => { flowChaptersRef.current = flowChapters }, [flowChapters])

  // Measure after render — like koodo-reader's getPageSize()
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

    // Column width = (container - gap) / 2 — one page
    const colW = (rect.width - gap) / 2

    setPageHeight(alignedH)
    setPageWidth(colW)
    setSpineGap(gap)
  }, [contentStyle])

  // Re-measure when chapters change (content height changes).
  // ResizeObserver + window.resize share a 100ms debounced handler
  // so rapid resize events are collapsed into a single remeasure.
  useEffect(() => {
    let debounceTimer: ReturnType<typeof setTimeout> | null = null
    const debouncedMeasure = () => {
      if (debounceTimer) clearTimeout(debounceTimer)
      debounceTimer = setTimeout(measureLayout, 100)
    }

    // Initial measure — fire once after layout
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

  // Ensure each chapter starts on an odd column (left page).
  // CSS columns + break-before:column can't guarantee column parity, so we
  // measure positions after render and adjust spacers.
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
      let gap = pageHeight  // base spacer = one page height
      if (el) {
        const rect = el.getBoundingClientRect()
        const col = Math.round(rect.left / step)
        // Even columns (0, 2, 4...) = left page. Odd columns (1, 3, 5...) = right page.
        if (col % 2 !== 0) {
          // On a right page — push one more column to land on left page
          gap = pageHeight * 2
        }
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

  // Total pages from column scrollWidth — like koodo-reader's k()
  const [totalPages, setTotalPages] = useState(2)

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

  const spreadIndexRef = useRef(spreadIndex)

  useEffect(() => {
    totalSpreadsRef.current = totalSpreads
  }, [totalSpreads])

  useEffect(() => {
    spreadIndexRef.current = spreadIndex
  }, [spreadIndex])

  const activeSpreadIndex = Math.min(spreadIndex, totalSpreads - 1)

  // Scroll to show current spread — like koodo-reader's body.scrollTo()
  useLayoutEffect(() => {
    const el = scrollRef.current
    if (!el) return
    const step = el.clientWidth + spineGap
    const target = activeSpreadIndex * step
    if (Math.abs(el.scrollLeft - target) > 1) {
      el.scrollLeft = target
    }
  }, [activeSpreadIndex, spineGap])

  const recalcSpreads = useCallback(() => {
    totalSpreadsRef.current = totalSpreads
  }, [totalSpreads])

  const goToSpread = useCallback((index: number) => {
    setSpreadIndex(() => Math.max(0, Math.min(totalSpreadsRef.current - 1, index)))
  }, [])

  const findSpreadByParagraph = useCallback((paragraphIndex: number): number | null => {
    const el = scrollRef.current
    if (!el) return null
    const step = pageWidth + spineGap
    if (step <= 0) return null

    const para = el.querySelector(`.reader-paragraph[data-para-index="${paragraphIndex}"]`) as HTMLElement | null
    if (!para) return null

    const txEl = el.querySelector('.tx') as HTMLElement | null
    const txLeft = txEl?.getBoundingClientRect().left ?? 0
    const col = Math.floor((para.getBoundingClientRect().left - txLeft) / step)
    return Math.max(0, Math.floor(col / 2))
  }, [pageWidth, spineGap])

  const loadNextChapter = useCallback(async (): Promise<boolean> => {
    const chapters = flowChaptersRef.current
    const lastLoaded = chapters[chapters.length - 1]
    if (!lastLoaded) return false
    const nextIndex = lastLoaded.chapter_index + 1
    if (nextIndex >= chapterCount || loadingChapterIndex === nextIndex) return false
    setLoadingChapterIndex(nextIndex)
    try {
      const nextChapter = await readerGetChapter(nextIndex)
      setExtraChapters(c => c.some(x => x.chapter_index === nextIndex) ? c : [...c, nextChapter])
      return true
    } finally { setLoadingChapterIndex(null) }
  }, [chapterCount, loadingChapterIndex])

  const hasNextChapter = useMemo(() => {
    const lastLoaded = flowChapters[flowChapters.length - 1]
    return !!lastLoaded && lastLoaded.chapter_index < chapterCount - 1
  }, [chapterCount, flowChapters])

  const turnSpread = useCallback((delta: number) => {
    onNavigate?.()
    recalcSpreads()
    const current = spreadIndexRef.current
    const total = totalSpreadsRef.current
    if (delta > 0 && current >= total - 1) {
      if (hasNextChapter) {
        loadNextChapter().then(loaded => {
          if (loaded) requestAnimationFrame(() => {
            setSpreadIndex(s => Math.min(s + 1, totalSpreadsRef.current - 1))
          })
        })
        return
      }
      onNextChapter?.()
      return
    }
    if (delta < 0 && current === 0) {
      onPreviousChapter?.()
      return
    }
    goToSpread(current + delta)
  }, [goToSpread, recalcSpreads, onNextChapter, onPreviousChapter, onNavigate, hasNextChapter, loadNextChapter])

  const nextSpread = useCallback(() => turnSpread(1), [turnSpread])
  const prevSpread = useCallback(() => turnSpread(-1), [turnSpread])

  const visibleChapterIndex = useMemo(() => {
    if (flowChapters.length === 0) return chapter?.chapter_index ?? 0
    return flowChapters[0].chapter_index
  }, [flowChapters, chapter])

  useEffect(() => {
    const el = contentRef.current
    if (!el) return
    const nav: TwoPageNav = {
      findSpreadByParagraph,
      goToSpread, recalcSpreads,
      spreadIndex, spreadCount: totalSpreads,
      currentChapterIndex: visibleChapterIndex, innerRef,
    }
    ;(el as HTMLDivElement & { __twoPageNav?: TwoPageNav }).__twoPageNav = nav
    return () => { delete (el as HTMLDivElement & { __twoPageNav?: TwoPageNav }).__twoPageNav }
  }, [findSpreadByParagraph, goToSpread, recalcSpreads, spreadIndex, totalSpreads, visibleChapterIndex, contentRef])

  useEffect(() => {
    totalSpreadsRef.current = totalSpreads
    requestAnimationFrame(() => {
      setExtraChapters([])
      setSpreadIndex(0)
    })
  }, [chapter?.chapter_index])

  useEffect(() => {
    totalSpreadsRef.current = totalSpreads
    if (spreadIndex < totalSpreads) return
    requestAnimationFrame(() => setSpreadIndex(Math.max(0, totalSpreads - 1)))
  }, [totalSpreads])

  useEffect(() => {
    if (totalSpreads - spreadIndex > 2 || !hasNextChapter) return
    requestAnimationFrame(() => { loadNextChapter() })
  }, [hasNextChapter, loadNextChapter, spreadIndex, totalSpreads])

  // Keyboard + wheel
  useEffect(() => {
    const el = contentRef.current
    if (!el) return
    let wheelAccum = 0
    let wheelTimer: ReturnType<typeof setTimeout> | null = null
    const onWheel = (e: WheelEvent) => {
      e.preventDefault()
      wheelAccum += e.deltaY
      if (wheelTimer) clearTimeout(wheelTimer)
      wheelTimer = setTimeout(() => { wheelAccum = 0 }, 200)
      if (Math.abs(wheelAccum) > 50) {
        if (wheelAccum > 0) nextSpread()
        else prevSpread()
        wheelAccum = 0
        if (wheelTimer) clearTimeout(wheelTimer)
      }
    }
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'ArrowRight' || e.key === 'PageDown') { e.preventDefault(); nextSpread() }
      else if (e.key === 'ArrowLeft' || e.key === 'PageUp') { e.preventDefault(); prevSpread() }
    }
    el.addEventListener('wheel', onWheel, { passive: false })
    window.addEventListener('keydown', onKey)
    return () => {
      el.removeEventListener('wheel', onWheel)
      window.removeEventListener('keydown', onKey)
      if (wheelTimer) clearTimeout(wheelTimer)
    }
  }, [nextSpread, prevSpread, contentRef])

  useEffect(() => {
    if (initialParagraphIndex == null || totalSpreads <= 1) return
    requestAnimationFrame(() => {
      const spread = findSpreadByParagraph(initialParagraphIndex)
      if (spread != null) goToSpread(spread)
      else setSpreadIndex(0)
    })
  }, [initialParagraphIndex, totalSpreads, findSpreadByParagraph, goToSpread])

  return (
    <div className="reader-content two-page" ref={contentRef}>
      {/* Scroll container — like koodo-reader's iframe body */}
      <div
        ref={scrollRef}
        style={{
          flex: 1,
          alignSelf: 'stretch',
          overflow: 'hidden',
        }}
      >
        {/* CSS columns — like koodo-reader's F() injected styles */}
        <div
          className="tx"
          style={{
            ...contentStyle,
            columnWidth: `${pageWidth}px`,
            columnGap: `${spineGap}px`,
            ...columnFillAuto,
            height: pageHeight,
            overflow: 'visible',
          }}
        >
          {flowChapters.map((ch, ci) => {
            const needSpacer = ci > 0 && chapterGaps[ci - 1] != null && chapterGaps[ci - 1] > pageHeight
            return (
            <div
              key={ch.chapter_index}
              ref={(el) => { chapterRefs.current[ci] = el }}
              style={ci > 0 ? breakBeforeColumn : undefined}
            >
              {needSpacer && <div style={{ height: pageHeight }} />}
              {ch.blocks.map((block, i) => (
                <div className="reader-block-shell" key={blockKey(block, i)}>
                  <ReaderBlock
                    block={block}
                    imageCache={imageCache}
                    paragraphStyle={paragraphStyle}
                    highlight={blockParagraphIndex(block) === highlightedParagraphIndex}
                    onLinkClick={onLinkClick}
                  />
                </div>
              ))}
            </div>
            )
          })}
        </div>
      </div>

      {/* Click zones for page turning */}
      <div
        onClick={prevSpread}
        style={{ position: 'absolute', left: 0, top: 0, bottom: 0, width: '20%', zIndex: 2 }}
      />
      <div
        onClick={nextSpread}
        style={{ position: 'absolute', right: 0, top: 0, bottom: 0, width: '20%', zIndex: 2 }}
      />
    </div>
  )
}
