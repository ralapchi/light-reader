interface ReaderStatusBarProps {
  chapterTitle: string
  currentChapterIndex: number
  progressDisplay: string
  tts: {
    status: string
    segment_index: number
    total_segments: number
  }
}

export default function ReaderStatusBar({
  chapterTitle,
  currentChapterIndex,
  progressDisplay,
  tts,
}: ReaderStatusBarProps) {
  return (
    <div className="status-bar">
      <span>第 {currentChapterIndex + 1} 章 · {chapterTitle}</span>
      <span>
        {tts.status !== 'idle' && (
          <span className="tts-status-badge">
            {tts.status === 'playing' ? '听书中' : tts.status === 'paused' ? '已暂停' : tts.status === 'buffering' ? '合成中' : tts.status === 'error' ? '出错' : ''}
            {tts.total_segments > 0 && ` ${tts.segment_index + 1}/${tts.total_segments}`}
          </span>
        )}
        {progressDisplay}
      </span>
    </div>
  )
}
