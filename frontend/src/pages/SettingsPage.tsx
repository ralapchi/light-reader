import { FONT_SIZE_RANGE, LINE_HEIGHT_RANGE, PARAGRAPH_SPACING_RANGE, READER_FONTS, READER_THEMES } from '../utils/readerOptions'
import { useSettingsPage, type SettingsSection } from './settings/useSettingsPage'
import TagGroupManager from './settings/TagGroupManager'
import './SettingsPage.css'

const SECTIONS: { id: SettingsSection; label: string; desc: string }[] = [
  { id: 'general', label: '常规', desc: '应用偏好' },
  { id: 'tts', label: '听书', desc: '朗读服务' },
  { id: 'reading', label: '阅读', desc: '主题与排版' },
  { id: 'tags', label: '标签', desc: '分组管理' },
  { id: 'about', label: '关于', desc: '应用信息' },
]

function SettingsPage() {
  const {
    activeFont,
    activeSection,
    activeTheme,
    bookCacheClearStatus,
    clearBookCache,
    clearTtsCache,
    handleTtsSave,
    previewFontFamily,
    saveApiKey,
    saveTtsPartial,
    setActiveSection,
    settings,
    testTtsConnection,
    ttsConfig,
    ttsLoaded,
    ttsTestStatus,
    updateAndSave,
    updateTtsDraft,
  } = useSettingsPage()

  return (
    <main className="settings-main">
      <div className="settings-page-header">
        <div>
          <h1>设置</h1>
        </div>
      </div>

      <div className="settings-layout">
        {/* Left nav */}
        <nav className="settings-nav" aria-label="设置分类">
          {SECTIONS.map(s => (
            <button
              key={s.id}
              type="button"
              className={`settings-nav-item ${activeSection === s.id ? 'active' : ''}`}
              onClick={() => setActiveSection(s.id)}
            >
              <span className="settings-nav-label">{s.label}</span>
              <span className="settings-nav-desc">{s.desc}</span>
            </button>
          ))}
        </nav>

        {/* Right content */}
        <div className="settings-content">

          {/* ── 外观 ── */}
          {activeSection === 'reading' && (
            <div className="settings-section">
              <div className="settings-appearance-grid">
                <div className="settings-group">
                  <div className="settings-field">
                    <div className="settings-label-row">
                      <span>主题</span>
                      <span className="settings-value">{activeTheme.label}</span>
                    </div>
                    <div className="settings-theme-grid">
                      {READER_THEMES.map(t => (
                        <button
                          key={t.id}
                          type="button"
                          className={`settings-theme-card ${settings.theme === t.id ? 'active' : ''}`}
                          onClick={() => updateAndSave({ theme: t.id })}
                          aria-pressed={settings.theme === t.id}
                        >
                          <span className="theme-preview" style={{ background: t.bg, borderColor: t.border }}>
                            <span style={{ background: t.text, opacity: 0.7 }} />
                            <span style={{ background: t.accent }} />
                          </span>
                          <span className="theme-label">{t.label}</span>
                        </button>
                      ))}
                    </div>
                  </div>

                  <div className="settings-field">
                    <div className="settings-label-row">
                      <span>字体</span>
                      <span className="settings-value">{activeFont}</span>
                    </div>
                    <select
                      className="settings-select"
                      value={settings.font_family}
                      onChange={e => updateAndSave({ font_family: e.target.value })}
                    >
                      {READER_FONTS.map(f => (
                        <option key={f.id} value={f.id}>{f.label}</option>
                      ))}
                    </select>
                  </div>

                  <div className="settings-field">
                    <div className="settings-label-row">
                      <span>字号</span>
                      <span className="settings-value">{settings.font_size}px</span>
                    </div>
                    <input
                      type="range" min={FONT_SIZE_RANGE.min} max={FONT_SIZE_RANGE.max} step={FONT_SIZE_RANGE.step}
                      value={settings.font_size}
                      onChange={e => updateAndSave({ font_size: Number(e.target.value) })}
                      className="settings-slider"
                    />
                  </div>

                  <div className="settings-field">
                    <div className="settings-label-row">
                      <span>行距</span>
                      <span className="settings-value">{settings.line_height.toFixed(2)}</span>
                    </div>
                    <input
                      type="range" min={LINE_HEIGHT_RANGE.min} max={LINE_HEIGHT_RANGE.max} step={LINE_HEIGHT_RANGE.step}
                      value={settings.line_height}
                      onChange={e => updateAndSave({ line_height: Number(e.target.value) })}
                      className="settings-slider"
                    />
                  </div>

                  <div className="settings-field">
                    <div className="settings-label-row">
                      <span>段间距</span>
                      <span className="settings-value">{settings.paragraph_spacing.toFixed(1)}em</span>
                    </div>
                    <input
                      type="range" min={PARAGRAPH_SPACING_RANGE.min} max={PARAGRAPH_SPACING_RANGE.max} step={PARAGRAPH_SPACING_RANGE.step}
                      value={settings.paragraph_spacing}
                      onChange={e => updateAndSave({ paragraph_spacing: Number(e.target.value) })}
                      className="settings-slider"
                    />
                  </div>
                </div>

                <div
                  className="settings-preview-panel"
                  style={{ background: activeTheme.bg, color: activeTheme.text, borderColor: activeTheme.border }}
                >
                  <div className="settings-preview-top">
                    <span style={{ background: activeTheme.accent }} />
                    <span />
                    <span />
                  </div>
                  <div className="settings-preview-title">章节预览</div>
                  <p
                    className="settings-preview-text"
                    style={{
                      fontSize: `${Math.min(settings.font_size, 22)}px`,
                      lineHeight: settings.line_height,
                      fontFamily: previewFontFamily,
                      marginBottom: `${settings.paragraph_spacing}em`,
                    }}
                  >
                    雨声落在窗沿，书页慢慢展开。合适的字号和行距，会让长时间阅读更轻一些。
                  </p>
                  <p
                    className="settings-preview-text muted"
                    style={{
                      fontSize: `${Math.min(settings.font_size - 1, 20)}px`,
                      lineHeight: settings.line_height,
                      fontFamily: previewFontFamily,
                    }}
                  >
                    当前主题保留清晰对比，也尽量减少视觉噪声。
                  </p>
                </div>
              </div>
            </div>
          )}

          {/* ── 听书 ── */}
          {activeSection === 'tts' && (
            <div className="settings-section">
              {!ttsLoaded || !ttsConfig ? (
                <div className="settings-loading">加载中...</div>
              ) : (
                <div className="settings-group">
                  <div className="settings-row vertical">
                    <span className="settings-label">服务商</span>
                    <select
                      className="settings-select"
                      value={ttsConfig.provider}
                      onChange={e => handleTtsSave({ ...ttsConfig, provider: e.target.value })}
                    >
                      <option value="xiaomi">小米</option>
                    </select>
                  </div>

                  <div className="settings-row vertical">
                    <span className="settings-label">API Key</span>
                    <input
                      className="settings-input"
                      type="password"
                      placeholder={ttsConfig.has_api_key ? '已设置（输入新值覆盖）' : '输入 API Key'}
                      onBlur={e => {
                        if (e.target.value) {
                          saveApiKey(e.target.value)
                          e.target.value = ''
                        }
                      }}
                    />
                  </div>

                  <div className="settings-row vertical">
                    <span className="settings-label">Base URL</span>
                    <input
                      className="settings-input"
                      type="text"
                      placeholder="https://api.example.com/v1"
                      value={ttsConfig.base_url ?? ''}
                      onChange={e => updateTtsDraft({ base_url: e.target.value })}
                      onBlur={e => saveTtsPartial({ base_url: e.target.value || null })}
                    />
                  </div>

                  <div className="settings-row vertical">
                    <span className="settings-label">Model</span>
                    <input
                      className="settings-input"
                      type="text"
                      placeholder="mimo-v2-tts"
                      value={ttsConfig.model ?? ''}
                      onChange={e => updateTtsDraft({ model: e.target.value })}
                      onBlur={e => saveTtsPartial({ model: e.target.value || null })}
                    />
                  </div>

                  <div className="settings-row vertical">
                    <span className="settings-label">音色</span>
                    <input
                      className="settings-input"
                      type="text"
                      placeholder="voice_id"
                      value={ttsConfig.voice_id ?? ''}
                      onChange={e => updateTtsDraft({ voice_id: e.target.value })}
                      onBlur={e => saveTtsPartial({ voice_id: e.target.value || null })}
                    />
                  </div>

                  <div className="settings-actions">
                    <button
                      className="settings-action-btn primary"
                      type="button"
                      onClick={testTtsConnection}
                    >测试连接</button>
                    <button
                      className="settings-action-btn"
                      type="button"
                      onClick={clearTtsCache}
                    >清空缓存</button>
                  </div>
                  {ttsTestStatus !== 'idle' && (
                    <div className={`settings-inline-status ${ttsTestStatus}`}>
                      {ttsTestStatus === 'success' ? '连接成功' : '连接失败，请检查配置'}
                    </div>
                  )}
                </div>
              )}
            </div>
          )}

          {/* ── 常规 ── */}
          {activeSection === 'general' && (
            <div className="settings-section">
              <div className="settings-group">
                <div className="settings-row vertical">
                  <span className="settings-label">应用主题</span>
                  <div className="settings-chip-group">
                    {[
                      { id: 'system', label: '系统' },
                      { id: 'light', label: '亮色' },
                      { id: 'dark', label: '深色' },
                    ].map(t => (
                      <button
                        key={t.id}
                        type="button"
                        className={`settings-chip ${settings.app_theme === t.id ? 'active' : ''}`}
                        onClick={() => updateAndSave({ app_theme: t.id })}
                      >{t.label}</button>
                    ))}
                  </div>
                </div>
                <div className="settings-row">
                  <span className="settings-label-block">
                    <span className="settings-label">自动保存进度</span>
                    <span className="settings-hint">阅读时自动记录当前位置</span>
                  </span>
                  <label className="settings-toggle">
                    <input
                      type="checkbox"
                      checked={settings.auto_save_progress}
                      onChange={e => updateAndSave({ auto_save_progress: e.target.checked })}
                    />
                    <span className="settings-toggle-track" />
                  </label>
                </div>
                <div className="settings-row">
                  <span className="settings-label-block">
                    <span className="settings-label">恢复上次位置</span>
                    <span className="settings-hint">重新打开书籍时回到上次阅读处</span>
                  </span>
                  <label className="settings-toggle">
                    <input
                      type="checkbox"
                      checked={settings.restore_last_position}
                      onChange={e => updateAndSave({ restore_last_position: e.target.checked })}
                    />
                    <span className="settings-toggle-track" />
                  </label>
                </div>
                <div className="settings-row">
                  <span className="settings-label-block">
                    <span className="settings-label">显示状态栏</span>
                    <span className="settings-hint">保留底部阅读状态信息</span>
                  </span>
                  <label className="settings-toggle">
                    <input
                      type="checkbox"
                      checked={settings.show_status_bar}
                      onChange={e => updateAndSave({ show_status_bar: e.target.checked })}
                    />
                    <span className="settings-toggle-track" />
                  </label>
                </div>
                <div className="settings-row">
                  <span className="settings-label-block">
                    <span className="settings-label">平滑滚动</span>
                    <span className="settings-hint">章节跳转时使用缓动滚动</span>
                  </span>
                  <label className="settings-toggle">
                    <input
                      type="checkbox"
                      checked={settings.smooth_scroll}
                      onChange={e => updateAndSave({ smooth_scroll: e.target.checked })}
                    />
                    <span className="settings-toggle-track" />
                  </label>
                </div>
              </div>
            </div>
          )}

          {/* ── 标签分组管理 ── */}
          {activeSection === 'tags' && (
            <TagGroupManager />
          )}

          {/* ── 关于 ── */}
          {activeSection === 'about' && (
            <div className="settings-section">
              <div className="settings-group">
                <div className="settings-about-brand">
                  <div className="settings-about-mark">轻</div>
                  <div>
                    <div className="settings-about-name">轻看</div>
                    <div className="settings-about-subtitle">本地阅读器</div>
                  </div>
                </div>
                <div className="settings-row">
                  <span className="settings-label">应用名称</span>
                  <span className="settings-value-text">轻看</span>
                </div>
                <div className="settings-row">
                  <span className="settings-label">版本</span>
                  <span className="settings-value-text">0.1.0</span>
                </div>
                <div className="settings-row">
                  <span className="settings-label">格式支持</span>
                  <span className="settings-value-text">EPUB, TXT</span>
                </div>
                <div className="settings-row">
                  <span className="settings-label-block">
                    <span className="settings-label">清理缓存</span>
                  </span>
                  <div className="settings-row-action">
                    {bookCacheClearStatus !== 'idle' && (
                      <span className={`settings-action-status ${bookCacheClearStatus}`}>
                        {bookCacheClearStatus === 'success' ? '已清理' : '清理失败'}
                      </span>
                    )}
                    <button
                      className="settings-action-btn compact"
                      type="button"
                      onClick={clearBookCache}
                    >清理缓存</button>
                  </div>
                </div>
              </div>
            </div>
          )}

        </div>
      </div>
    </main>
  )
}

export default SettingsPage
