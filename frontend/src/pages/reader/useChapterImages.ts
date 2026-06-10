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

    // Block-level images
    const imageBlocks = blocks.filter(b => b.type === 'image')

    // Inline images embedded in paragraph text via PUA markers
    const inlineImageIds: string[] = []
    for (const b of blocks) {
      if ('text' in b) {
        const re = /(.+?)/g
        let m: RegExpExecArray | null
        while ((m = re.exec(b.text)) !== null) {
          inlineImageIds.push(m[1])
        }
      }
    }

    const allAssetIds = [
      ...imageBlocks.map(b => b.asset_id),
      ...inlineImageIds,
    ]
    if (allAssetIds.length === 0) return

    const pendingIds = allAssetIds.filter(id => {
      const state = imageStateRef.current.get(id)
      return state !== 'loading' && state !== 'loaded'
    })
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
