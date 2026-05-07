use std::collections::HashMap;
use eframe::egui;
use log::warn;

use crate::storage;

/// Manages decoded egui textures for cover and content images.
/// Loads from disk cache, decodes on first access, caches texture handles.
pub struct ImageCache {
    textures: HashMap<String, egui::TextureHandle>,
}

impl ImageCache {
    pub fn new() -> Self {
        Self { textures: HashMap::new() }
    }

    /// Get or load a cover texture for the given book_id.
    /// Returns the texture handle if a cached cover image exists and decodes successfully.
    pub fn cover_texture(
        &mut self,
        ctx: &egui::Context,
        book_id: &str,
        cache_key: Option<&str>,
    ) -> Option<egui::TextureHandle> {
        let key = cache_key?;
        if let Some(handle) = self.textures.get(key) {
            return Some(handle.clone());
        }
        // Try known extensions
        for ext in &["png", "jpg", "jpeg", "webp"] {
            let path = storage::paths::cover_cache_path(book_id, ext);
            if path.exists() {
                match load_texture_from_path(ctx, &path) {
                    Ok(handle) => {
                        self.textures.insert(key.to_string(), handle.clone());
                        return Some(handle);
                    }
                    Err(e) => {
                        warn!("封面解码失败 (book_id={}, ext={}): {}", book_id, ext, e);
                    }
                }
            }
        }
        None
    }

    /// Get or load an inline image texture.
    pub fn image_texture(
        &mut self,
        ctx: &egui::Context,
        book_id: &str,
        asset_id: &str,
    ) -> Option<egui::TextureHandle> {
        let cache_key = format!("{}-{}", book_id, asset_id);
        if let Some(handle) = self.textures.get(&cache_key) {
            return Some(handle.clone());
        }
        // Try known extensions
        for ext in &["png", "jpg", "jpeg", "webp", "gif"] {
            let path = storage::paths::image_cache_path(asset_id, ext);
            if path.exists() {
                match load_texture_from_path(ctx, &path) {
                    Ok(handle) => {
                        self.textures.insert(cache_key.clone(), handle.clone());
                        return Some(handle);
                    }
                    Err(e) => {
                        warn!("图片解码失败 (asset={}, path={}): {}", asset_id, path.display(), e);
                    }
                }
            }
        }
        warn!("图片缓存未找到: asset_id={}, 已检查 png/jpg/jpeg/webp/gif", asset_id);
        None
    }

    /// Clear all cached textures (e.g., on memory pressure or book close).
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.textures.clear();
    }
}

/// Load image bytes from disk and decode into an egui texture.
fn load_texture_from_path(
    ctx: &egui::Context,
    path: &std::path::Path,
) -> Result<egui::TextureHandle, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read: {}", e))?;
    let image = image::load_from_memory(&bytes).map_err(|e| format!("decode: {}", e))?;
    let rgba = image.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
    let handle = ctx.load_texture(
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("cover")
            .to_string(),
        color_image,
        egui::TextureOptions::default(),
    );
    Ok(handle)
}
