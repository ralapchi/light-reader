import type { CSSProperties } from 'react'
import type { ReaderBlockDto, ReaderTextLinkDto } from '../../services/api'

type TextBlock = Extract<ReaderBlockDto, { text: string }>

function adjustLinks(links: ReaderTextLinkDto[] | undefined, shift: number): ReaderTextLinkDto[] | undefined {
  if (!links) return undefined
  return links
    .map(l => ({ ...l, start: l.start - shift, end: l.end - shift }))
    .filter(l => l.end > 0)
    .map(l => ({ ...l, start: Math.max(0, l.start) }))
}

function createMeasureContainer(
  contentStyle: CSSProperties,
  paragraphStyle: CSSProperties,
  textLayerWidth: number,
): { tx: HTMLDivElement; para: HTMLDivElement } {
  const tx = document.createElement('div')
  tx.className = 'tx'
  Object.assign(tx.style, contentStyle, {
    position: 'absolute',
    left: '-9999px',
    top: '0',
    width: `${textLayerWidth}px`,
    visibility: 'hidden',
  })

  const para = document.createElement('div')
  para.className = 'reader-paragraph'
  Object.assign(para.style, paragraphStyle)

  tx.appendChild(para)
  document.body.appendChild(tx)
  return { tx, para }
}

function measureRangeHeight(
  textNode: Text,
  start: number,
  end: number,
): number {
  const range = document.createRange()
  range.setStart(textNode, start)
  range.setEnd(textNode, end)
  return range.getBoundingClientRect().height
}

/**
 * Split a text block at the character position where its rendered height
 * exceeds maxHeight. Uses DOM Range binary search for precision.
 */
export function splitBlockAtHeight(
  block: ReaderBlockDto,
  maxHeight: number,
  contentStyle: CSSProperties,
  paragraphStyle: CSSProperties,
  textLayerWidth: number,
): [ReaderBlockDto | null, ReaderBlockDto | null] {
  if (block.type !== 'paragraph' && block.type !== 'heading' && block.type !== 'quote') {
    return [block, null]
  }

  const chars = Array.from((block as TextBlock).text)
  if (chars.length === 0) return [block, null]

  const { tx, para } = createMeasureContainer(contentStyle, paragraphStyle, textLayerWidth)
  para.textContent = chars.join('')
  const textNode = para.firstChild as Text
  if (!textNode) {
    document.body.removeChild(tx)
    return [block, null]
  }

  const fullHeight = measureRangeHeight(textNode, 0, chars.length)

  if (fullHeight <= maxHeight) {
    document.body.removeChild(tx)
    return [block, null]
  }

  // Binary search for the largest character count that fits
  let lo = 1
  let hi = chars.length
  while (lo < hi) {
    const mid = Math.ceil((lo + hi) / 2)
    const h = measureRangeHeight(textNode, 0, mid)
    if (h <= maxHeight) {
      lo = mid
    } else {
      hi = mid - 1
    }
  }

  document.body.removeChild(tx)

  const splitPoint = lo
  if (splitPoint <= 0 || splitPoint >= chars.length) {
    return [block, null]
  }

  const textBlock = block as TextBlock
  const first: ReaderBlockDto = {
    ...textBlock,
    text: chars.slice(0, splitPoint).join(''),
    links: adjustLinks(textBlock.links?.filter(l => l.start < splitPoint), 0),
    __fragmentIsStart: true,
    __fragmentIsEnd: false,
  } as unknown as ReaderBlockDto

  const second: ReaderBlockDto = {
    ...textBlock,
    text: chars.slice(splitPoint).join(''),
    links: adjustLinks(textBlock.links?.filter(l => l.end > splitPoint), splitPoint),
    __fragmentIsStart: false,
    __fragmentIsEnd: true,
  } as unknown as ReaderBlockDto

  return [first, second]
}

/** Measure the height of a split fragment using DOM measurement. */
export function measureFragmentHeight(
  block: ReaderBlockDto,
  contentStyle: CSSProperties,
  paragraphStyle: CSSProperties,
  textLayerWidth: number,
): number {
  const text = 'text' in block ? block.text : ''
  if (!text) return 0

  const { tx, para } = createMeasureContainer(contentStyle, paragraphStyle, textLayerWidth)
  para.textContent = text
  const height = tx.getBoundingClientRect().height
  document.body.removeChild(tx)
  return height
}
