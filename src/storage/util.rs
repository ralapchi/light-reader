use std::io::Write;
use std::path::Path;

/// Write a serializable value as pretty-printed JSON atomically.
///
/// Writes to a temporary file in the same directory, flushes/syncs to disk,
/// then renames to the target path. This prevents partial writes on crash.
pub fn write_json_atomic<T: serde::Serialize>(path: &Path, value: &T) -> std::io::Result<()> {
    let data = serde_json::to_string_pretty(value).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, e)
    })?;

    let tmp_path = path.with_extension("json.tmp");
    let mut file = std::fs::File::create(&tmp_path)?;
    file.write_all(data.as_bytes())?;
    file.flush()?;
    file.sync_all()?;
    std::fs::rename(&tmp_path, path)?;
    Ok(())
}
