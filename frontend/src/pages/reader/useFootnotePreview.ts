import { useCallback, useRef, useState } from 'react'
import { readerGetLinkPreview } from '../../services/api'

interface FootnotePreviewState {
  text: string
  x: number
  y: number
  direction: 'up' | 'down'
  status: 'loading' | 'ready'
}

function tooltipPosition(target: HTMLElement): { x: number; y: number; direction: 'up' | 'down' } {
  const rect = target.getBoundingClientRect()
  const x = Math.min(window.innerWidth - 18, Math.max(18, rect.left + rect.width / 2))

  // 如果上方空间不足 200px，改为向下显示
  if (rect.top < 200) {
    return { x, y: rect.bottom + 8, direction: 'down' }
  }
  return { x, y: rect.top - 10, direction: 'up' }
}

export function useFootnotePreview(currentChapterIndex: number) {
  const [preview, setPreview] = useState<FootnotePreviewState | null>(null)
  const cacheRef = useRef<Map<string, string>>(new Map())
  const requestIdRef = useRef(0)
  const hoverTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  const showPreview = useCallback((href: string, target: HTMLElement, title?: string | null) => {
    if (hoverTimerRef.current) {
      clearTimeout(hoverTimerRef.current)
      hoverTimerRef.current = null
    }

    const pos = tooltipPosition(target)
    const cached = cacheRef.current.get(href)
    if (cached) {
      setPreview({ text: cached, ...pos, status: 'ready' })
      return
    }

    if (title?.trim()) {
      setPreview({ text: title.trim(), ...pos, status: 'ready' })
    } else {
      setPreview({ text: '加载中...', ...pos, status: 'loading' })
    }

    hoverTimerRef.current = setTimeout(() => {
      const requestId = ++requestIdRef.current
      readerGetLinkPreview(href, currentChapterIndex)
        .then(result => {
          if (!result || requestIdRef.current !== requestId) return
          const text = result.title?.trim() || result.text?.trim()
          if (!text) return
          cacheRef.current.set(href, text)
          const newPos = tooltipPosition(target)
          setPreview({ text, ...newPos, status: 'ready' })
        })
        .catch(() => {
          if (requestIdRef.current === requestId && !title?.trim()) setPreview(null)
        })
    }, 150)
  }, [currentChapterIndex])

  const hidePreview = useCallback(() => {
    if (hoverTimerRef.current) {
      clearTimeout(hoverTimerRef.current)
      hoverTimerRef.current = null
    }
    requestIdRef.current += 1
    setPreview(null)
  }, [])

  return { preview, showPreview, hidePreview }
}
