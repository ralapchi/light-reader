import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { readerGetChapter } from '../../services/api'
import type { ReaderChapterDto } from '../../services/api'

export function useAdjacentChapterPreload(
  chapter: ReaderChapterDto | null,
  chapterCount: number,
) {
  const [extraChapters, setExtraChapters] = useState<ReaderChapterDto[]>([])
  const loadingChapterIndexRef = useRef<number | null>(null)

  const flowChapters = useMemo(
    () => chapter ? [chapter, ...extraChapters] : [],
    [chapter, extraChapters],
  )

  const flowChaptersRef = useRef(flowChapters)
  useEffect(() => { flowChaptersRef.current = flowChapters }, [flowChapters])

  const loadNextChapter = useCallback(async (): Promise<boolean> => {
    const chapters = flowChaptersRef.current
    const lastLoaded = chapters[chapters.length - 1]
    if (!lastLoaded) return false
    const nextIndex = lastLoaded.chapter_index + 1
    if (nextIndex >= chapterCount || loadingChapterIndexRef.current === nextIndex) return false
    loadingChapterIndexRef.current = nextIndex
    try {
      const nextChapter = await readerGetChapter(nextIndex)
      setExtraChapters(c => c.some(x => x.chapter_index === nextIndex) ? c : [...c, nextChapter])
      return true
    } finally { loadingChapterIndexRef.current = null }
  }, [chapterCount])

  const hasNextChapter = useMemo(() => {
    const lastLoaded = flowChapters[flowChapters.length - 1]
    return !!lastLoaded && lastLoaded.chapter_index < chapterCount - 1
  }, [chapterCount, flowChapters])

  return {
    flowChapters,
    flowChaptersRef,
    loadNextChapter,
    hasNextChapter,
    extraChapters,
    setExtraChapters,
  }
}
