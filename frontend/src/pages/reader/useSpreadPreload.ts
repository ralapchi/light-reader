import { useEffect } from 'react'
import { afterNextPaint } from './rafUtils'

/**
 * Auto-preloads the next chapter when the reader is within 2 spreads
 * of the end of loaded content.
 */
export function useSpreadPreload(
  spreadIndex: number,
  totalSpreads: number,
  hasNextChapter: boolean,
  loadNextChapter: () => Promise<boolean>,
) {
  useEffect(() => {
    if (totalSpreads - spreadIndex > 2 || !hasNextChapter) return
    const cancel = afterNextPaint(() => { loadNextChapter() })
    return cancel
  }, [hasNextChapter, loadNextChapter, spreadIndex, totalSpreads])
}
