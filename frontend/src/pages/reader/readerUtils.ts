import type { TocItemDto, ReaderBlockDto } from '../../services/api'

export function flattenToc(items: TocItemDto[]): TocItemDto[] {
  const out: TocItemDto[] = []
  for (const item of items) {
    out.push(item)
    if (item.children.length > 0) out.push(...flattenToc(item.children))
  }
  return out
}

export function blockKey(block: ReaderBlockDto, fallbackIndex: number): string {
  if (block.type === 'separator') return `sep-${fallbackIndex}`
  return `${block.type}-${block.index}`
}

export function blockParagraphIndex(block: ReaderBlockDto): number | null {
  return block.type === 'paragraph' ? block.index : null
}

export function scrollToOffset(el: HTMLElement, offset: number) {
  const prev = el.style.scrollBehavior
  el.style.scrollBehavior = 'auto'
  el.scrollTop = offset
  el.style.scrollBehavior = prev
}

export function scrollToParagraph(container: HTMLElement, paraIndex: number) {
  const paras = container.querySelectorAll('.reader-paragraph')
  const target = paras[paraIndex]
  if (target) (target as HTMLElement).scrollIntoView({ behavior: 'smooth', block: 'center' })
}

export function findVisibleParagraphIndex(container: HTMLElement): number | null {
  const scrollTop = container.scrollTop
  const blocks = container.querySelectorAll('.reader-paragraph, .reader-image')
  for (let i = blocks.length - 1; i >= 0; i--) {
    if ((blocks[i] as HTMLElement).offsetTop <= scrollTop + 100) return i
  }
  return null
}
