use std::io::Write;
use std::path::Path;

fn write_atomic_impl(path: &Path, data: &str) -> std::io::Result<()> {
    let tmp_path = path.with_extension("json.tmp");
    let mut file = std::fs::File::create(&tmp_path)?;
    file.write_all(data.as_bytes())?;
    file.flush()?;
    file.sync_all()?;
    std::fs::rename(&tmp_path, path)?;
    Ok(())
}

/// Write a serializable value as pretty-printed JSON atomically.
pub fn write_json_atomic<T: serde::Serialize>(path: &Path, value: &T) -> std::io::Result<()> {
    let data = serde_json::to_string_pretty(value).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, e)
    })?;
    write_atomic_impl(path, &data)
}

/// Write a serializable value as compact JSON atomically.
pub fn write_json_atomic_compact<T: serde::Serialize>(path: &Path, value: &T) -> std::io::Result<()> {
    let data = serde_json::to_string(value).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, e)
    })?;
    write_atomic_impl(path, &data)
}
