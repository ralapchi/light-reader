import { useCallback, useRef } from 'react'
import { libraryCover } from '../../services/api'
import type { LibraryBookCardDto } from '../../services/api'
import useAppStore from '../../store/useAppStore'

const MAX_CONCURRENT = 4

export function useCoverLoader() {
  const coverImages = useAppStore(s => s.coverImages)
  const setCoverImage = useAppStore(s => s.setCoverImage)
  const pruneCoverImages = useAppStore(s => s.pruneCoverImages)
  const loadIdRef = useRef(0)

  const loadCovers = useCallback(async (items: LibraryBookCardDto[]) => {
    const loadId = ++loadIdRef.current
    for (let i = 0; i < items.length; i += MAX_CONCURRENT) {
      if (loadIdRef.current !== loadId) return
      const batch = items.slice(i, i + MAX_CONCURRENT)
      const results = await Promise.allSettled(
        batch.map(async (item) => {
          const uri = await libraryCover(item.book_id)
          return { bookId: item.book_id, uri }
        })
      )
      for (const r of results) {
        if (r.status === 'fulfilled' && r.value.uri) {
          setCoverImage(r.value.bookId, r.value.uri)
        }
      }
    }
  }, [setCoverImage])

  const pruneCovers = useCallback((items: LibraryBookCardDto[]) => {
    pruneCoverImages(new Set(items.map(i => i.book_id)))
  }, [pruneCoverImages])

  return { coverImages, loadCovers, pruneCovers }
}
