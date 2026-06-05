import { useCallback, useEffect, useRef, useState } from 'react'
import { readerChapterImage } from '../../services/api'
import type { ReaderBlockDto } from '../../services/api'

const MAX_CONCURRENT = 4
type ImageState = 'loading' | 'loaded' | 'failed'

export function useChapterImages(bookId: string | undefined) {
  const [imageCache, setImageCache] = useState<Record<string, string>>({})
  const imageStateRef = useRef<Map<string, ImageState>>(new Map())
  const mountedRef = useRef(true)

  useEffect(() => {
    mountedRef.current = true
    return () => { mountedRef.current = false }
  }, [])

  useEffect(() => {
    setImageCache({})
    imageStateRef.current.clear()
  }, [bookId])

  const loadChapterImages = useCallback(async (blocks: ReaderBlockDto[]) => {
    if (!bookId) return
    const imageBlocks = blocks.filter(b => b.type === 'image')
    if (imageBlocks.length === 0) return

    const pending = imageBlocks.filter(b => {
      const state = imageStateRef.current.get(b.asset_id)
      return state !== 'loading' && state !== 'loaded'
    })
    if (pending.length === 0) return

    let active = 0
    let index = 0
    const updates: Record<string, string> = {}

    await new Promise<void>(resolve => {
      const tryStart = () => {
        while (active < MAX_CONCURRENT && index < pending.length) {
          const block = pending[index++]
          if (block.type !== 'image') continue
          const assetId = block.asset_id
          imageStateRef.current.set(assetId, 'loading')
          active++
          readerChapterImage(bookId, assetId)
            .then(dataUri => {
              if (dataUri) {
                updates[assetId] = dataUri
                imageStateRef.current.set(assetId, 'loaded')
              } else {
                imageStateRef.current.set(assetId, 'failed')
              }
            })
            .catch(() => {
              imageStateRef.current.set(assetId, 'failed')
            })
            .finally(() => {
              active--
              if (index < pending.length) {
                tryStart()
              } else if (active === 0) {
                resolve()
              }
            })
        }
        if (active === 0 && index >= pending.length) resolve()
      }
      tryStart()
    })

    if (mountedRef.current && Object.keys(updates).length > 0) {
      setImageCache(prev => ({ ...prev, ...updates }))
    }
  }, [bookId])

  return { imageCache, loadChapterImages }
}
