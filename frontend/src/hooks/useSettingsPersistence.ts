import { useCallback } from 'react'
import { settingsSave } from '../services/api'
import type { ReaderSettings } from '../services/api'
import useAppStore from '../store/useAppStore'

export function useSettingsPersistence() {
  const { reader, setSettings } = useAppStore()
  const { settings } = reader

  const updateAndSave = useCallback((partial: Partial<ReaderSettings>) => {
    if (settings.theme === 'original' && !partial.theme && (partial.font_family || partial.font_size || partial.line_height || partial.paragraph_spacing)) {
      partial = { ...partial, theme: 'light' }
    }
    setSettings(partial)
    const next = { ...settings, ...partial }
    settingsSave(next).catch(() => {})
  }, [settings, setSettings])

  return updateAndSave
}
