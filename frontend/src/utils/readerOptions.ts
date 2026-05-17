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
  compactLabel: string
}

export const READER_THEMES: ReaderThemeOption[] = [
  { id: 'original', label: '原始', bg: '#FAF8F5', text: '#1A1A1A', accent: '#C8553D', border: '#E8E4DF', surface: '#FFFFFF', textSec: '#8A8680', textTer: '#B5AFA8', hover: '#F5F2EE', accentSoft: '#F2E0D9' },
  { id: 'light', label: '亮色', bg: '#FAF8F5', text: '#1A1A1A', accent: '#C8553D', border: '#E8E4DF', surface: '#FFFFFF', textSec: '#8A8680', textTer: '#B5AFA8', hover: '#F5F2EE', accentSoft: '#F2E0D9' },
  { id: 'dark', label: '深色', bg: '#181715', text: '#FAFAF5', accent: '#CC785C', border: '#2E2D2A', surface: '#222120', textSec: '#A09C96', textTer: '#6B6760', hover: '#2E2D2A', accentSoft: '#3D2A20' },
  { id: 'sepia', label: '暖纸', bg: '#F5F0E8', text: '#2C2420', accent: '#B5624A', border: '#DDD5C8', surface: '#FAF6F0', textSec: '#7A7068', textTer: '#A89E94', hover: '#EDE5DA', accentSoft: '#E8D5C8' },
]

export const READER_FONTS: ReaderFontOption[] = [
  { id: 'sans-serif', label: '黑体', compactLabel: '无衬线' },
  { id: 'serif', label: '宋体', compactLabel: '衬线' },
  { id: 'kai', label: '楷体', compactLabel: '楷体' },
  { id: 'yahei', label: '微软雅黑', compactLabel: '雅黑' },
  { id: 'fangsong', label: '仿宋', compactLabel: '仿宋' },
  { id: 'monospace', label: '等宽', compactLabel: '等宽' },
]

export const COMPACT_READER_FONTS = READER_FONTS.filter(font =>
  ['sans-serif', 'serif', 'monospace'].includes(font.id),
)

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
