import { FONT_SIZE_RANGE, LINE_HEIGHT_RANGE, PARAGRAPH_SPACING_RANGE, CONTENT_WIDTH_RANGE, READER_FONTS, READER_THEMES } from '../../utils/readerOptions'
import type { ReaderSettings } from '../../services/api'

export type SettingsPanel = 'theme' | 'font' | 'format' | null

interface ReaderSettingsControlsProps {
  activePanel: SettingsPanel
  isTwoPageAvailable?: boolean
  onPanelChange: (panel: SettingsPanel) => void
  onUpdateSettings: (partial: Partial<ReaderSettings>) => void
  settings: ReaderSettings
}

function nextPanel(current: SettingsPanel, panel: NonNullable<SettingsPanel>): SettingsPanel {
  return current === panel ? null : panel
}

export default function ReaderSettingsControls({
  activePanel,
  isTwoPageAvailable = true,
  onPanelChange,
  onUpdateSettings,
  settings,
}: ReaderSettingsControlsProps) {
  return (
    <>
      <div className="settings-floating-panel">
        {activePanel && (
          <div className="settings-panel" onClick={(e) => e.stopPropagation()}>
            {activePanel === 'theme' && (
              <div className="theme-options">
                {READER_THEMES.map(t => (
                  <button
                    key={t.id}
                    className={`theme-swatch ${settings.theme === t.id ? 'active' : ''}`}
                    onClick={() => onUpdateSettings({ theme: t.id })}
                    title={t.label}
                  >
                    <span className="swatch-color" style={{ background: t.bg, boxShadow: `inset 0 0 0 1px ${t.border}` }} />
                    <span className="swatch-label">{t.label}</span>
                  </button>
                ))}
              </div>
            )}
            {activePanel === 'font' && (
              <div className="font-options">
                <div className="option-row">
                  <span className="option-label">字体</span>
                  <div className="option-group">
                    <select
                      className="option-select"
                      value={settings.font_family}
                      onChange={e => onUpdateSettings({ font_family: e.target.value })}
                    >
                      {READER_FONTS.map(f => (
                        <option key={f.id} value={f.id}>{f.label}</option>
                      ))}
                    </select>
                  </div>
                </div>
                <div className="option-row">
                  <span className="option-label">字号</span>
                  <input
                    type="range" min={FONT_SIZE_RANGE.min} max={FONT_SIZE_RANGE.max} step={FONT_SIZE_RANGE.step}
                    value={settings.font_size}
                    onChange={e => onUpdateSettings({ font_size: Number(e.target.value) })}
                    className="option-slider"
                  />
                </div>
              </div>
            )}
            {activePanel === 'format' && (
              <div className="format-options">
                <div className="option-row">
                  <span className="option-label">布局</span>
                  <div className="layout-toggles">
                    <button
                      className={`layout-btn ${settings.reading_mode === 'ChapterScroll' ? 'active' : ''}`}
                      onClick={() => onUpdateSettings({ reading_mode: 'ChapterScroll' })}
                      title="单页滚动"
                    >
                      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                        <rect x="4" y="3" width="16" height="18" rx="2" />
                        <line x1="8" y1="8" x2="16" y2="8" />
                        <line x1="8" y1="12" x2="16" y2="12" />
                        <line x1="8" y1="16" x2="12" y2="16" />
                      </svg>
                    </button>
                    <button
                      className={`layout-btn ${settings.reading_mode === 'TwoPage' ? 'active' : ''}`}
                      disabled={!isTwoPageAvailable}
                      onClick={() => onUpdateSettings({ reading_mode: 'TwoPage' })}
                      title="双页阅读"
                    >
                      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                        <path d="M3 4.5h8.5v15H4a1 1 0 0 1-1-1V5.5a1 1 0 0 1 1-1z" />
                        <path d="M21 4.5h-8.5v15H20a1 1 0 0 0 1-1V5.5a1 1 0 0 0-1-1z" />
                        <line x1="12" y1="4.5" x2="12" y2="19.5" />
                      </svg>
                    </button>
                  </div>
                </div>
                <div className="option-row">
                  <span className="option-label">行距</span>
                  <input
                    type="range" min={LINE_HEIGHT_RANGE.min} max={LINE_HEIGHT_RANGE.max} step={LINE_HEIGHT_RANGE.step}
                    value={settings.line_height}
                    onChange={e => onUpdateSettings({ line_height: Number(e.target.value) })}
                    className="option-slider"
                  />
                </div>
                <div className="option-row">
                  <span className="option-label">段间距</span>
                  <input
                    type="range" min={PARAGRAPH_SPACING_RANGE.min} max={PARAGRAPH_SPACING_RANGE.max} step={PARAGRAPH_SPACING_RANGE.step}
                    value={settings.paragraph_spacing}
                    onChange={e => onUpdateSettings({ paragraph_spacing: Number(e.target.value) })}
                    className="option-slider"
                  />
                </div>
                {settings.reading_mode === 'ChapterScroll' && (
                  <div className="option-row">
                    <span className="option-label">页宽</span>
                    <input
                      type="range" min={CONTENT_WIDTH_RANGE.min} max={CONTENT_WIDTH_RANGE.max} step={CONTENT_WIDTH_RANGE.step}
                      value={settings.content_width}
                      onChange={e => onUpdateSettings({ content_width: Number(e.target.value) })}
                      className="option-slider"
                    />
                  </div>
                )}
              </div>
            )}
          </div>
        )}
        <div className="settings-floating-btn-row">
          <button
            className={`settings-floating-btn ${activePanel === 'theme' ? 'active' : ''}`}
            onClick={() => onPanelChange(nextPanel(activePanel, 'theme'))}
          >
            主题
          </button>
          <button
            className={`settings-floating-btn ${activePanel === 'font' ? 'active' : ''}`}
            onClick={() => onPanelChange(nextPanel(activePanel, 'font'))}
          >
            字体
          </button>
          <button
            className={`settings-floating-btn ${activePanel === 'format' ? 'active' : ''}`}
            onClick={() => onPanelChange(nextPanel(activePanel, 'format'))}
          >
            格式
          </button>
        </div>
      </div>

      {activePanel && (
        <div className="panel-backdrop" onClick={() => onPanelChange(null)} />
      )}
    </>
  )
}
