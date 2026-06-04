import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState, type RefObject } from 'react'
import type { ReaderChapterDto } from '../../services/api'
import useAppStore from '../../store/useAppStore'
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
  chapterSpreadStarts: number[],
  chapterContentPageCounts: number[],
  hasNextChapter: boolean,
  loadNextChapter: () => Promise<boolean>,
  setExtraChapters: (chapters: ReaderChapterDto[]) => void,
  twoPageNavRef: React.MutableRefObject<TwoPageNav | null>,
  onNextChapter: (() => void) | undefined,
  onPreviousChapter: (() => void) | undefined,
  onNavigate: (() => void) | undefined,
  initialParagraphIndex: number | null | undefined,
  saveCurrentPosition: (() => void) | undefined,
) {
  const [spreadIndex, setSpreadIndex] = useState(0)
  const needsResetRef = useRef(false)
  const innerRef = useRef<HTMLDivElement | null>(null)

  const spreadIndexRef = useRef(spreadIndex)
  useEffect(() => { spreadIndexRef.current = spreadIndex }, [spreadIndex])

  const boundedSpreadIndex = Math.min(spreadIndex, totalSpreads - 1)
  const spreadStep = pageWidth * 2 + spineGap * 2

  const filledSpreadIndexes = useMemo(() => {
    const indexes = new Set<number>()
    for (let i = 0; i < flowChapters.length; i++) {
      const start = chapterSpreadStarts[i] ?? 0
      const contentSpreads = Math.max(1, Math.ceil((chapterContentPageCounts[i] ?? 1) / 2))
      for (let offset = 0; offset < contentSpreads; offset++) {
        indexes.add(start + offset)
      }
    }
    return indexes
  }, [chapterContentPageCounts, chapterSpreadStarts, flowChapters.length])

  const findNearestFilledSpread = useCallback((target: number, delta: number) => {
    const bounded = Math.max(0, Math.min(totalSpreads - 1, target))
    if (filledSpreadIndexes.has(bounded)) return bounded
    const step = delta >= 0 ? 1 : -1
    for (let i = bounded + step; i >= 0 && i < totalSpreads; i += step) {
      if (filledSpreadIndexes.has(i)) return i
    }
    for (let i = bounded - step; i >= 0 && i < totalSpreads; i -= step) {
      if (filledSpreadIndexes.has(i)) return i
    }
    return bounded
  }, [filledSpreadIndexes, totalSpreads])
  const activeSpreadIndex = findNearestFilledSpread(boundedSpreadIndex, 1)

  // ── Scroll to current spread ───────────────────────────────

  useLayoutEffect(() => {
    const el = scrollRef.current
    if (!el) return
    const target = activeSpreadIndex * spreadStep
    if (Math.abs(el.scrollLeft - target) > 1) {
      el.scrollLeft = target
    }
  }, [activeSpreadIndex, spreadStep, scrollRef, chapter?.chapter_index])

  // ── Navigation callbacks ───────────────────────────────────

  const recalcSpreads = useCallback(() => {
    totalSpreadsRef.current = totalSpreads
  }, [totalSpreads, totalSpreadsRef])

  const goToSpread = useCallback((index: number) => {
    setSpreadIndex(current => findNearestFilledSpread(index, index >= current ? 1 : -1))
  }, [findNearestFilledSpread])

  const findSpreadByParagraph = useCallback((paragraphIndex: number): number | null => {
    const el = scrollRef.current
    if (!el) return null
    const step = pageWidth + spineGap
    if (step <= 0) return null

    const para = el.querySelector(`.reader-paragraph[data-para-index="${paragraphIndex}"]`) as HTMLElement | null
    if (!para) return null

    const chapterEl = para.closest('[data-chapter-index]') as HTMLElement | null
    const chapterIndexAttr = chapterEl?.dataset.chapterIndex
    const chapterIndex = chapterIndexAttr == null ? null : Number(chapterIndexAttr)
    const chapterFlowIndex = chapterIndex == null
      ? -1
      : flowChapters.findIndex(ch => ch.chapter_index === chapterIndex)
    const chapterLeft = chapterEl?.getBoundingClientRect().left ?? 0
    const col = Math.floor((para.getBoundingClientRect().left - chapterLeft) / step)
    const localSpread = Math.max(0, Math.floor(col / 2))
    const spreadStart = chapterFlowIndex >= 0 ? (chapterSpreadStarts[chapterFlowIndex] ?? 0) : 0
    return spreadStart + localSpread
  }, [pageWidth, spineGap, scrollRef, flowChapters, chapterSpreadStarts])

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
    goToSpread(findNearestFilledSpread(current + delta, delta))
  }, [goToSpread, recalcSpreads, onNextChapter, onPreviousChapter, onNavigate, hasNextChapter, loadNextChapter, totalSpreadsRef, spreadIndexRef, findNearestFilledSpread])

  const nextSpread = useCallback(() => turnSpread(1), [turnSpread])
  const prevSpread = useCallback(() => turnSpread(-1), [turnSpread])

  // ── Visible chapter index ───────────────────────────────────

  const visibleChapterIndex = useMemo(() => {
    if (flowChapters.length === 0) return chapter?.chapter_index ?? 0
    let flowIndex = 0
    for (let i = 0; i < chapterSpreadStarts.length; i++) {
      if (chapterSpreadStarts[i] <= spreadIndex) flowIndex = i
      else break
    }
    return flowChapters[flowIndex]?.chapter_index ?? chapter?.chapter_index ?? 0
  }, [flowChapters, chapter, chapterSpreadStarts, spreadIndex])

  const visibleChapterProgress = useMemo(() => {
    const flowIndex = flowChapters.findIndex(ch => ch.chapter_index === visibleChapterIndex)
    if (flowIndex < 0) return 0
    const start = chapterSpreadStarts[flowIndex] ?? 0
    const nextStart = chapterSpreadStarts[flowIndex + 1] ?? totalSpreads
    const spreadCount = Math.max(1, nextStart - start)
    return Math.max(0, Math.min(1, (spreadIndex - start) / spreadCount))
  }, [chapterSpreadStarts, flowChapters, spreadIndex, totalSpreads, visibleChapterIndex])

  // ── Update progress bar on spread change ─────────────────

  const book = useAppStore(s => s.reader.book)
  const { setProgressPercent } = useAppStore()
  useEffect(() => {
    if (!book) return
    const bookPct = Math.min(1, (visibleChapterIndex + visibleChapterProgress) / book.chapter_count)
    setProgressPercent(bookPct)
  }, [visibleChapterProgress, visibleChapterIndex, book, setProgressPercent])

  // ── Export TwoPageNav to parent ────────────────────────────

  const nav: TwoPageNav = useMemo(() => ({
    findSpreadByParagraph,
    goToSpread, recalcSpreads,
    spreadIndex, spreadCount: totalSpreads,
    currentChapterIndex: visibleChapterIndex,
    currentChapterProgress: visibleChapterProgress,
    innerRef,
  }), [findSpreadByParagraph, goToSpread, recalcSpreads, spreadIndex, totalSpreads, visibleChapterIndex, visibleChapterProgress, innerRef])

  useEffect(() => {
    twoPageNavRef.current = nav
  }, [nav, twoPageNavRef])

  // ── Persist progress to backend on spread change ──────────

  const saveRef = useRef(saveCurrentPosition)
  saveRef.current = saveCurrentPosition
  useEffect(() => {
    if (nav.spreadIndex < 0) return
    requestAnimationFrame(() => saveRef.current?.())
  }, [nav])

  // ── Chapter reset ──────────────────────────────────────────

  useEffect(() => {
    totalSpreadsRef.current = totalSpreads
    setExtraChapters([])
    setSpreadIndex(0)
    needsResetRef.current = true
    // eslint-disable-next-line react-hooks/exhaustive-deps -- only reset on chapter change, not when extra chapters are added
  }, [chapter?.chapter_index])

  // ── Clamp spread when total shrinks ────────────────────────

  useEffect(() => {
    totalSpreadsRef.current = totalSpreads
    requestAnimationFrame(() => {
      if (needsResetRef.current) {
        needsResetRef.current = false
        setSpreadIndex(0)
      } else {
        setSpreadIndex(s => findNearestFilledSpread(s < totalSpreads ? s : Math.max(0, totalSpreads - 1), -1))
      }
    })
  }, [findNearestFilledSpread, totalSpreads, totalSpreadsRef])

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
