import ReaderContent from './reader/ReaderContent'
import ReaderSearchPanel from './reader/ReaderSearchPanel'
import ReaderSettingsControls from './reader/ReaderSettingsControls'
import ReaderStatusBar from './reader/ReaderStatusBar'
import ReaderTocPanel from './reader/ReaderTocPanel'
import ReaderToolbar from './reader/ReaderToolbar'
import { useReaderPage } from './reader/useReaderPage'
import './ReaderPage.css'

function ReaderPage() {
  const page = useReaderPage()

  return (
    <div className="reader-app" style={page.readerStyle}>
      <div className="toolbar-trigger" />

      <ReaderToolbar
        book={page.book}
        chapterTitle={page.chapterTitle}
        currentBookmark={page.currentBookmark}
        onBack={page.goBackToLibrary}
        onNextChapter={page.goToNextChapter}
        onPreviousChapter={page.goToPreviousChapter}
        onToggleBookmark={page.toggleBookmark}
        onToggleSearch={page.handleToggleSearch}
        onToggleToc={page.toggleToc}
        onTtsStop={page.handleTtsStop}
        onTtsToggle={page.handleTtsToggle}
        ttsStatus={page.tts.status}
      />

      <ReaderTocPanel
        currentChapterIndex={page.currentChapterIndex}
        items={page.flatToc}
        onClose={page.closeToc}
        onGoToChapter={page.goToChapter}
        progressDisplay={page.progressDisplay}
        progressPercent={page.progressPercent}
        visible={page.showToc}
      />

      <ReaderSearchPanel
        onClose={page.handleCloseSearch}
        onInput={page.handleSearchInput}
        onResultClick={page.handleSearchResultClick}
        query={page.searchQuery}
        results={page.searchResults}
        visible={page.showSearch}
      />

      <div className="reading-progress-track">
        <div className="reading-progress-fill" style={{ width: `${Math.round(page.progressPercent * 100)}%` }} />
      </div>

      <ReaderContent
        chapter={page.currentChapter}
        contentRef={page.contentRef}
        contentStyle={page.contentStyle}
        highlightedParagraphIndex={page.tts.paragraph_indices[0]}
        imageCache={page.imageCache}
        onScroll={page.handleScroll}
        paragraphStyle={page.paragraphStyle}
      />

      <ReaderStatusBar
        chapterTitle={page.chapterTitle}
        currentChapterIndex={page.currentChapterIndex}
        progressDisplay={page.progressDisplay}
        tts={page.tts}
      />

      <ReaderSettingsControls
        activePanel={page.activePanel}
        onPanelChange={page.setActivePanel}
        onUpdateSettings={page.updateAndSave}
        settings={page.settings}
      />
    </div>
  )
}

export default ReaderPage
