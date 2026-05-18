import { FONT_SIZE_RANGE, LINE_HEIGHT_RANGE, PARAGRAPH_SPACING_RANGE, READER_FONTS, READER_THEMES } from '../../utils/readerOptions'
import type { ReaderSettings } from '../../services/api'

export type SettingsPanel = 'theme' | 'font' | 'format' | null

interface ReaderSettingsControlsProps {
  activePanel: SettingsPanel
  onPanelChange: (panel: SettingsPanel) => void
  onUpdateSettings: (partial: Partial<ReaderSettings>) => void
  settings: ReaderSettings
}

function nextPanel(current: SettingsPanel, panel: NonNullable<SettingsPanel>): SettingsPanel {
  return current === panel ? null : panel
}

export default function ReaderSettingsControls({
  activePanel,
  onPanelChange,
  onUpdateSettings,
  settings,
}: ReaderSettingsControlsProps) {
  return (
    <>
      <div className="demo-controls">
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
              </div>
            )}
          </div>
        )}
        <div className="demo-btn-row">
          <button
            className={`demo-btn ${activePanel === 'theme' ? 'active' : ''}`}
            onClick={() => onPanelChange(nextPanel(activePanel, 'theme'))}
          >
            主题
          </button>
          <button
            className={`demo-btn ${activePanel === 'font' ? 'active' : ''}`}
            onClick={() => onPanelChange(nextPanel(activePanel, 'font'))}
          >
            字体
          </button>
          <button
            className={`demo-btn ${activePanel === 'format' ? 'active' : ''}`}
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
