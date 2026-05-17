import { useCallback, useEffect, useState } from 'react'
import { ttsClearCache, ttsConfigLoad, ttsConfigSave, ttsTestConnection } from '../../services/api'
import type { TtsConfigDto } from '../../services/api'
import useAppStore from '../../store/useAppStore'
import { findReaderFont, findReaderTheme, readerFontFamily } from '../../utils/readerOptions'
import { useSettingsPersistence } from '../../hooks/useSettingsPersistence'

export type SettingsSection = 'general' | 'tts' | 'reading' | 'about'
export type TtsTestStatus = 'idle' | 'success' | 'error'

export function useSettingsPage() {
  const setSidebarFooter = useAppStore(s => s.setSidebarFooter)
  const settings = useAppStore(s => s.reader.settings)
  const [activeSection, setActiveSection] = useState<SettingsSection>('general')
  const [ttsConfig, setTtsConfig] = useState<TtsConfigDto | null>(null)
  const [ttsLoaded, setTtsLoaded] = useState(false)
  const [ttsTestStatus, setTtsTestStatus] = useState<TtsTestStatus>('idle')

  useEffect(() => {
    setSidebarFooter('设置')
  }, [setSidebarFooter])

  useEffect(() => {
    ttsConfigLoad().then(cfg => {
      setTtsConfig(cfg)
      setTtsLoaded(true)
    }).catch(() => setTtsLoaded(true))
  }, [])

  const handleTtsSave = useCallback((next: TtsConfigDto) => {
    setTtsConfig(next)
    ttsConfigSave(next).catch(() => {})
  }, [])

  const updateTtsDraft = useCallback((partial: Partial<TtsConfigDto>) => {
    setTtsConfig(prev => prev ? { ...prev, ...partial } : prev)
  }, [])

  const saveTtsPartial = useCallback((partial: Partial<TtsConfigDto>) => {
    const next = ttsConfig ? { ...ttsConfig, ...partial } : null
    if (!next) return
    setTtsConfig(next)
    ttsConfigSave(next).catch(() => {})
  }, [ttsConfig])

  const saveApiKey = useCallback((apiKey: string) => {
    if (!ttsConfig || !apiKey) return
    ttsConfigSave({ ...ttsConfig, api_key: apiKey }).then(() => {
      setTtsConfig(prev => prev ? { ...prev, has_api_key: true } : prev)
    })
  }, [ttsConfig])

  const testTtsConnection = useCallback(async () => {
    if (!ttsConfig) return
    const ok = await ttsTestConnection(ttsConfig)
    setTtsTestStatus(ok ? 'success' : 'error')
  }, [ttsConfig])

  const updateAndSave = useSettingsPersistence()

  return {
    activeFont: findReaderFont(settings.font_family)?.label ?? '自定义',
    activeSection,
    activeTheme: findReaderTheme(settings.theme),
    clearTtsCache: ttsClearCache,
    handleTtsSave,
    previewFontFamily: readerFontFamily(settings.font_family),
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
  }
}
