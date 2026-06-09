import { useCallback, useRef, useEffect } from 'react'
import { settingsSave } from '../services/api'
import type { ReaderSettings } from '../services/api'
import useAppStore from '../store/useAppStore'

export function useSettingsPersistence() {
  const setSettings = useAppStore(s => s.setSettings)
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  // Cleanup pending timer on unmount
  useEffect(() => {
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current)
    }
  }, [])

  const updateAndSave = useCallback((partial: Partial<ReaderSettings>) => {
    const current = useAppStore.getState().reader.settings
    const adjusted = (current.theme === 'original' && !partial.theme && (partial.font_family || partial.font_size || partial.line_height || partial.paragraph_spacing))
      ? { ...partial, theme: 'light' as const }
      : partial
    setSettings(adjusted)

    // Debounce IPC: merge rapid slider changes into a single save
    if (timerRef.current) clearTimeout(timerRef.current)
    timerRef.current = setTimeout(() => {
      settingsSave({ ...current, ...adjusted }).catch(() => {})
      timerRef.current = null
    }, 500)
  }, [setSettings])

  return updateAndSave
}
