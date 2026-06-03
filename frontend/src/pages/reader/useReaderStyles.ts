import { useMemo, type CSSProperties } from 'react'
import type { ReaderSettings } from '../../services/api'
import { findReaderTheme, readerFontFamily } from '../../utils/readerOptions'

export function useReaderStyles(settings: ReaderSettings, isOriginal: boolean) {
  const currentTheme = findReaderTheme(settings.theme)

  const readerStyle = useMemo(() => ({
    '--bg': currentTheme.bg,
    '--text-primary': currentTheme.text,
    '--accent': currentTheme.accent,
    '--border': currentTheme.border,
    '--surface': currentTheme.surface,
    '--text-secondary': currentTheme.textSec,
    '--text-tertiary': currentTheme.textTer,
    '--surface-hover': currentTheme.hover,
    '--accent-soft': currentTheme.accentSoft,
  } as CSSProperties), [currentTheme])

  const contentStyle = useMemo<CSSProperties>(() => isOriginal ? {} : {
    fontFamily: readerFontFamily(settings.font_family),
    fontSize: `${settings.font_size}px`,
    lineHeight: settings.line_height,
  }, [isOriginal, settings.font_family, settings.font_size, settings.line_height])

  const paragraphStyle = useMemo<CSSProperties>(() => isOriginal ? {} : {
    marginBottom: `${settings.paragraph_spacing}em`,
  }, [isOriginal, settings.paragraph_spacing])

  return { readerStyle, contentStyle, paragraphStyle }
}
