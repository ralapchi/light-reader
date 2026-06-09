import { useCallback, useState } from 'react'
import { libraryCover } from '../../services/api'
import type { LibraryBookCardDto } from '../../services/api'

const MAX_CONCURRENT = 4

function keepCoversForBooks(
  prev: Record<string, string>,
  items: LibraryBookCardDto[],
): Record<string, string> {
  const validIds = new Set(items.map(i => i.book_id))
  const next: Record<string, string> = {}
  for (const [id, uri] of Object.entries(prev)) {
    if (validIds.has(id)) next[id] = uri
  }
  return next
}

export function useCoverLoader() {
  const [coverImages, setCoverImages] = useState<Record<string, string>>({})

  const loadCovers = useCallback(async (items: LibraryBookCardDto[]) => {
    const covers: Record<string, string> = {}
    // Process in batches to limit concurrent IPC requests
    for (let i = 0; i < items.length; i += MAX_CONCURRENT) {
      const batch = items.slice(i, i + MAX_CONCURRENT)
      const results = await Promise.allSettled(
        batch.map(async (item) => {
          const uri = await libraryCover(item.book_id)
          return { bookId: item.book_id, uri }
        })
      )
      for (const r of results) {
        if (r.status === 'fulfilled' && r.value.uri) {
          covers[r.value.bookId] = r.value.uri
        }
      }
    }
    setCoverImages(prev => ({ ...prev, ...covers }))
  }, [])

  const pruneCovers = useCallback((items: LibraryBookCardDto[]) => {
    setCoverImages(prev => keepCoversForBooks(prev, items))
  }, [])

  return { coverImages, loadCovers, pruneCovers }
}
