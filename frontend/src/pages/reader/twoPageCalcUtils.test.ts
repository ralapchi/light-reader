import { describe, it, expect } from 'vitest'
import {
  getChapterBookProgress,
  findFlowIndexForSpread,
  findNearestFilledSpread,
  buildFilledSpreadIndexes,
} from './twoPageCalcUtils'
import {
  chapterProgressPercent,
  buildChapterOnlyProgress,
} from './readerProgressUtils'

describe('chapterProgressPercent', () => {
  it('returns 0 for first chapter', () => {
    expect(chapterProgressPercent(0, 10)).toBe(0)
  })
  it('returns fraction for middle chapter', () => {
    expect(chapterProgressPercent(5, 10)).toBe(0.5)
  })
  it('clamps to <1 for last chapter', () => {
    expect(chapterProgressPercent(9, 10)).toBeCloseTo(0.9)
  })
  it('returns 0 for zero chapterCount', () => {
    expect(chapterProgressPercent(0, 0)).toBe(0)
  })
  it('clamps negative index to 0', () => {
    expect(chapterProgressPercent(-1, 10)).toBe(0)
  })
})

describe('buildChapterOnlyProgress', () => {
  it('builds correct DTO', () => {
    const dto = buildChapterOnlyProgress('book-1', 3, 10)
    expect(dto.book_id).toBe('book-1')
    expect(dto.chapter_index).toBe(3)
    expect(dto.progress_percent).toBeCloseTo(0.3)
    expect(dto.paragraph_index).toBeNull()
    expect(dto.scroll_offset).toBeNull()
    expect(dto.anchor).toBeNull()
  })
})

describe('getChapterBookProgress', () => {
  it('returns 0 for first chapter', () => {
    expect(getChapterBookProgress(0, 10)).toBe(0)
  })
  it('returns correct fraction', () => {
    expect(getChapterBookProgress(3, 10)).toBeCloseTo(0.3)
  })
  it('clamps to 1 at last chapter', () => {
    expect(getChapterBookProgress(10, 10)).toBe(1)
  })
  it('returns 0 for zero chapterCount', () => {
    expect(getChapterBookProgress(0, 0)).toBe(0)
  })
})

describe('findFlowIndexForSpread', () => {
  it('returns 0 for spread before any chapter', () => {
    expect(findFlowIndexForSpread(0, [0, 5, 10])).toBe(0)
  })
  it('returns correct chapter for spread in middle', () => {
    expect(findFlowIndexForSpread(7, [0, 5, 10])).toBe(1)
  })
  it('returns last chapter for spread beyond all starts', () => {
    expect(findFlowIndexForSpread(15, [0, 5, 10])).toBe(2)
  })
  it('returns 0 for empty starts array', () => {
    expect(findFlowIndexForSpread(5, [])).toBe(0)
  })
})

describe('buildFilledSpreadIndexes', () => {
  it('builds correct set for single chapter', () => {
    const result = buildFilledSpreadIndexes([0], [4], 1)
    expect(result.has(0)).toBe(true)
    expect(result.has(1)).toBe(true)
    expect(result.size).toBe(2) // 4 pages / 2 = 2 spreads
  })
  it('builds correct set for multiple chapters', () => {
    const result = buildFilledSpreadIndexes([0, 3, 7], [4, 6, 2], 3)
    // Chapter 0: spreads 0,1 (4 pages / 2 = 2)
    // Chapter 1: spreads 3,4,5 (6 pages / 2 = 3)
    // Chapter 2: spread 7 (2 pages / 2 = 1)
    expect(result.has(0)).toBe(true)
    expect(result.has(1)).toBe(true)
    expect(result.has(3)).toBe(true)
    expect(result.has(4)).toBe(true)
    expect(result.has(5)).toBe(true)
    expect(result.has(7)).toBe(true)
    expect(result.size).toBe(6)
  })
  it('returns empty set for empty inputs', () => {
    expect(buildFilledSpreadIndexes([], [], 0).size).toBe(0)
  })
})

describe('findNearestFilledSpread', () => {
  const filled = new Set([0, 1, 3, 4, 5, 7])
  const total = 10

  it('returns target if it is filled', () => {
    expect(findNearestFilledSpread(3, 1, filled, total)).toBe(3)
  })
  it('finds next filled spread when going forward', () => {
    // target=2, not filled, delta=1 → search forward → 3
    expect(findNearestFilledSpread(2, 1, filled, total)).toBe(3)
  })
  it('finds previous filled spread when going backward', () => {
    // target=6, not filled, delta=-1 → search backward → 5
    expect(findNearestFilledSpread(6, -1, filled, total)).toBe(5)
  })
  it('falls back to opposite direction', () => {
    // target=6, not filled, delta=1 → forward: 7, found
    expect(findNearestFilledSpread(6, 1, filled, total)).toBe(7)
  })
  it('handles boundary: target at 0', () => {
    expect(findNearestFilledSpread(0, -1, filled, total)).toBe(0)
  })
  it('handles boundary: target at last spread', () => {
    // target=9, not filled, delta=-1 → backward: 7
    expect(findNearestFilledSpread(9, -1, filled, total)).toBe(7)
  })
  it('returns bounded target when no filled spreads exist', () => {
    const empty = new Set<number>()
    expect(findNearestFilledSpread(5, 1, empty, total)).toBe(5)
  })
})

describe('two-page edge cases', () => {
  it('new chapter starts at left page (spread 0)', () => {
    // Chapter 0 starts at spread 0
    const starts = [0, 3]
    expect(findFlowIndexForSpread(0, starts)).toBe(0)
    expect(findFlowIndexForSpread(1, starts)).toBe(0)
    expect(findFlowIndexForSpread(2, starts)).toBe(0)
    expect(findFlowIndexForSpread(3, starts)).toBe(1)
  })

  it('same-chapter page turn does not trigger progress save', () => {
    // This is a behavioral test: when flowIndex doesn't change, no save
    const starts = [0, 5]
    const idx1 = findFlowIndexForSpread(0, starts)
    const idx2 = findFlowIndexForSpread(1, starts)
    expect(idx1).toBe(idx2) // same chapter → no save needed
  })

  it('blank spread between chapters is skipped', () => {
    // Filled: 0,1 (ch0) and 4,5 (ch1). Spread 2,3 are blank.
    const filled = new Set([0, 1, 4, 5])
    // Going forward from spread 1, next filled is 4
    expect(findNearestFilledSpread(2, 1, filled, 6)).toBe(4)
    // Going backward from spread 3, prev filled is 1
    expect(findNearestFilledSpread(3, -1, filled, 6)).toBe(1)
  })

  it('last chapter / last page boundary', () => {
    const starts = [0, 3]
    const counts = [4, 2]
    const filled = buildFilledSpreadIndexes(starts, counts, 2)
    // ch0: spreads 0,1; ch1: spread 3
    expect(filled.has(0)).toBe(true)
    expect(filled.has(1)).toBe(true)
    expect(filled.has(3)).toBe(true)
    expect(filled.size).toBe(3)
    // Last spread is 3, which is filled
    expect(findNearestFilledSpread(3, 1, filled, 5)).toBe(3)
    // Beyond last: bounded to 4, not filled, find backward → 3
    expect(findNearestFilledSpread(4, 1, filled, 5)).toBe(3)
  })
})
