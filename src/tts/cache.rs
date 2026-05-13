use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

pub struct TtsCache {
    base_dir: PathBuf,
}

#[allow(dead_code)]
impl TtsCache {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Compute a stable cache key from synthesis parameters.
    /// Used for dedup detection when voice/format changes.
    pub fn cache_key(voice_id: &str, format: &str, text: &str) -> String {
        let mut hasher = DefaultHasher::new();
        voice_id.hash(&mut hasher);
        format.hash(&mut hasher);
        text.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
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
            .join(format!("{}_{}.{}", segment_index, sanitize_filename(voice_id), ext))
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

    /// Remove cached audio for a specific book across all providers.
    pub fn clear_book(&self, book_id: &str) -> std::io::Result<()> {
        let sanitized = sanitize_filename(book_id);
        if self.base_dir.exists() {
            for entry in std::fs::read_dir(&self.base_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    let book_dir = entry.path().join(&sanitized);
                    if book_dir.exists() {
                        std::fs::remove_dir_all(&book_dir)?;
                    }
                }
            }
        }
        Ok(())
    }
}

/// Replace non-alphanumeric characters for safe filesystem usage.
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_is_deterministic() {
        let a = TtsCache::cache_key("voice1", "mp3", "hello");
        let b = TtsCache::cache_key("voice1", "mp3", "hello");
        assert_eq!(a, b);
    }

    #[test]
    fn cache_key_differs_for_different_voice() {
        let a = TtsCache::cache_key("voice1", "mp3", "hello");
        let b = TtsCache::cache_key("voice2", "mp3", "hello");
        assert_ne!(a, b);
    }

    #[test]
    fn cache_key_differs_for_different_text() {
        let a = TtsCache::cache_key("v1", "mp3", "hello");
        let b = TtsCache::cache_key("v1", "mp3", "world");
        assert_ne!(a, b);
    }

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
