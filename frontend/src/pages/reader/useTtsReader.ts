import { useCallback, useEffect, useRef } from 'react'
import {
  onTtsBuffering,
  onTtsError,
  onTtsFinished,
  onTtsPaused,
  onTtsPlaying,
  onTtsStopped,
  ttsPause,
  ttsResume,
  ttsStart,
  ttsStop,
} from '../../services/api'
import useAppStore from '../../store/useAppStore'
import { scrollToParagraph } from './readerUtils'

export function useTtsReader(contentRef: React.RefObject<HTMLDivElement | null>) {
  const setTtsState = useAppStore(s => s.setTtsState)
  const resetTts = useAppStore(s => s.resetTts)
  const tts = useAppStore(s => s.reader.tts)
  const currentChapterIndex = useAppStore(s => s.reader.currentChapterIndex)

  const handleTtsToggle = useCallback(async () => {
    if (tts.status === 'playing') {
      await ttsPause()
    } else if (tts.status === 'paused') {
      await ttsResume()
    } else {
      await ttsStart(currentChapterIndex)
    }
  }, [tts.status, currentChapterIndex])

  const handleTtsStop = useCallback(async () => {
    await ttsStop()
  }, [])

  useEffect(() => {
    let cancelled = false
    const unsubs: (() => void)[] = []
    Promise.all([
      onTtsPlaying(p => setTtsState({ status: 'playing', paragraph_indices: p.paragraph_indices, segment_index: p.segment_index, total_segments: p.total_segments, error: null })),
      onTtsPaused(() => setTtsState({ status: 'paused' })),
      onTtsStopped(() => resetTts()),
      onTtsBuffering(p => setTtsState({ status: 'buffering', segment_index: p.segment_index })),
      onTtsFinished(() => setTtsState({ status: 'finished', paragraph_indices: [] })),
      onTtsError(p => setTtsState({ status: 'error', error: p.error_message })),
    ]).then(fns => {
      if (cancelled) {
        fns.forEach(u => u())
      } else {
        unsubs.push(...fns.map(u => u))
      }
    })
    return () => {
      cancelled = true
      unsubs.forEach(u => u())
    }
  }, [setTtsState, resetTts])

  const prevSegRef = useRef(-1)
  useEffect(() => {
    if (tts.status !== 'playing') { prevSegRef.current = -1; return }
    if (tts.paragraph_indices.length === 0) return
    if (tts.segment_index === prevSegRef.current) return
    prevSegRef.current = tts.segment_index
    requestAnimationFrame(() => {
      const content = contentRef.current
      if (!content) return
      scrollToParagraph(content, tts.paragraph_indices[0])
    })
  }, [tts.status, tts.paragraph_indices, tts.segment_index, contentRef])

  return { handleTtsToggle, handleTtsStop }
}
