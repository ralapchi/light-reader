import type { CSSProperties } from 'react'
import type { ReaderBlockDto, ReaderAnchor } from '../../services/api'
import { measureBlockHeight } from './readerUtils'
import { splitBlockAtHeight, measureFragmentHeight } from './RangeSplitter'

/** A page produced by the pagination engine. */
export interface PaginatedPage {
  blocks: ReaderBlockDto[]
  startAnchor: ReaderAnchor | null
  endAnchor: ReaderAnchor | null
}

interface PaginationConfig {
  blocks: ReaderBlockDto[]
  chapterIndex: number
  usableHeight: number
  textLayerWidth: number
  contentStyle: CSSProperties
  paragraphStyle: CSSProperties
}

/** Map a block to its anchor at a given character offset. */
function blockAnchor(block: ReaderBlockDto, chapterIndex: number, charOffset: number): ReaderAnchor {
  return { chapterId: `ch-${chapterIndex}`, blockId: block.block_id, charOffset }
}

/**
 * DOM-measurement-based pagination engine.
 *
 * Iterates blocks, measures each one via HiddenMeasureLayer, and fills pages
 * to fit within `usableHeight`. Emits a `PaginatedPage` per full page with
 * start/end anchors for cache keys.
 */
export function paginateBlocks(config: PaginationConfig): PaginatedPage[] {
  const { blocks, chapterIndex, usableHeight, textLayerWidth, contentStyle, paragraphStyle } = config
  const pages: PaginatedPage[] = []
  let pageBlocks: ReaderBlockDto[] = []
  let pageHeight = 0
  let pageStartIdx = 0

  function commitPage() {
    const startBlock = blocks[pageStartIdx] ?? pageBlocks[0]
    const endBlock = pageBlocks[pageBlocks.length - 1]
    pages.push({
      blocks: pageBlocks,
      startAnchor: startBlock ? blockAnchor(startBlock, chapterIndex, 0) : null,
      endAnchor: endBlock ? blockAnchor(endBlock, chapterIndex, 99999) : null,
    })
  }

  for (let i = 0; i < blocks.length; i++) {
    const block = blocks[i]

    // Safety factor (1.06) accounts for font rendering differences
    // (kerning, subpixel AA, antialiasing) between measurement container and real .tx
    const blockH = Math.ceil(measureBlockHeight(block, contentStyle, paragraphStyle, textLayerWidth) * 1.06)

    // Heading: page-break-before if current page already has content
    if (block.type === 'heading' && pageBlocks.length > 0) {
      commitPage()
      pageBlocks = []
      pageHeight = 0
      pageStartIdx = i
    }

    // Image > 60% of usableHeight: page-break-before
    if (block.type === 'image' && blockH > usableHeight * 0.6 && pageBlocks.length > 0) {
      commitPage()
      pageBlocks = []
      pageHeight = 0
      pageStartIdx = i
    }

    // Block fits on current page
    if (pageHeight + blockH <= usableHeight) {
      pageBlocks.push(block)
      pageHeight += blockH
      continue
    }

    // Non-splittable block: push to next page
    if (block.type === 'image' || block.type === 'separator') {
      commitPage()
      pageBlocks = [block]
      pageHeight = blockH
      pageStartIdx = i
      continue
    }

    // Text block that needs splitting — use DOM Range-based measurement
    const remaining = usableHeight - pageHeight
    const [first, second] = splitBlockAtHeight(block, remaining, contentStyle, paragraphStyle, textLayerWidth)

    if (first && 'text' in first && first.text) {
      pageBlocks.push(first)
    }
    commitPage()

    const firstText = (first as { text?: string } | null)?.text ?? ''
    if (second && 'text' in second && second.text && second.text !== firstText) {
      pageBlocks = [second]
      pageHeight = measureFragmentHeight(second, contentStyle, paragraphStyle, textLayerWidth)
      pageStartIdx = i
    } else {
      pageBlocks = []
      pageHeight = 0
      pageStartIdx = i + 1
    }
  }

  if (pageBlocks.length > 0) {
    commitPage()
  }

  return pages.length > 0 ? pages : [{ blocks: [], startAnchor: null, endAnchor: null }]
}
