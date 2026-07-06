use super::super::dto::*;
use super::dto_convert::tag_group_to_dto;

#[tauri::command]
pub fn tag_group_list(
    db: tauri::State<'_, Box<dyn crate::storage::traits::DatabaseBackend>>,
) -> Result<Vec<TagGroupDto>, String> {
    let groups = db.tag_groups().list_all()?;
    let mut result = Vec::new();
    for g in groups {
        let tags = db.tag_groups().list_tags(&g.id)?;
        result.push(tag_group_to_dto(&g, tags));
    }
    Ok(result)
}

#[tauri::command]
pub fn tag_group_create(
    name: String,
    color: Option<String>,
    db: tauri::State<'_, Box<dyn crate::storage::traits::DatabaseBackend>>,
) -> Result<TagGroupDto, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let group = crate::domain::tag_group::TagGroup {
        id,
        name,
        color,
        sort_order: 0,
    };
    db.tag_groups().create(&group)?;
    Ok(tag_group_to_dto(&group, vec![]))
}

#[tauri::command]
pub fn tag_group_update(
    id: String,
    name: String,
    color: Option<String>,
    sort_order: Option<i32>,
    db: tauri::State<'_, Box<dyn crate::storage::traits::DatabaseBackend>>,
) -> Result<(), String> {
    // Fetch existing to preserve sort_order if not provided
    let groups = db.tag_groups().list_all()?;
    let existing = groups.iter().find(|g| g.id == id);
    let so = sort_order.or(existing.map(|g| g.sort_order)).unwrap_or(0);
    let group = crate::domain::tag_group::TagGroup {
        id,
        name,
        color,
        sort_order: so,
    };
    db.tag_groups().update(&group)
}

#[tauri::command]
pub fn tag_group_delete(
    id: String,
    db: tauri::State<'_, Box<dyn crate::storage::traits::DatabaseBackend>>,
) -> Result<(), String> {
    db.tag_groups().delete(&id)
}

#[tauri::command]
pub fn tag_group_add_tag(
    tag: String,
    group_id: String,
    db: tauri::State<'_, Box<dyn crate::storage::traits::DatabaseBackend>>,
) -> Result<(), String> {
    db.tag_groups().add_tag(&tag, &group_id)
}

#[tauri::command]
pub fn tag_group_remove_tag(
    tag: String,
    db: tauri::State<'_, Box<dyn crate::storage::traits::DatabaseBackend>>,
) -> Result<(), String> {
    db.tag_groups().remove_tag(&tag)
}

#[tauri::command]
pub fn library_get_tags(
    book_id: String,
    db: tauri::State<'_, Box<dyn crate::storage::traits::DatabaseBackend>>,
) -> Result<BookTagsDto, String> {
    let tags_with_groups = db.tags().get_tags_with_groups(&book_id)?;
    let groups = db.tag_groups().list_all()?;

    // Group tags by their group_id
    let mut group_map: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for (tag, group_id) in &tags_with_groups {
        let gid = group_id.clone().unwrap_or_else(|| "default".to_string());
        group_map.entry(gid).or_default().push(tag.clone());
    }

    let mut result_groups = Vec::new();
    for g in &groups {
        let tags = group_map.get(&g.id).cloned().unwrap_or_default();
        if !tags.is_empty() {
            result_groups.push(BookTagGroupDto {
                group_id: g.id.clone(),
                group_name: g.name.clone(),
                color: g.color.clone(),
                tags,
            });
        }
    }

    // Include groups with no tags as well (for the editor UI to show empty groups)
    for g in &groups {
        if !result_groups.iter().any(|rg| rg.group_id == g.id) {
            result_groups.push(BookTagGroupDto {
                group_id: g.id.clone(),
                group_name: g.name.clone(),
                color: g.color.clone(),
                tags: vec![],
            });
        }
    }

    Ok(BookTagsDto {
        book_id,
        groups: result_groups,
    })
}

#[tauri::command]
pub fn library_set_tags(
    book_id: String,
    tags: Vec<String>,
    db: tauri::State<'_, Box<dyn crate::storage::traits::DatabaseBackend>>,
) -> Result<(), String> {
    db.tags().set_tags(&book_id, &tags)
}

#[tauri::command]
pub fn library_all_tags(
    db: tauri::State<'_, Box<dyn crate::storage::traits::DatabaseBackend>>,
) -> Result<Vec<(String, u32)>, String> {
    db.tags().all_tags()
}
