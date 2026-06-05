import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState, type RefObject } from 'react'
import { readerSaveProgress } from '../../services/api'
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
) {
  const [spreadIndex, setSpreadIndex] = useState(0)
  const needsResetRef = useRef(false)
  const needsNextSpreadRef = useRef(false)
  const clampRafRef = useRef<number | null>(null)
  const innerRef = useRef<HTMLDivElement | null>(null)


  // Track which chapter the user is viewing (index into flowChapters).
  // This is the source of truth for visibleChapterIndex — NOT derived from spreadIndex.
  const currentChapterFlowIndexRef = useRef(0)

  const spreadIndexRef = useRef(spreadIndex)
  useEffect(() => { spreadIndexRef.current = spreadIndex }, [spreadIndex])

  // Refs to avoid stale closures in async callbacks
  const chapterSpreadStartsRef = useRef(chapterSpreadStarts)
  useEffect(() => { chapterSpreadStartsRef.current = chapterSpreadStarts }, [chapterSpreadStarts])
  const flowChaptersRef = useRef(flowChapters)
  useEffect(() => { flowChaptersRef.current = flowChapters }, [flowChapters])
  const chapterRef = useRef(chapter)
  useEffect(() => { chapterRef.current = chapter }, [chapter])

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

  // ── Helper: find flowIndex for a given global spread ────────

  const findFlowIndexForSpread = useCallback((spread: number): number => {
    let flowIndex = 0
    for (let i = 0; i < chapterSpreadStarts.length; i++) {
      if (chapterSpreadStarts[i] <= spread) flowIndex = i
      else break
    }
    return flowIndex
  }, [chapterSpreadStarts])

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

  // Save progress — reads from refs to avoid stale closures
  const saveProgressForSpread = useCallback((spread: number) => {
    const bookData = useAppStore.getState().reader.book
    const bookIdent = bookData?.book_id
    if (!bookData || !bookIdent) return
    const flowIdx = currentChapterFlowIndexRef.current
    const chapters = flowChaptersRef.current
    const visChapterIndex = chapters[flowIdx]?.chapter_index ?? chapterRef.current?.chapter_index ?? 0
    const starts = chapterSpreadStartsRef.current
    const total = totalSpreadsRef.current
    const chStart = starts[flowIdx] ?? 0
    const chNextStart = starts[flowIdx + 1] ?? total
    const chSpreadCount = Math.max(1, chNextStart - chStart)
    const chProgress = Math.max(0, Math.min(1, (spread - chStart) / chSpreadCount))
    const bookPct = Math.min(1, (visChapterIndex + chProgress) / bookData.chapter_count)
    console.log(`[progress] SAVE spread=${spread} flowIdx=${flowIdx} visCh=${visChapterIndex} starts=[${starts}] chStart=${chStart} bookPct=${bookPct.toFixed(4)}`)
    useAppStore.getState().setProgressPercent(bookPct)
    readerSaveProgress({
      book_id: bookIdent,
      chapter_index: visChapterIndex,
      progress_percent: bookPct,
    }).catch(() => { /* non-critical */ })
  }, [])

  const turnSpread = useCallback((delta: number) => {
    onNavigate?.()
    recalcSpreads()
    const current = spreadIndexRef.current
    const total = totalSpreadsRef.current
    console.log(`[turnSpread] delta=${delta} current=${current} total=${total} flowChLen=${flowChaptersRef.current.length} starts=[${chapterSpreadStartsRef.current}]`)
    if (delta > 0 && current >= total - 1) {
      if (hasNextChapter) {
        console.log(`[turnSpread] → loadNextChapter (at boundary)`)
        needsNextSpreadRef.current = true
        loadNextChapter().then(loaded => {
          console.log(`[turnSpread] loadNextChapter done, loaded=${loaded}`)
          if (!loaded) {
            needsNextSpreadRef.current = false
          } else {
            // Chapter may have loaded synchronously (from preload).
            // Cancel any pending clampEffect RAF and schedule our own
            // to ensure we use fresh ref values, not stale closures.
            if (clampRafRef.current != null) cancelAnimationFrame(clampRafRef.current)
            clampRafRef.current = requestAnimationFrame(() => {
              needsNextSpreadRef.current = false
              const freshTotal = totalSpreadsRef.current
              const prev = spreadIndexRef.current
              const newSpread = Math.min(prev + 1, freshTotal - 1)
              const flowIdx = findFlowIndexForSpread(newSpread)
              currentChapterFlowIndexRef.current = flowIdx
              console.log(`[turnSpread-advance] ${prev} → ${newSpread} freshTotal=${freshTotal} flowIdx=${flowIdx}`)
              setSpreadIndex(newSpread)
              saveProgressForSpread(newSpread)
            })
          }
        })
        return
      }
      console.log(`[turnSpread] → onNextChapter`)
      onNextChapter?.()
      return
    }
    if (delta < 0 && current === 0) {
      console.log(`[turnSpread] → onPreviousChapter`)
      onPreviousChapter?.()
      return
    }
    const newSpread = findNearestFilledSpread(current + delta, delta)
    currentChapterFlowIndexRef.current = findFlowIndexForSpread(newSpread)
    console.log(`[turnSpread] → goToSpread(${newSpread}) flowIdx=${currentChapterFlowIndexRef.current}`)
    goToSpread(newSpread)
    saveProgressForSpread(newSpread)
  }, [goToSpread, recalcSpreads, onNextChapter, onPreviousChapter, onNavigate, hasNextChapter, loadNextChapter, totalSpreadsRef, spreadIndexRef, findNearestFilledSpread, findFlowIndexForSpread, saveProgressForSpread])

  const nextSpread = useCallback(() => turnSpread(1), [turnSpread])
  const prevSpread = useCallback(() => turnSpread(-1), [turnSpread])

  // ── Visible chapter index (from ref, not from spreadIndex) ──

  const visibleChapterIndex = useMemo(() => {
    if (flowChapters.length === 0) return chapter?.chapter_index ?? 0
    const idx = currentChapterFlowIndexRef.current
    return flowChapters[idx]?.chapter_index ?? chapter?.chapter_index ?? 0
  }, [flowChapters, chapter])

  // ── Export TwoPageNav to parent ────────────────────────────

  const nav: TwoPageNav = useMemo(() => ({
    findSpreadByParagraph,
    goToSpread, recalcSpreads,
    spreadIndex, spreadCount: totalSpreads,
    currentChapterIndex: visibleChapterIndex,
    innerRef,
  }), [findSpreadByParagraph, goToSpread, recalcSpreads, spreadIndex, totalSpreads, visibleChapterIndex, innerRef])

  useEffect(() => {
    twoPageNavRef.current = nav
  }, [nav, twoPageNavRef])

  // ── Chapter reset ──────────────────────────────────────────

  useEffect(() => {
    console.log(`[chapterReset] ch=${chapter?.chapter_index} totalSpreads=${totalSpreads}`)
    totalSpreadsRef.current = totalSpreads
    currentChapterFlowIndexRef.current = 0
    needsNextSpreadRef.current = false
    setExtraChapters([])
    setSpreadIndex(0)
    needsResetRef.current = true
    // eslint-disable-next-line react-hooks/exhaustive-deps -- only reset on chapter change, not when extra chapters are added
  }, [chapter?.chapter_index])

  // ── Clamp spread when total shrinks + advance after loadNext ─

  useEffect(() => {
    totalSpreadsRef.current = totalSpreads
    console.log(`[clampEffect] totalSpreads=${totalSpreads} needsReset=${needsResetRef.current} needsNext=${needsNextSpreadRef.current} starts=[${chapterSpreadStarts}]`)
    if (clampRafRef.current != null) cancelAnimationFrame(clampRafRef.current)
    clampRafRef.current = requestAnimationFrame(() => {
      // Use refs (not closure) to avoid stale values when multiple RAFs queue up
      const freshTotal = totalSpreadsRef.current
      if (needsResetRef.current) {
        needsResetRef.current = false
        setSpreadIndex(0)
      } else if (needsNextSpreadRef.current) {
        needsNextSpreadRef.current = false
        const prevSpread = spreadIndexRef.current
        const newSpread = Math.min(prevSpread + 1, freshTotal - 1)
        const flowIdx = findFlowIndexForSpread(newSpread)
        currentChapterFlowIndexRef.current = flowIdx
        console.log(`[clampEffect] ADVANCE ${prevSpread} → ${newSpread} freshTotal=${freshTotal} flowIdx=${flowIdx}`)
        setSpreadIndex(newSpread)
        saveProgressForSpread(newSpread)
      } else {
        setSpreadIndex(s => {
          const clamped = findNearestFilledSpread(s < freshTotal ? s : Math.max(0, freshTotal - 1), -1)
          if (clamped !== s) console.log(`[clampEffect] CLAMPED ${s} → ${clamped}`)
          return clamped
        })
        // Do NOT update currentChapterFlowIndexRef here — chapter stays the same
      }
    })
  }, [findNearestFilledSpread, totalSpreads, totalSpreadsRef, findFlowIndexForSpread, saveProgressForSpread])

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
      if (spread != null) {
        currentChapterFlowIndexRef.current = findFlowIndexForSpread(spread)
        goToSpread(spread)
      } else {
        setSpreadIndex(0)
      }
    })
  }, [initialParagraphIndex, totalSpreads, findSpreadByParagraph, goToSpread, findFlowIndexForSpread])

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
