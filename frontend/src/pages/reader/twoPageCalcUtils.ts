/** Chapter-level book progress fraction (pure). */
export function getChapterBookProgress(chapterIndex: number, chapterCount: number): number {
  if (chapterCount <= 0) return 0
  return Math.min(1, Math.max(0, chapterIndex) / chapterCount)
}

/** Find which flow chapter a given global spread belongs to (pure). */
export function findFlowIndexForSpread(spread: number, chapterSpreadStarts: number[]): number {
  let flowIndex = 0
  for (let i = 0; i < chapterSpreadStarts.length; i++) {
    if (chapterSpreadStarts[i] <= spread) flowIndex = i
    else break
  }
  return flowIndex
}

/** Build the set of spread indexes that contain actual chapter content (pure). */
export function buildFilledSpreadIndexes(
  chapterSpreadStarts: number[],
  chapterContentPageCounts: number[],
  chapterCount: number,
): Set<number> {
  const indexes = new Set<number>()
  const len = Math.min(chapterSpreadStarts.length, chapterContentPageCounts.length, chapterCount)
  for (let i = 0; i < len; i++) {
    const start = chapterSpreadStarts[i] ?? 0
    const contentSpreads = Math.max(1, Math.ceil((chapterContentPageCounts[i] ?? 1) / 2))
    for (let offset = 0; offset < contentSpreads; offset++) {
      indexes.add(start + offset)
    }
  }
  return indexes
}

/** Find the nearest spread that has actual content, searching in delta direction first (pure). */
export function findNearestFilledSpread(
  target: number,
  delta: number,
  filledSpreadIndexes: Set<number>,
  totalSpreads: number,
): number {
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
}
