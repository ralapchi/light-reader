import { useEffect, useRef } from 'react'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

// ── Event payload types (mirrors Rust events.rs) ──────────────

export interface BookOpeningStarted {
  book_id: string
  title: string
  author: string | null
}

export interface BookOpeningProgress {
  book_id: string
  stage: string
  progress_text: string | null
}

export interface BookOpeningFinished {
  book_id: string
  chapter_count: number
  load_duration_ms: number
}

export interface BookOpeningFailed {
  book_id: string | null
  error_code: string
  error_message: string
  recoverable: boolean
}

/**
 * Subscribe to a Tauri event.
 *
 * Wraps @tauri-apps/api `listen()` with automatic cleanup on unmount.
 * The handler receives the typed event payload.
 *
 * @example
 * ```tsx
 * useTauriEvent<BookOpeningFinished>('book-opening-finished', (payload) => {
 *   console.log('Book loaded:', payload.chapter_count)
 * })
 * ```
 */
export function useTauriEvent<T = unknown>(
  event: string,
  handler: (payload: T) => void,
) {
  const handlerRef = useRef(handler)

  useEffect(() => {
    handlerRef.current = handler
  }, [handler])

  useEffect(() => {
    let unlisten: UnlistenFn | undefined
    let active = true

    listen<T>(event, (e) => {
      handlerRef.current(e.payload)
    }).then((fn) => {
      if (active) {
        unlisten = fn
      } else {
        fn()
      }
    })

    return () => {
      active = false
      unlisten?.()
    }
  }, [event])
}
