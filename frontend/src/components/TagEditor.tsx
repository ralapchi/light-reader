import { useState, useEffect, useRef, useCallback } from 'react'
import type { BookTagGroupDto, TagGroupDto } from '../services/api'
import { libraryGetTags, librarySetTags, tagGroupList, tagGroupAddTag } from '../services/api'
import './TagEditor.css'

interface TagEditorProps {
  bookId: string
  onTagsChanged?: () => void
}

export default function TagEditor({ bookId, onTagsChanged }: TagEditorProps) {
  const [bookTags, setBookTags] = useState<BookTagGroupDto[]>([])
  const [allGroups, setAllGroups] = useState<TagGroupDto[]>([])
  const [allTags, setAllTags] = useState<string[]>([])
  const [loading, setLoading] = useState(true)
  const [popoverGroup, setPopoverGroup] = useState<string | null>(null)
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedTag, setSelectedTag] = useState<string | null>(null)
  const popoverRef = useRef<HTMLDivElement>(null)
  const anchorRef = useRef<HTMLButtonElement>(null)

  const loadData = useCallback(async () => {
    try {
      setLoading(true)
      const [bt, groups] = await Promise.all([
        libraryGetTags(bookId),
        tagGroupList(),
      ])
      setBookTags(bt.groups)
      setAllGroups(groups)
      // Collect all known tags across groups
      const tags = groups.flatMap(g => g.tags)
      setAllTags([...new Set(tags)].sort())
    } catch (e) {
      console.error('Failed to load tags:', e)
    } finally {
      setLoading(false)
    }
  }, [bookId])

  useEffect(() => { loadData() }, [loadData])

  const currentTagSet = new Set(bookTags.flatMap(g => g.tags))

  const handleRemoveTag = async (tag: string) => {
    const newTags = [...currentTagSet].filter(t => t !== tag)
    try {
      await librarySetTags(bookId, newTags)
      setBookTags(prev =>
        prev.map(g => ({
          ...g,
          tags: g.tags.filter(t => t !== tag),
        }))
      )
      onTagsChanged?.()
    } catch (e) {
      console.error('Failed to remove tag:', e)
    }
  }

  const handleAddTag = async (tag: string, groupId: string) => {
    if (currentTagSet.has(tag)) return
    const newTags = [...currentTagSet, tag]
    try {
      // Ensure tag exists in the group
      await tagGroupAddTag(tag, groupId)
      await librarySetTags(bookId, newTags)
      setBookTags(prev => {
        const updated = prev.map(g => {
          if (g.group_id === groupId && !g.tags.includes(tag)) {
            return { ...g, tags: [...g.tags, tag].sort() }
          }
          return g
        })
        return updated
      })
      setPopoverGroup(null)
      setSearchQuery('')
      setSelectedTag(null)
      onTagsChanged?.()
    } catch (e) {
      console.error('Failed to add tag:', e)
    }
  }

  const handleCreateAndAdd = async (tag: string, groupId: string) => {
    if (!tag.trim()) return
    await handleAddTag(tag.trim(), groupId)
  }

  if (loading) {
    return <div className="tag-editor-loading">加载标签...</div>
  }

  // Get available tags for a group (not already assigned to this book)
  const getAvailableTags = (groupId: string) => {
    const group = allGroups.find(g => g.id === groupId)
    if (!group) return []
    return group.tags.filter(t => !currentTagSet.has(t))
  }

  const getFilteredTags = (groupId: string) => {
    const available = getAvailableTags(groupId)
    if (!searchQuery.trim()) return available
    const q = searchQuery.toLowerCase()
    return available.filter(t => t.toLowerCase().includes(q))
  }

  const showCreateOption = () => {
    if (!searchQuery.trim()) return false
    const q = searchQuery.trim().toLowerCase()
    return !allTags.some(t => t.toLowerCase() === q)
  }

  return (
    <div className="tag-editor">
      {bookTags.map(group => (
        <div key={group.group_id} className="tag-editor-group">
          <div className="tag-editor-group-header">
            <div className="tag-editor-group-name">
              <span
                className="tag-editor-group-dot"
                style={{ background: group.color || '#B5AFA8' }}
              />
              <span>{group.group_name}</span>
            </div>
            <button
              className="tag-editor-add-btn"
              ref={popoverGroup === group.group_id ? anchorRef : undefined}
              onClick={(e) => {
                e.stopPropagation()
                setPopoverGroup(popoverGroup === group.group_id ? null : group.group_id)
                setSearchQuery('')
                setSelectedTag(null)
              }}
            >
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <line x1="12" y1="5" x2="12" y2="19" />
                <line x1="5" y1="12" x2="19" y2="12" />
              </svg>
              添加标签
            </button>
          </div>
          <div className="tag-editor-tag-list">
            {group.tags.length === 0 && (
              <span className="tag-editor-empty">无标签</span>
            )}
            {group.tags.map(tag => (
              <span
                key={tag}
                className="tag-editor-pill"
                style={{
                  '--tag-color': group.color || '#C8553D',
                } as React.CSSProperties}
              >
                {tag}
                <span
                  className="tag-editor-pill-remove"
                  onClick={(e) => {
                    e.stopPropagation()
                    handleRemoveTag(tag)
                  }}
                >
                  &times;
                </span>
              </span>
            ))}
          </div>

          {/* Popover for this group */}
          {popoverGroup === group.group_id && (
            <>
              <div
                className="tag-editor-popover-backdrop"
                onClick={() => {
                  setPopoverGroup(null)
                  setSearchQuery('')
                  setSelectedTag(null)
                }}
              />
              <div className="tag-editor-popover" ref={popoverRef}>
                <div className="tag-editor-popover-title">
                  添加标签到「{group.group_name}」
                </div>
                <div className="tag-editor-popover-search">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <circle cx="11" cy="11" r="8" />
                    <line x1="21" y1="21" x2="16.65" y2="16.65" />
                  </svg>
                  <input
                    type="text"
                    placeholder="搜索或新建标签..."
                    value={searchQuery}
                    onChange={e => setSearchQuery(e.target.value)}
                    autoFocus
                  />
                </div>
                <div className="tag-editor-popover-hint">可选标签</div>
                <div className="tag-editor-popover-tags">
                  {getFilteredTags(group.group_id).map(tag => (
                    <span
                      key={tag}
                      className={`tag-editor-popover-tag ${selectedTag === tag ? 'selected' : ''}`}
                      onClick={() => setSelectedTag(selectedTag === tag ? null : tag)}
                    >
                      {tag}
                    </span>
                  ))}
                  {getFilteredTags(group.group_id).length === 0 && !showCreateOption() && (
                    <span className="tag-editor-popover-empty">该组暂无可选标签</span>
                  )}
                </div>
                {showCreateOption() && (
                  <button
                    className="tag-editor-popover-create"
                    onClick={() => handleCreateAndAdd(searchQuery.trim(), group.group_id)}
                  >
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <line x1="12" y1="5" x2="12" y2="19" />
                      <line x1="5" y1="12" x2="19" y2="12" />
                    </svg>
                    创建新标签：{searchQuery.trim()}
                  </button>
                )}
                <div className="tag-editor-popover-actions">
                  <button
                    className="btn-secondary"
                    onClick={() => {
                      setPopoverGroup(null)
                      setSearchQuery('')
                      setSelectedTag(null)
                    }}
                  >
                    取消
                  </button>
                  <button
                    className="btn-primary"
                    disabled={!selectedTag}
                    onClick={() => {
                      if (selectedTag) {
                        handleAddTag(selectedTag, group.group_id)
                      }
                    }}
                  >
                    确认
                  </button>
                </div>
              </div>
            </>
          )}
        </div>
      ))}
    </div>
  )
}
