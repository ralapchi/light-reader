import type { LibraryBookCardDto, LibraryImportResultDto } from '../services/api'
import { libraryCover } from '../services/api'
import useAppStore from '../store/useAppStore'

/**
 * 合并导入结果到现有书籍列表
 */
export function mergeImportedBooks(
  currentBooks: LibraryBookCardDto[],
  importedBooks: LibraryBookCardDto[]
): LibraryBookCardDto[] {
  const merged = [...currentBooks]
  for (const book of importedBooks) {
    const idx = merged.findIndex(b => b.book_id === book.book_id)
    if (idx >= 0) {
      merged[idx] = book
    } else {
      merged.push(book)
    }
  }
  return merged
}

/**
 * 生成导入结果的 toast 消息
 */
export function getImportToastMessage(result: LibraryImportResultDto): {
  type: 'success' | 'error'
  message: string
  detail: string
} {
  const { new_count, updated_count, failed_count } = result

  if (failed_count > 0 && new_count === 0 && updated_count === 0) {
    return { type: 'error', message: '导入失败', detail: `${failed_count} 个文件导入失败` }
  }

  const detail = new_count > 0
    ? `新增 ${new_count} 本${updated_count > 0 ? `，更新 ${updated_count} 本` : ''}`
    : `已更新 ${updated_count} 本`

  return { type: 'success', message: '导入成功', detail }
}

/**
 * 异步加载书籍封面（fire-and-forget）
 */
export function loadBookCovers(books: LibraryBookCardDto[]): void {
  const setCoverImage = useAppStore.getState().setCoverImage
  for (const book of books) {
    libraryCover(book.book_id)
      .then(uri => { if (uri) setCoverImage(book.book_id, uri) })
      .catch(() => {})
  }
}
