import { useCallback, useState } from 'react'
import { readerChapterImage } from '../../services/api'
import type { ReaderBlockDto } from '../../services/api'

export function useChapterImages(bookId: string | undefined) {
  const [imageCache, setImageCache] = useState<Record<string, string>>({})

  const loadChapterImages = useCallback(async (blocks: ReaderBlockDto[]) => {
    const imageBlocks = blocks.filter(b => b.type === 'image')
    if (imageBlocks.length === 0) return
    const updates: Record<string, string> = {}
    await Promise.allSettled(
      imageBlocks.map(async (b) => {
        if (b.type !== 'image') return
        if (imageCache[b.asset_id]) return
        try {
          const dataUri = await readerChapterImage(bookId!, b.asset_id)
          if (dataUri) updates[b.asset_id] = dataUri
        } catch { /* skip */ }
      })
    )
    if (Object.keys(updates).length > 0) {
      setImageCache(prev => ({ ...prev, ...updates }))
    }
  }, [imageCache, bookId])

  return { imageCache, loadChapterImages }
}
