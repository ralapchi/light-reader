import type { TocItemDto, ReaderBlockDto, ReaderAnchor } from '../../services/api'
import type { TwoPageNav } from './TwoPageReaderContent'

export const INLINE_IMAGE_RE = /\u{E000}(.+?)\u{E001}/gu

export function flattenToc(items: TocItemDto[]): TocItemDto[] {
  const out: TocItemDto[] = []
  for (const item of items) {
    out.push(item)
    if (item.children.length > 0) out.push(...flattenToc(item.children))
  }
  return out
}

export function blockKey(block: ReaderBlockDto, fallbackIndex: number): string {
  if ('block_id' in block && block.block_id) return block.block_id
  if (block.type === 'separator') return `sep-${fallbackIndex}`
  return `${block.type}-${block.index}`
}

export function blockParagraphIndex(block: ReaderBlockDto): number | null {
  if (block.type === 'paragraph' || block.type === 'heading' || block.type === 'quote') return block.index
  return null
}

export function scrollToOffset(el: HTMLElement, offset: number) {
  const prev = el.style.scrollBehavior
  el.style.scrollBehavior = 'auto'
  el.scrollTop = offset
  el.style.scrollBehavior = prev
}

export function scrollToProgressOffset(el: HTMLElement, offset: number) {
  const maxScroll = Math.max(0, el.scrollHeight - el.clientHeight)
  scrollToOffset(el, Math.max(0, Math.min(1, offset)) * maxScroll)
}

export function scrollToParagraph(container: HTMLElement, paraIndex: number) {
  const paras = container.querySelectorAll('.reader-paragraph')
  const target = paras[paraIndex]
  if (target) (target as HTMLElement).scrollIntoView({ behavior: 'smooth', block: 'center' })
}

export function findVisibleParagraphIndex(container: HTMLElement): number | null {
  const scrollTop = container.scrollTop
  const paras = container.querySelectorAll('.reader-paragraph')
  for (let i = paras.length - 1; i >= 0; i--) {
    if ((paras[i] as HTMLElement).offsetTop <= scrollTop + 100) return i
  }
  return null
}

/** Two-page mode: find first paragraph visible in the current spread using viewport coordinates. */
export function findVisibleParagraphTwoPage(container: HTMLElement): number | null {
  const paras = container.querySelectorAll('.reader-paragraph[data-para-index]')
  const viewport = container.getBoundingClientRect()
  for (const p of paras) {
    const r = (p as HTMLElement).getBoundingClientRect()
    if (
      r.top < viewport.bottom &&
      r.bottom > viewport.top &&
      r.left > viewport.left - 100 &&
      r.left < viewport.right + 100
    ) {
      const idx = (p as HTMLElement).dataset.paraIndex
      if (idx != null) return Number(idx)
    }
  }
  return null
}

/** Two-page mode: navigate to the spread containing the paragraph. */
export function scrollToParagraphTwoPage(_container: HTMLElement, paraIndex: number, nav?: TwoPageNav | null) {
  if (nav) {
    nav.recalcSpreads()
    const spread = nav.findSpreadByParagraph(paraIndex)
    if (spread != null) nav.goToSpread(spread)
  }
}

/** Record the currently visible paragraph before a layout change (e.g. resize). */
export function captureVisibleParagraph(container: HTMLElement, twoPage: boolean): number | null {
  return twoPage ? findVisibleParagraphTwoPage(container) : findVisibleParagraphIndex(container)
}

// ── ReaderAnchor ──────────────────────────────────────────────

/** Binary-search through a paragraph's text nodes to find the first character position whose bounding rect is at or below the viewport top. */
function findFirstVisibleCharInParagraph(container: HTMLElement, paraIndex: number): number {
  const paras = container.querySelectorAll('.reader-paragraph')
  const el = paras[paraIndex] as HTMLElement | undefined
  if (!el) return 0

  const viewportTop = container.getBoundingClientRect().top
  const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT)
  const range = document.createRange()
  let globalOffset = 0
  let node = walker.nextNode() as Text | null

  while (node) {
    const text = node.textContent ?? ''
    let lo = 0
    let hi = text.length
    while (lo < hi) {
      const mid = Math.floor((lo + hi) / 2)
      range.setStart(node, mid)
      range.setEnd(node, mid + 1)
      const rect = range.getBoundingClientRect()
      if (rect.height === 0) { lo = mid + 1; continue }
      if (rect.bottom <= viewportTop) {
        lo = mid + 1
      } else {
        hi = mid
      }
    }
    if (lo < text.length) {
      return globalOffset + lo
    }
    globalOffset += text.length
    node = walker.nextNode() as Text | null
  }

  return 0
}

/** Capture a ReaderAnchor for the current scroll position in single-page mode. */
export function captureReaderAnchor(container: HTMLElement, chapterIndex: number): ReaderAnchor | null {
  const paraIndex = findVisibleParagraphIndex(container)
  if (paraIndex == null) return null
  return {
    chapterId: `ch-${chapterIndex}`,
    blockId: `p-${paraIndex}`,
    charOffset: findFirstVisibleCharInParagraph(container, paraIndex),
  }
}

/** Scroll to the position described by a ReaderAnchor in single-page mode. */
export function scrollToAnchor(container: HTMLElement, anchor: ReaderAnchor): void {
  const m = anchor.blockId.match(/^p-(\d+)$/)
  if (!m) return
  const paraIndex = parseInt(m[1], 10)
  const paras = container.querySelectorAll('.reader-paragraph')
  const el = paras[paraIndex] as HTMLElement | undefined
  if (!el) return

  const prev = container.style.scrollBehavior
  container.style.scrollBehavior = 'auto'

  el.scrollIntoView({ block: 'start' })

  if (anchor.charOffset > 0) {
    const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT)
    let runOffset = 0
    let node = walker.nextNode() as Text | null
    while (node) {
      const len = node.textContent?.length ?? 0
      if (runOffset + len >= anchor.charOffset) {
        const range = document.createRange()
        range.setStart(node, Math.min(anchor.charOffset - runOffset, len))
        range.collapse(true)
        const rect = range.getBoundingClientRect()
        if (rect.height > 0) {
          container.scrollTop += rect.top - container.getBoundingClientRect().top
        }
        break
      }
      runOffset += len
      node = walker.nextNode() as Text | null
    }
  }

  container.style.scrollBehavior = prev
}
