import { useCallback, useEffect, useRef, useState } from 'react'
import { readerChapterImages } from '../../services/api'
import type { ReaderBlockDto } from '../../services/api'

type ImageState = 'loading' | 'loaded' | 'failed'

export function useChapterImages(bookId: string | undefined) {
  const [imageCache, setImageCache] = useState<Record<string, string>>({})
  const imageStateRef = useRef<Map<string, ImageState>>(new Map())
  const mountedRef = useRef(true)
  const loadIdRef = useRef(0)

  useEffect(() => {
    mountedRef.current = true
    return () => { mountedRef.current = false }
  }, [])

  useEffect(() => {
    setImageCache({})
    imageStateRef.current.clear()
    loadIdRef.current++
  }, [bookId])

  const loadChapterImages = useCallback(async (blocks: ReaderBlockDto[]) => {
    if (!bookId) return
    const imageBlocks = blocks.filter(b => b.type === 'image')
    if (imageBlocks.length === 0) return

    const pendingIds = imageBlocks
      .filter(b => {
        const state = imageStateRef.current.get(b.asset_id)
        return state !== 'loading' && state !== 'loaded'
      })
      .map(b => b.asset_id)
    if (pendingIds.length === 0) return

    const loadId = loadIdRef.current
    for (const id of pendingIds) {
      imageStateRef.current.set(id, 'loading')
    }

    try {
      const data = await readerChapterImages(bookId, pendingIds)
      if (loadIdRef.current !== loadId) return

      for (const id of pendingIds) {
        if (data[id]) {
          imageStateRef.current.set(id, 'loaded')
        } else {
          imageStateRef.current.set(id, 'failed')
        }
      }

      if (mountedRef.current && Object.keys(data).length > 0) {
        setImageCache(prev => ({ ...prev, ...data }))
      }
    } catch {
      if (loadIdRef.current !== loadId) return
      for (const id of pendingIds) {
        imageStateRef.current.set(id, 'failed')
      }
    }
  }, [bookId])

  return { imageCache, loadChapterImages }
}
