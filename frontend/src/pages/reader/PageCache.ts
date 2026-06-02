import type { ReaderAnchor } from '../../services/api'
import type { PaginatedPage } from './PaginationEngine'
import { anchorKey } from '../../services/api'

/**
 * Incremental page cache keyed by startAnchor.
 * Enables fast forward/backward navigation — only paginates new anchors.
 * Cleared on resize/font change.
 */
export class PageCache {
  private store = new Map<string, PaginatedPage>()

  get(fromAnchor: ReaderAnchor): PaginatedPage | undefined {
    return this.store.get(anchorKey(fromAnchor))
  }

  set(page: PaginatedPage): void {
    if (page.startAnchor) {
      this.store.set(anchorKey(page.startAnchor), page)
    }
  }

  invalidate(): void {
    this.store.clear()
  }

  get size(): number {
    return this.store.size
  }
}
