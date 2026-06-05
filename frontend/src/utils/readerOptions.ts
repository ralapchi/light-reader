export interface ReaderThemeOption {
  id: string
  label: string
  bg: string
  text: string
  accent: string
  border: string
  surface: string
  textSec: string
  textTer: string
  hover: string
  accentSoft: string
}

export interface ReaderFontOption {
  id: string
  label: string
}

export const READER_THEMES: ReaderThemeOption[] = [
  { id: 'original', label: '默认', bg: '#FAF8F5', text: '#1A1A1A', accent: '#C8553D', border: '#E8E4DF', surface: '#FFFFFF', textSec: '#8A8680', textTer: '#B5AFA8', hover: '#F5F2EE', accentSoft: '#F2E0D9' },
  { id: 'light', label: '护眼', bg: '#E0EDE4', text: '#2C3B30', accent: '#5B8C5A', border: '#C8D8CC', surface: '#EAF4EC', textSec: '#6B7B6E', textTer: '#9AACA0', hover: '#D4E8D8', accentSoft: '#DCEFE0' },
  { id: 'dark', label: '夜间', bg: '#181715', text: '#FAFAF5', accent: '#CC785C', border: '#2E2D2A', surface: '#222120', textSec: '#A09C96', textTer: '#6B6760', hover: '#2E2D2A', accentSoft: '#3D2A20' },
  { id: 'sepia', label: '暖纸', bg: '#F5F0E8', text: '#2C2420', accent: '#B5624A', border: '#DDD5C8', surface: '#FAF6F0', textSec: '#7A7068', textTer: '#A89E94', hover: '#EDE5DA', accentSoft: '#E8D5C8' },
]

export const READER_FONTS: ReaderFontOption[] = [
  { id: 'sans-serif', label: '黑体' },
  { id: 'serif', label: '宋体' },
  { id: 'kai', label: '楷体' },
  { id: 'yahei', label: '微软雅黑' },
  { id: 'fangsong', label: '仿宋' },
  { id: 'monospace', label: '等宽' },
]

export const FONT_SIZE_RANGE = { min: 12, max: 28, step: 1 }
export const LINE_HEIGHT_RANGE = { min: 1.2, max: 2.5, step: 0.05 }
export const PARAGRAPH_SPACING_RANGE = { min: 0.4, max: 3.0, step: 0.1 }
export const CONTENT_WIDTH_RANGE = { min: 400, max: 800, step: 10 }

export const LINE_HEIGHT_PRESETS = [
  { label: '紧凑', value: 1.4 },
  { label: '适中', value: 1.8 },
  { label: '舒适', value: 2.2 },
]

export const PARAGRAPH_SPACING_PRESETS = [
  { label: '紧凑', value: 0.5 },
  { label: '适中', value: 1.0 },
  { label: '宽松', value: 2.0 },
]

export const CONTENT_WIDTH_PRESETS = [
  { label: '窄', value: 480 },
  { label: '适中', value: 600 },
  { label: '宽', value: 720 },
]

export function findReaderTheme(themeId: string): ReaderThemeOption {
  return READER_THEMES.find(theme => theme.id === themeId) ?? READER_THEMES[0]
}

export function findReaderFont(fontId: string): ReaderFontOption | undefined {
  return READER_FONTS.find(font => font.id === fontId)
}

export function readerFontFamily(fontFamily: string): string {
  switch (fontFamily) {
    case 'serif':
      return "'Songti SC', 'STSong', serif"
    case 'kai':
      return "'Kaiti SC', 'STKaiti', cursive"
    case 'yahei':
      return "'Microsoft YaHei', 'PingFang SC', sans-serif"
    case 'fangsong':
      return "'FangSong', 'STFangSong', serif"
    case 'monospace':
      return "'Courier New', monospace"
    default:
      return "'DM Sans', -apple-system, sans-serif"
  }
}
