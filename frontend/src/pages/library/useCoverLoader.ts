import { useCallback, useState } from 'react'
import { libraryCover } from '../../services/api'
import type { LibraryBookCardDto } from '../../services/api'

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
    const results = await Promise.allSettled(
      items.map(async (item) => {
        const uri = await libraryCover(item.book_id)
        return { bookId: item.book_id, uri }
      })
    )
    const covers: Record<string, string> = {}
    for (const r of results) {
      if (r.status === 'fulfilled' && r.value.uri) {
        covers[r.value.bookId] = r.value.uri
      }
    }
    setCoverImages(prev => ({ ...prev, ...covers }))
  }, [])

  const pruneCovers = useCallback((items: LibraryBookCardDto[]) => {
    setCoverImages(prev => keepCoversForBooks(prev, items))
  }, [])

  return { coverImages, loadCovers, pruneCovers }
}
