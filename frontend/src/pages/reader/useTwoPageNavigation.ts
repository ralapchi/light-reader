import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState, type RefObject } from 'react'
import type { ReaderChapterDto } from '../../services/api'
import type { TwoPageNav } from './TwoPageReaderContent'

export function useTwoPageNavigation(
  contentRef: RefObject<HTMLDivElement | null>,
  scrollRef: RefObject<HTMLDivElement | null>,
  totalSpreadsRef: React.MutableRefObject<number>,
  pageWidth: number,
  spineGap: number,
  totalSpreads: number,
  flowChapters: ReaderChapterDto[],
  chapter: ReaderChapterDto | null,
  hasNextChapter: boolean,
  loadNextChapter: () => Promise<boolean>,
  setExtraChapters: (chapters: ReaderChapterDto[]) => void,
  twoPageNavRef: React.MutableRefObject<TwoPageNav | null>,
  onNextChapter: (() => void) | undefined,
  onPreviousChapter: (() => void) | undefined,
  onNavigate: (() => void) | undefined,
  initialParagraphIndex: number | null | undefined,
) {
  const [spreadIndex, setSpreadIndex] = useState(0)
  const innerRef = useRef<HTMLDivElement | null>(null)

  const spreadIndexRef = useRef(spreadIndex)
  useEffect(() => { spreadIndexRef.current = spreadIndex }, [spreadIndex])

  const activeSpreadIndex = Math.min(spreadIndex, totalSpreads - 1)

  // ── Scroll to current spread ───────────────────────────────

  useLayoutEffect(() => {
    const el = scrollRef.current
    if (!el) return
    const step = el.clientWidth + spineGap
    const target = activeSpreadIndex * step
    if (Math.abs(el.scrollLeft - target) > 1) {
      el.scrollLeft = target
    }
  }, [activeSpreadIndex, spineGap, scrollRef])

  // ── Navigation callbacks ───────────────────────────────────

  const recalcSpreads = useCallback(() => {
    totalSpreadsRef.current = totalSpreads
  }, [totalSpreads, totalSpreadsRef])

  const goToSpread = useCallback((index: number) => {
    setSpreadIndex(() => Math.max(0, Math.min(totalSpreadsRef.current - 1, index)))
  }, [totalSpreadsRef])

  const findSpreadByParagraph = useCallback((paragraphIndex: number): number | null => {
    const el = scrollRef.current
    if (!el) return null
    const step = pageWidth + spineGap
    if (step <= 0) return null

    const para = el.querySelector(`.reader-paragraph[data-para-index="${paragraphIndex}"]`) as HTMLElement | null
    if (!para) return null

    const textEl = el.querySelector('.reader-page-text') as HTMLElement | null
    const textLeft = textEl?.getBoundingClientRect().left ?? 0
    const col = Math.floor((para.getBoundingClientRect().left - textLeft) / step)
    return Math.max(0, Math.floor(col / 2))
  }, [pageWidth, spineGap, scrollRef])

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
  }, [goToSpread, recalcSpreads, onNextChapter, onPreviousChapter, onNavigate, hasNextChapter, loadNextChapter, totalSpreadsRef, spreadIndexRef])

  const nextSpread = useCallback(() => turnSpread(1), [turnSpread])
  const prevSpread = useCallback(() => turnSpread(-1), [turnSpread])

  // ── Visible chapter index ───────────────────────────────────

  const visibleChapterIndex = useMemo(() => {
    if (flowChapters.length === 0) return chapter?.chapter_index ?? 0
    return flowChapters[0].chapter_index
  }, [flowChapters, chapter])

  // ── Export TwoPageNav to parent ────────────────────────────

  const nav: TwoPageNav = useMemo(() => ({
    findSpreadByParagraph,
    goToSpread, recalcSpreads,
    spreadIndex, spreadCount: totalSpreads,
    currentChapterIndex: visibleChapterIndex, innerRef,
  }), [findSpreadByParagraph, goToSpread, recalcSpreads, spreadIndex, totalSpreads, visibleChapterIndex, innerRef])

  useEffect(() => {
    twoPageNavRef.current = nav
  }, [nav, twoPageNavRef])

  // ── Chapter reset ──────────────────────────────────────────

  useEffect(() => {
    totalSpreadsRef.current = totalSpreads
    requestAnimationFrame(() => {
      setExtraChapters([])
      setSpreadIndex(0)
    })
    // eslint-disable-next-line react-hooks/exhaustive-deps -- setExtraChapters is stable, spreadIndex intentionally excluded
  }, [chapter?.chapter_index, totalSpreads])

  // ── Clamp spread when total shrinks ────────────────────────

  useEffect(() => {
    totalSpreadsRef.current = totalSpreads
    requestAnimationFrame(() => {
      setSpreadIndex(s => s < totalSpreads ? s : Math.max(0, totalSpreads - 1))
    })
  }, [totalSpreads, totalSpreadsRef])

  // ── Auto preload adjacent chapters ─────────────────────────

  useEffect(() => {
    if (totalSpreads - spreadIndex > 2 || !hasNextChapter) return
    requestAnimationFrame(() => { loadNextChapter() })
  }, [hasNextChapter, loadNextChapter, spreadIndex, totalSpreads])

  // ── Keyboard + wheel ───────────────────────────────────────

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

  // ── Initial paragraph navigation ───────────────────────────

  useEffect(() => {
    if (initialParagraphIndex == null || totalSpreads <= 1) return
    requestAnimationFrame(() => {
      const spread = findSpreadByParagraph(initialParagraphIndex)
      if (spread != null) goToSpread(spread)
      else setSpreadIndex(0)
    })
  }, [initialParagraphIndex, totalSpreads, findSpreadByParagraph, goToSpread])

  return {
    spreadIndex,
    activeSpreadIndex,
    innerRef,
    goToSpread,
    turnSpread,
    nextSpread,
    prevSpread,
    findSpreadByParagraph,
    recalcSpreads,
    visibleChapterIndex,
  }
}
