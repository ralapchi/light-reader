const COVER_COLORS = ['cover-1', 'cover-2', 'cover-3', 'cover-4', 'cover-5', 'cover-6']

export function coverColor(bookId: string): string {
  let hash = 0
  for (let i = 0; i < bookId.length; i++) {
    hash = (hash * 31 + bookId.charCodeAt(i)) >>> 0
  }
  return COVER_COLORS[hash % COVER_COLORS.length]
}
