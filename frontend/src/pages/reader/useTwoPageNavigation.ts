import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState, type RefObject } from 'react'
import { readerSaveProgress } from '../../services/api'
import type { ReaderChapterDto } from '../../services/api'
import useAppStore from '../../store/useAppStore'
import type { TwoPageNav, TwoPageVisibleChapter } from './TwoPageReaderContent'
import { afterNextPaint } from './rafUtils'
import { createReadingPosition, readingPositionToSaveProgress } from './readerProgressUtils'
import { findFlowIndexForSpread, findNearestFilledSpread, buildFilledSpreadIndexes } from './twoPageCalcUtils'

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
  onVisibleChapterChange: ((visible: TwoPageVisibleChapter | null) => void) | undefined,
) {
  const [spreadIndex, setSpreadIndex] = useState(0)
  const needsResetRef = useRef(false)
  const needsNextSpreadRef = useRef(false)
  const preLoadTotalSpreadsRef = useRef(0)
  const clampRafRef = useRef<number | null>(null)
  const innerRef = useRef<HTMLDivElement | null>(null)

  const [currentChapterFlowIndex, setCurrentChapterFlowIndex] = useState(0)
  const currentChapterFlowIndexRef = useRef(0)

  const spreadIndexRef = useRef(spreadIndex)
  const chapterSpreadStartsRef = useRef(chapterSpreadStarts)
  const flowChaptersRef = useRef(flowChapters)
  const chapterContentPageCountsRef = useRef(chapterContentPageCounts)
  const chapterRef = useRef(chapter)

  // Sync state to refs in a single effect to avoid intermediate renders
  useEffect(() => {
    spreadIndexRef.current = spreadIndex
    chapterSpreadStartsRef.current = chapterSpreadStarts
    flowChaptersRef.current = flowChapters
    chapterContentPageCountsRef.current = chapterContentPageCounts
    chapterRef.current = chapter
  })

  const getChapterIndexForFlowIndex = useCallback((index: number) => {
    const chapters = flowChaptersRef.current
    return chapters[index]?.chapter_index ?? chapterRef.current?.chapter_index ?? 0
  }, [])

  const syncNavVisibleChapter = useCallback((index: number) => {
    const nav = twoPageNavRef.current
    if (!nav) return
    twoPageNavRef.current = {
      ...nav,
      currentChapterIndex: getChapterIndexForFlowIndex(index),
    }
  }, [getChapterIndexForFlowIndex, twoPageNavRef])

  const setVisibleFlowIndex = useCallback((index: number) => {
    currentChapterFlowIndexRef.current = index
    syncNavVisibleChapter(index)
    setCurrentChapterFlowIndex(index)
  }, [syncNavVisibleChapter])

  const boundedSpreadIndex = Math.min(spreadIndex, totalSpreads - 1)
  const spreadStep = pageWidth * 2 + spineGap * 2

  const filledSpreadIndexes = useMemo(
    () => buildFilledSpreadIndexes(chapterSpreadStarts, chapterContentPageCounts, flowChapters.length),
    [chapterContentPageCounts, chapterSpreadStarts, flowChapters.length],
  )

  const findNearestFilledSpreadLocal = useCallback(
    (target: number, delta: number) => findNearestFilledSpread(target, delta, filledSpreadIndexes, totalSpreads),
    [filledSpreadIndexes, totalSpreads],
  )
  const activeSpreadIndex = findNearestFilledSpreadLocal(boundedSpreadIndex, 1)

  const findFlowIndexForSpreadLocal = useCallback(
    (spread: number): number => findFlowIndexForSpread(spread, chapterSpreadStarts),
    [chapterSpreadStarts],
  )

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
    setSpreadIndex(current => findNearestFilledSpreadLocal(index, index >= current ? 1 : -1))
  }, [findNearestFilledSpreadLocal])

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

  const getChapterOffsetForSpread = useCallback((spread: number, flowIdx: number) => {
    const starts = chapterSpreadStartsRef.current
    const contentPageCounts = chapterContentPageCountsRef.current
    const chapterStart = starts[flowIdx] ?? 0
    const contentSpreads = Math.max(1, Math.ceil((contentPageCounts[flowIdx] ?? 1) / 2))
    if (contentSpreads <= 1) return 0
    return Math.max(0, Math.min(1, (spread - chapterStart) / (contentSpreads - 1)))
  }, [])

  const findSpreadByChapterOffset = useCallback((chapterIndex: number, chapterOffset: number): number | null => {
    const chapters = flowChaptersRef.current
    const flowIdx = chapters.findIndex(ch => ch.chapter_index === chapterIndex)
    if (flowIdx < 0) return null
    const starts = chapterSpreadStartsRef.current
    const contentPageCounts = chapterContentPageCountsRef.current
    const chapterStart = starts[flowIdx] ?? 0
    const contentSpreads = Math.max(1, Math.ceil((contentPageCounts[flowIdx] ?? 1) / 2))
    const localSpread = Math.round(Math.max(0, Math.min(1, chapterOffset)) * Math.max(0, contentSpreads - 1))
    return chapterStart + localSpread
  }, [])

  const goToChapterOffset = useCallback((chapterIndex: number, chapterOffset: number) => {
    recalcSpreads()
    const spread = findSpreadByChapterOffset(chapterIndex, chapterOffset)
    if (spread == null) return
    const flowIdx = findFlowIndexForSpreadLocal(spread)
    setVisibleFlowIndex(flowIdx)
    goToSpread(spread)
  }, [findFlowIndexForSpreadLocal, findSpreadByChapterOffset, goToSpread, recalcSpreads, setVisibleFlowIndex])

  // Save progress — reads from refs to avoid stale closures
  const saveProgressForSpread = useCallback((spread: number) => {
    const bookData = useAppStore.getState().reader.book
    const bookIdent = bookData?.book_id
    if (!bookData || !bookIdent) return
    const starts = chapterSpreadStartsRef.current
    const flowIdx = findFlowIndexForSpread(spread, starts)
    const chapters = flowChaptersRef.current
    const visChapterIndex = chapters[flowIdx]?.chapter_index ?? chapterRef.current?.chapter_index ?? 0
    const chapterOffset = getChapterOffsetForSpread(spread, flowIdx)
    const position = createReadingPosition(bookIdent, visChapterIndex, chapterOffset, 'two-page')
    const progress = readingPositionToSaveProgress(position, bookData.chapter_count)
    useAppStore.getState().setProgressPercent(progress.progress_percent)
    readerSaveProgress(progress).catch(() => { /* non-critical */ })
  }, [getChapterOffsetForSpread])

  const turnSpread = useCallback((delta: number) => {
    onNavigate?.()
    recalcSpreads()
    const current = spreadIndexRef.current
    const total = totalSpreadsRef.current
    if (delta > 0 && current >= total - 1) {
      if (hasNextChapter) {
        needsNextSpreadRef.current = true
        preLoadTotalSpreadsRef.current = total
        loadNextChapter().then(loaded => {
          if (!loaded) needsNextSpreadRef.current = false
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
    const newSpread = findNearestFilledSpreadLocal(current + delta, delta)
    const nextFlowIndex = findFlowIndexForSpreadLocal(newSpread)
    setVisibleFlowIndex(nextFlowIndex)
    goToSpread(newSpread)
    saveProgressForSpread(newSpread)
  }, [goToSpread, recalcSpreads, onNextChapter, onPreviousChapter, onNavigate, hasNextChapter, loadNextChapter, totalSpreadsRef, spreadIndexRef, findNearestFilledSpreadLocal, findFlowIndexForSpreadLocal, saveProgressForSpread, setVisibleFlowIndex])

  const nextSpread = useCallback(() => turnSpread(1), [turnSpread])
  const prevSpread = useCallback(() => turnSpread(-1), [turnSpread])

  // ── Visible chapter index (from ref, not from spreadIndex) ──

  const visibleChapterIndex = useMemo(() => {
    if (flowChapters.length === 0) return chapter?.chapter_index ?? 0
    const idx = currentChapterFlowIndex
    return flowChapters[idx]?.chapter_index ?? chapter?.chapter_index ?? 0
  }, [flowChapters, chapter, currentChapterFlowIndex])

  const visibleChapterTitle = useMemo(() => {
    const idx = currentChapterFlowIndex
    return flowChapters[idx]?.title ?? chapter?.title ?? ''
  }, [flowChapters, chapter, currentChapterFlowIndex])

  const visibleChapterOffset = useMemo(
    () => getChapterOffsetForSpread(activeSpreadIndex, currentChapterFlowIndex),
    [activeSpreadIndex, currentChapterFlowIndex, getChapterOffsetForSpread],
  )

  useEffect(() => {
    if (flowChapters.length === 0) {
      onVisibleChapterChange?.(null)
      return
    }
    onVisibleChapterChange?.({
      chapterIndex: visibleChapterIndex,
      title: visibleChapterTitle,
    })
  }, [flowChapters.length, onVisibleChapterChange, visibleChapterIndex, visibleChapterTitle])

  // ── Export TwoPageNav to parent ────────────────────────────

  const nav: TwoPageNav = useMemo(() => ({
    findSpreadByParagraph,
    goToSpread, goToChapterOffset, recalcSpreads,
    spreadIndex, spreadCount: totalSpreads,
    currentChapterIndex: visibleChapterIndex,
    currentChapterOffset: visibleChapterOffset,
    innerRef,
  }), [findSpreadByParagraph, goToSpread, goToChapterOffset, recalcSpreads, spreadIndex, totalSpreads, visibleChapterIndex, visibleChapterOffset, innerRef])

  useEffect(() => {
    twoPageNavRef.current = nav
  }, [nav, twoPageNavRef])

  // ── Chapter reset ──────────────────────────────────────────

  useEffect(() => {
    totalSpreadsRef.current = totalSpreads
    setVisibleFlowIndex(0)
    needsNextSpreadRef.current = false
    preLoadTotalSpreadsRef.current = 0
    setExtraChapters([])
    setSpreadIndex(0)
    needsResetRef.current = true
    // eslint-disable-next-line react-hooks/exhaustive-deps -- only reset on chapter change, not when extra chapters are added
  }, [chapter?.chapter_index, setVisibleFlowIndex])

  // ── Clamp spread when total shrinks ────────────────────────

  useEffect(() => {
    totalSpreadsRef.current = totalSpreads
    if (clampRafRef.current != null) cancelAnimationFrame(clampRafRef.current)
    clampRafRef.current = requestAnimationFrame(() => {
      clampRafRef.current = null
      const freshTotal = totalSpreadsRef.current
      if (needsResetRef.current) {
        needsResetRef.current = false
        setSpreadIndex(0)
      } else {
        setSpreadIndex(s => {
          return findNearestFilledSpreadLocal(s < freshTotal ? s : Math.max(0, freshTotal - 1), -1)
        })
      }
    })
    return () => {
      if (clampRafRef.current != null) {
        cancelAnimationFrame(clampRafRef.current)
        clampRafRef.current = null
      }
    }
  }, [findNearestFilledSpreadLocal, totalSpreads, totalSpreadsRef])

  // ── Advance spread after next chapter loads ─────────────────
  // Waits until totalSpreads has actually increased (useLayoutEffect measured new columns)
  // before clearing the flag and saving progress.

  useEffect(() => {
    if (!needsNextSpreadRef.current) return
    if (totalSpreads <= preLoadTotalSpreadsRef.current) return
    needsNextSpreadRef.current = false
    const newSpread = Math.min(spreadIndexRef.current + 1, totalSpreads - 1)
    const flowIdx = findFlowIndexForSpreadLocal(newSpread)
    setVisibleFlowIndex(flowIdx)
    setSpreadIndex(newSpread)
    saveProgressForSpread(newSpread)
  }, [flowChapters, totalSpreads, findFlowIndexForSpreadLocal, saveProgressForSpread, setVisibleFlowIndex])

  // ── Auto preload adjacent chapters ─────────────────────────

  useEffect(() => {
    if (totalSpreads - spreadIndex > 2 || !hasNextChapter) return
    const cancel = afterNextPaint(() => { loadNextChapter() })
    return cancel
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
    const cancel = afterNextPaint(() => {
      const spread = findSpreadByParagraph(initialParagraphIndex)
      if (spread != null) {
        currentChapterFlowIndexRef.current = findFlowIndexForSpreadLocal(spread)
        setCurrentChapterFlowIndex(currentChapterFlowIndexRef.current)
        goToSpread(spread)
      } else {
        setSpreadIndex(0)
      }
    })
    return cancel
  }, [initialParagraphIndex, totalSpreads, findSpreadByParagraph, goToSpread, findFlowIndexForSpreadLocal])

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
