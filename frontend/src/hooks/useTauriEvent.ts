import { useEffect, useRef } from 'react'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export interface BookOpeningProgress {
  book_id: string
  stage: string
  progress_text: string | null
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
