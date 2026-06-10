import { useCallback, useEffect, useRef, useState } from 'react'
import { readerGetLinkPreview } from '../../services/api'

interface FootnotePreviewState {
  text: string
  left: number
  top: number
  direction: 'up' | 'down'
  status: 'loading' | 'ready'
}

function tooltipPosition(target: HTMLElement, tooltipWidth?: number): { left: number; top: number; direction: 'up' | 'down' } {
  const rect = target.getBoundingClientRect()
  const w = tooltipWidth ?? 300
  const halfW = w / 2
  const centerX = rect.left + rect.width / 2
  // 居中对齐，但不超出左右边界（留 12px 边距）
  const left = Math.max(12, Math.min(window.innerWidth - w - 12, centerX - halfW))

  if (rect.top < 200) {
    return { left, top: rect.bottom + 8, direction: 'down' }
  }
  return { left, top: rect.top - 10, direction: 'up' }
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

  const prevChapterRef = useRef(currentChapterIndex)
  useEffect(() => {
    if (prevChapterRef.current !== currentChapterIndex) {
      prevChapterRef.current = currentChapterIndex
      if (hoverTimerRef.current) {
        clearTimeout(hoverTimerRef.current)
        hoverTimerRef.current = null
      }
      requestIdRef.current += 1
      setPreview(null)
    }
  }, [currentChapterIndex])

  return { preview, showPreview, hidePreview }
}
