import { useEffect, useState, useCallback } from 'react'
import useAppStore from '../store/useAppStore'
import { ttsConfigLoad, ttsConfigSave, ttsTestConnection, ttsClearCache } from '../services/api'
import type { TtsConfigDto } from '../services/api'
import './SettingsPage.css'

function SettingsPage() {
  const setSidebarFooter = useAppStore(s => s.setSidebarFooter)
  const [ttsConfig, setTtsConfig] = useState<TtsConfigDto | null>(null)
  const [loaded, setLoaded] = useState(false)

  useEffect(() => {
    setSidebarFooter('阅读设置')
  }, [setSidebarFooter])

  useEffect(() => {
    ttsConfigLoad().then(cfg => {
      setTtsConfig(cfg)
      setLoaded(true)
    }).catch(() => setLoaded(true))
  }, [])

  const handleSave = useCallback((next: TtsConfigDto) => {
    setTtsConfig(next)
    ttsConfigSave(next).catch(() => {})
  }, [])

  return (
    <main className="settings-main">
      <div className="settings-page-header">
        <h1>阅读设置</h1>
      </div>

      {!loaded || !ttsConfig ? (
        <div className="settings-loading">加载中...</div>
      ) : (
        <div className="settings-body">
          <div className="settings-section">
            <div className="settings-section-header">
              <h2 className="settings-section-title">
                听书
                <span className="settings-pro-badge">Pro</span>
              </h2>
              <p className="settings-section-desc">配置 TTS 语音朗读服务，开启后可在阅读时使用听书功能</p>
            </div>

            <div className="settings-group">
              <div className="settings-row">
                <span className="settings-label">启用听书</span>
                <label className="settings-toggle">
                  <input
                    type="checkbox"
                    checked={ttsConfig.enabled}
                    onChange={e => handleSave({ ...ttsConfig, enabled: e.target.checked })}
                  />
                  <span className="settings-toggle-track" />
                </label>
              </div>

              <div className="settings-row vertical">
                <span className="settings-label">API Key</span>
                <input
                  className="settings-input"
                  type="password"
                  placeholder={ttsConfig.has_api_key ? '已设置（输入新值覆盖）' : '输入 API Key'}
                  onBlur={e => {
                    if (e.target.value) {
                      ttsConfigSave({ ...ttsConfig, api_key: e.target.value }).then(() => {
                        setTtsConfig(prev => prev ? { ...prev, has_api_key: true } : prev)
                        e.target.value = ''
                      })
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
                  onChange={e => setTtsConfig({ ...ttsConfig, base_url: e.target.value })}
                  onBlur={e => ttsConfigSave({ ...ttsConfig, base_url: e.target.value || null })}
                />
              </div>

              <div className="settings-row vertical">
                <span className="settings-label">Model</span>
                <input
                  className="settings-input"
                  type="text"
                  placeholder="mimo-v2-tts"
                  value={ttsConfig.model ?? ''}
                  onChange={e => setTtsConfig({ ...ttsConfig, model: e.target.value })}
                  onBlur={e => ttsConfigSave({ ...ttsConfig, model: e.target.value || null })}
                />
              </div>

              <div className="settings-row vertical">
                <span className="settings-label">音色</span>
                <input
                  className="settings-input"
                  type="text"
                  placeholder="voice_id"
                  value={ttsConfig.voice_id ?? ''}
                  onChange={e => setTtsConfig({ ...ttsConfig, voice_id: e.target.value })}
                  onBlur={e => ttsConfigSave({ ...ttsConfig, voice_id: e.target.value || null })}
                />
              </div>

              <div className="settings-actions">
                <button
                  className="settings-action-btn"
                  onClick={async () => {
                    const ok = await ttsTestConnection(ttsConfig)
                    alert(ok ? '连接成功' : '连接失败')
                  }}
                >
                  测试连接
                </button>
                <button
                  className="settings-action-btn"
                  onClick={() => ttsClearCache()}
                >
                  清空缓存
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </main>
  )
}

export default SettingsPage
