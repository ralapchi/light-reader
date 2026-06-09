use std::path::{Path, PathBuf};

/// Maximum TTS cache size in bytes (500 MB).
const MAX_CACHE_BYTES: u64 = 500 * 1024 * 1024;

pub struct TtsCache {
    base_dir: PathBuf,
}

impl TtsCache {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Compute the on-disk path for a cached audio segment.
    /// Structure: cache/tts/{provider}/{book_id}/{ch}/{seg}_{voice_id}.{ext}
    pub fn segment_path(
        &self,
        provider: &str,
        book_id: &str,
        chapter_index: usize,
        segment_index: usize,
        voice_id: &str,
        ext: &str,
    ) -> PathBuf {
        self.base_dir
            .join(provider)
            .join(sanitize_filename(book_id))
            .join(chapter_index.to_string())
            .join(format!(
                "{}_{}.{}",
                segment_index,
                sanitize_filename(voice_id),
                ext
            ))
    }

    pub fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    pub fn read(&self, path: &Path) -> std::io::Result<Vec<u8>> {
        std::fs::read(path)
    }

    pub fn write(&self, path: &Path, data: &[u8]) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, data)
    }

    /// Remove all TTS cache contents.
    pub fn clear_all(&self) -> std::io::Result<()> {
        if self.base_dir.exists() {
            std::fs::remove_dir_all(&self.base_dir)?;
            std::fs::create_dir_all(&self.base_dir)?;
        }
        Ok(())
    }

    /// Evict oldest cache files if total size exceeds `MAX_CACHE_BYTES`.
    pub fn prune_if_over_limit(&self) {
        let _ = self.prune_inner();
    }

    fn prune_inner(&self) -> std::io::Result<()> {
        if !self.base_dir.exists() {
            return Ok(());
        }

        let mut entries: Vec<(PathBuf, u64, std::time::SystemTime)> = Vec::new();
        collect_files(&self.base_dir, &mut entries)?;

        let total: u64 = entries.iter().map(|(_, size, _)| *size).sum();
        if total <= MAX_CACHE_BYTES {
            return Ok(());
        }

        // Oldest first
        entries.sort_by_key(|(_, _, mtime)| *mtime);

        let mut to_free = total - MAX_CACHE_BYTES;
        for (path, size, _) in &entries {
            if to_free == 0 {
                break;
            }
            let _ = std::fs::remove_file(path);
            to_free = to_free.saturating_sub(*size);
        }

        // Clean up empty directories
        clean_empty_dirs(&self.base_dir);
        Ok(())
    }
}

fn collect_files(
    dir: &Path,
    out: &mut Vec<(PathBuf, u64, std::time::SystemTime)>,
) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, out)?;
        } else if let Ok(meta) = path.metadata() {
            let mtime = meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            out.push((path, meta.len(), mtime));
        }
    }
    Ok(())
}

fn clean_empty_dirs(dir: &Path) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            clean_empty_dirs(&path);
            // Remove if empty (ignore errors for non-empty dirs)
            let _ = std::fs::remove_dir(&path);
        }
    }
}

/// Replace non-alphanumeric characters for safe filesystem usage.
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_path_uses_hierarchy() {
        let cache = TtsCache::new(PathBuf::from("/tmp/tts-cache"));
        let path = cache.segment_path("xiaomi", "book-123", 0, 1, "female", "pcm16");
        let expected = PathBuf::from("/tmp/tts-cache/xiaomi/book-123/0/1_female.pcm16");
        assert_eq!(path, expected);
    }

    #[test]
    fn sanizite_filename_replaces_special_chars() {
        assert_eq!(sanitize_filename("book-123"), "book-123");
        assert_eq!(sanitize_filename("test/path"), "test_path");
        assert_eq!(sanitize_filename("a:b"), "a_b");
    }

    #[test]
    fn segment_path_differs_for_different_providers() {
        let cache = TtsCache::new(PathBuf::from("/tmp/tts-cache"));
        let a = cache.segment_path("xiaomi", "book-1", 0, 0, "default", "pcm16");
        let b = cache.segment_path("aliyun", "book-1", 0, 0, "default", "pcm16");
        assert_ne!(a, b);
        assert!(a.to_string_lossy().contains("xiaomi"));
        assert!(b.to_string_lossy().contains("aliyun"));
    }

    #[test]
    fn segment_path_includes_all_indices() {
        let cache = TtsCache::new(PathBuf::from("/tmp/tts-cache"));
        let path = cache.segment_path("xiaomi", "book-42", 5, 3, "male", "mp3");
        let s = path.to_string_lossy();
        assert!(s.contains("/book-42/"), "missing book_id: {}", s);
        assert!(s.contains("/5/"), "missing chapter_index: {}", s);
        assert!(s.contains("3_male.mp3"), "missing segment/voice/ext: {}", s);
    }

    #[test]
    fn write_and_read_roundtrip() -> std::io::Result<()> {
        let tmp = std::env::temp_dir().join("reader-tts-cache-test");
        let _ = std::fs::remove_dir_all(&tmp);

        let cache = TtsCache::new(tmp.clone());
        let path = cache.segment_path("test", "book", 0, 0, "default", "pcm");
        assert!(!cache.exists(&path));

        cache.write(&path, b"audio-data")?;
        assert!(cache.exists(&path));

        let data = cache.read(&path)?;
        assert_eq!(data, b"audio-data");

        // Cleanup
        let _ = std::fs::remove_dir_all(&tmp);
        Ok(())
    }
}
