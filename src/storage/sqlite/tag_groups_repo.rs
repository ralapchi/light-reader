use rusqlite::params;

use crate::domain::tag_group::TagGroup;
use crate::storage::traits::TagGroupsRepo;

use super::connection::SqlitePool;

pub struct SqliteTagGroupsRepo {
    pool: SqlitePool,
}

impl SqliteTagGroupsRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn row_to_tag_group(row: &rusqlite::Row) -> rusqlite::Result<TagGroup> {
    Ok(TagGroup {
        id: row.get("id")?,
        name: row.get("name")?,
        color: row.get("color")?,
        sort_order: row.get::<_, i32>("sort_order")?,
    })
}

impl TagGroupsRepo for SqliteTagGroupsRepo {
    fn list_all(&self) -> Result<Vec<TagGroup>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, name, color, sort_order FROM tag_groups ORDER BY sort_order, name")
            .map_err(|e| e.to_string())?;
        let groups = stmt
            .query_map([], row_to_tag_group)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(groups)
    }

    fn create(&self, group: &TagGroup) -> Result<(), String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO tag_groups (id, name, color, sort_order) VALUES (?1, ?2, ?3, ?4)",
            params![group.id, group.name, group.color, group.sort_order],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn update(&self, group: &TagGroup) -> Result<(), String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE tag_groups SET name = ?2, color = ?3, sort_order = ?4 WHERE id = ?1",
            params![group.id, group.name, group.color, group.sort_order],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn delete(&self, group_id: &str) -> Result<(), String> {
        if group_id == "default" {
            return Err("不能删除默认分组".to_string());
        }
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        // Move tags in this group to default group
        conn.execute(
            "UPDATE tags SET group_id = 'default' WHERE group_id = ?1",
            params![group_id],
        )
        .map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM tag_groups WHERE id = ?1", params![group_id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn list_tags(&self, group_id: &str) -> Result<Vec<String>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT tag FROM tags WHERE group_id = ?1 ORDER BY tag")
            .map_err(|e| e.to_string())?;
        let tags = stmt
            .query_map(params![group_id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<String>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(tags)
    }

    fn add_tag(&self, tag: &str, group_id: &str) -> Result<(), String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        // Upsert: if tag exists update group, else insert
        conn.execute(
            "INSERT INTO tags (tag, group_id) VALUES (?1, ?2)
             ON CONFLICT(tag) DO UPDATE SET group_id = ?2",
            params![tag, group_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn remove_tag(&self, tag: &str) -> Result<(), String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        // Move tag to default group instead of deleting
        conn.execute(
            "UPDATE tags SET group_id = 'default' WHERE tag = ?1",
            params![tag],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn all_tags_with_groups(&self) -> Result<Vec<(String, Option<String>)>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT tag, group_id FROM tags ORDER BY tag")
            .map_err(|e| e.to_string())?;
        let tags = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(tags)
    }
}
