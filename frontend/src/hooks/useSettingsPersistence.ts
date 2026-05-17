import { useCallback } from 'react'
import { settingsSave } from '../services/api'
import type { ReaderSettings } from '../services/api'
import useAppStore from '../store/useAppStore'

export function useSettingsPersistence() {
  const setSettings = useAppStore(s => s.setSettings)

  const updateAndSave = useCallback((partial: Partial<ReaderSettings>) => {
    const current = useAppStore.getState().reader.settings
    const adjusted = (current.theme === 'original' && !partial.theme && (partial.font_family || partial.font_size || partial.line_height || partial.paragraph_spacing))
      ? { ...partial, theme: 'light' as const }
      : partial
    setSettings(adjusted)
    settingsSave({ ...current, ...adjusted }).catch(() => {})
  }, [setSettings])

  return updateAndSave
}
