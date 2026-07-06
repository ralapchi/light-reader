import { useState, useEffect, useCallback } from 'react'
import type { TagGroupDto } from '../../services/api'
import {
  tagGroupList,
  tagGroupCreate,
  tagGroupUpdate,
  tagGroupDelete,
  tagGroupAddTag,
  tagGroupRemoveTag,
} from '../../services/api'
import './TagGroupManager.css'

const GROUP_COLORS = ['#C8553D', '#6B8E6B', '#5B7B9B', '#9B7B5B', '#7B6B8A', '#B8915A']

export default function TagGroupManager() {
  const [groups, setGroups] = useState<TagGroupDto[]>([])
  const [activeGroupId, setActiveGroupId] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)
  const [newTagName, setNewTagName] = useState('')
  const [editingName, setEditingName] = useState('')
  const [editingColor, setEditingColor] = useState<string | null>(null)

  const loadGroups = useCallback(async () => {
    try {
      setLoading(true)
      const data = await tagGroupList()
      setGroups(data)
      if (!activeGroupId && data.length > 0) {
        setActiveGroupId(data[0].id)
      }
    } catch (e) {
      console.error('Failed to load tag groups:', e)
    } finally {
      setLoading(false)
    }
  }, [activeGroupId])

  useEffect(() => { loadGroups() }, [loadGroups])

  const activeGroup = groups.find(g => g.id === activeGroupId) || null

  useEffect(() => {
    if (activeGroup) {
      setEditingName(activeGroup.name)
      setEditingColor(activeGroup.color || GROUP_COLORS[0])
    }
  }, [activeGroupId]) // eslint-disable-line react-hooks/exhaustive-deps

  const handleCreateGroup = async () => {
    try {
      const color = GROUP_COLORS[groups.length % GROUP_COLORS.length]
      const newGroup = await tagGroupCreate('新分组', color)
      setGroups(prev => [...prev, { ...newGroup, tags: [] }])
      setActiveGroupId(newGroup.id)
    } catch (e) {
      console.error('Failed to create group:', e)
    }
  }

  const handleUpdateGroup = async () => {
    if (!activeGroup) return
    try {
      await tagGroupUpdate(activeGroup.id, editingName, editingColor ?? undefined)
      setGroups(prev =>
        prev.map(g =>
          g.id === activeGroup.id
            ? { ...g, name: editingName, color: editingColor }
            : g
        )
      )
    } catch (e) {
      console.error('Failed to update group:', e)
    }
  }

  const handleDeleteGroup = async () => {
    if (!activeGroup || activeGroup.id === 'default') return
    if (!confirm(`确定删除「${activeGroup.name}」分组？组内标签将移至"未分组"。`)) return
    try {
      await tagGroupDelete(activeGroup.id)
      setGroups(prev => prev.filter(g => g.id !== activeGroup.id))
      setActiveGroupId(groups.find(g => g.id !== activeGroup.id)?.id ?? null)
    } catch (e) {
      console.error('Failed to delete group:', e)
    }
  }

  const handleAddTag = async () => {
    if (!activeGroup || !newTagName.trim()) return
    const tagName = newTagName.trim()
    if (activeGroup.tags.includes(tagName)) return
    try {
      await tagGroupAddTag(tagName, activeGroup.id)
      setGroups(prev =>
        prev.map(g =>
          g.id === activeGroup.id
            ? { ...g, tags: [...g.tags, tagName].sort() }
            : g
        )
      )
      setNewTagName('')
    } catch (e) {
      console.error('Failed to add tag:', e)
    }
  }

  const handleRemoveTag = async (tag: string) => {
    if (!activeGroup) return
    try {
      await tagGroupRemoveTag(tag)
      setGroups(prev =>
        prev.map(g =>
          g.id === activeGroup.id
            ? { ...g, tags: g.tags.filter(t => t !== tag) }
            : g
        )
      )
    } catch (e) {
      console.error('Failed to remove tag:', e)
    }
  }

  if (loading) {
    return <div className="tgm-loading">加载中...</div>
  }

  return (
    <div className="settings-section">
      <div className="tgm-layout">
        {/* Left: group list */}
        <div className="tgm-side">
          {groups.map(g => (
            <button
              key={g.id}
              className={`tgm-side-item ${g.id === activeGroupId ? 'active' : ''}`}
              onClick={() => setActiveGroupId(g.id)}
            >
              <span className="tgm-side-dot" style={{ background: g.color || '#B5AFA8' }} />
              <span className="tgm-side-name">{g.name}</span>
              <span className="tgm-side-count">{g.tags.length}</span>
            </button>
          ))}
          <button className="tgm-side-add" onClick={handleCreateGroup}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <line x1="12" y1="5" x2="12" y2="19" />
              <line x1="5" y1="12" x2="19" y2="12" />
            </svg>
            新建分组
          </button>
        </div>

        {/* Right: group detail */}
        <div className="tgm-body">
          {!activeGroup ? (
            <div className="tgm-empty">选择或创建一个分组</div>
          ) : (
            <>
              <div className="tgm-body-title">分组详情</div>

              <div className="tgm-field">
                <div className="tgm-field-label">分组名称</div>
                <div className="tgm-field-hint">标签分组的显示名称</div>
                <input
                  className="tgm-field-input"
                  type="text"
                  value={editingName}
                  onChange={e => setEditingName(e.target.value)}
                  onBlur={handleUpdateGroup}
                  disabled={activeGroup.id === 'default'}
                />
              </div>

              <div className="tgm-field">
                <div className="tgm-field-label">分组颜色</div>
                <div className="tgm-field-hint">标签 pill 的主题颜色</div>
                <div className="tgm-color-picker">
                  {GROUP_COLORS.map(c => (
                    <div
                      key={c}
                      className={`tgm-color-swatch ${editingColor === c ? 'selected' : ''}`}
                      style={{ background: c }}
                      onClick={() => {
                        setEditingColor(c)
                        if (activeGroup) {
                          tagGroupUpdate(activeGroup.id, editingName, c).catch(() => {})
                          setGroups(prev =>
                            prev.map(g =>
                              g.id === activeGroup.id ? { ...g, color: c } : g
                            )
                          )
                        }
                      }}
                    />
                  ))}
                </div>
              </div>

              <div className="tgm-field">
                <div className="tgm-field-label">
                  组内标签
                  <span className="tgm-field-count">({activeGroup.tags.length})</span>
                </div>
                <div className="tgm-field-hint">属于该分组的标签列表</div>
                <div className="tgm-tags">
                  {activeGroup.tags.map(tag => (
                    <span
                      key={tag}
                      className="tgm-tag"
                      style={{
                        background: `${activeGroup.color || '#C8553D'}15`,
                        color: activeGroup.color || '#C8553D',
                      }}
                    >
                      {tag}
                      <span
                        className="tgm-tag-remove"
                        onClick={() => handleRemoveTag(tag)}
                      >
                        &times;
                      </span>
                    </span>
                  ))}
                  {activeGroup.tags.length === 0 && (
                    <span className="tgm-tags-empty">暂无标签</span>
                  )}
                </div>
                {activeGroup.id !== 'default' && (
                  <div className="tgm-add-input">
                    <input
                      type="text"
                      placeholder="输入新标签名称..."
                      value={newTagName}
                      onChange={e => setNewTagName(e.target.value)}
                      onKeyDown={e => { if (e.key === 'Enter') handleAddTag() }}
                    />
                    <button className="btn-primary" onClick={handleAddTag}>添加</button>
                  </div>
                )}
              </div>

              {activeGroup.id !== 'default' && (
                <div className="tgm-field" style={{ borderBottom: 'none', paddingTop: 16 }}>
                  <button className="tgm-delete-btn" onClick={handleDeleteGroup}>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <polyline points="3 6 5 6 21 6" />
                      <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                    </svg>
                    删除此分组
                  </button>
                </div>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  )
}
