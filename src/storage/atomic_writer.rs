use std::fs;
use std::io::Write;
use std::path::Path;
use std::thread;
use std::time::Duration;

use super::error::StorageError;

/// Maximum number of rename retries before falling back to direct overwrite.
const MAX_RETRIES: u32 = 3;
/// Delay between rename retries.
const RETRY_DELAY: Duration = Duration::from_secs(1);

/// Performs an atomic write to the target path using the temp-file → rename strategy.
///
/// Steps:
/// 1. Write `data` to `<target>.tmp`
/// 2. Flush + sync to disk
/// 3. Rename existing `<target>` → `<target>.bak`
/// 4. Rename `<target>.tmp` → `<target>`
///
/// If the rename fails, retries up to 3 times with 1-second delays,
/// then falls back to a direct overwrite as last resort.
pub fn atomic_write(target: &Path, data: &[u8]) -> Result<(), StorageError> {
    let target_str = target.display().to_string();
    let tmp_path = target.with_extension("json.tmp");
    let bak_path = target.with_extension("json.bak");

    // Step 1 & 2: Write to temp file, flush, and sync
    write_and_sync(&tmp_path, data)?;

    // Step 3: Rename existing file to .bak (if it exists)
    if target.exists() {
        // Remove old .bak if it exists so rename succeeds
        if bak_path.exists() {
            let _ = fs::remove_file(&bak_path);
        }
        if let Err(e) = fs::rename(target, &bak_path) {
            eprintln!(
                "[WARN] [storage] Could not create backup '{}': {}",
                bak_path.display(),
                e
            );
            // Continue anyway — we still want to write the new file
        }
    }

    // Step 4: Rename temp → target, with retries
    for attempt in 0..MAX_RETRIES {
        match fs::rename(&tmp_path, target) {
            Ok(()) => return Ok(()),
            Err(e) => {
                eprintln!(
                    "[WARN] [storage] Rename attempt {}/{} failed for '{}': {}",
                    attempt + 1,
                    MAX_RETRIES,
                    target_str,
                    e
                );
                if attempt + 1 < MAX_RETRIES {
                    thread::sleep(RETRY_DELAY);
                }
            }
        }
    }

    // Fallback: direct overwrite
    eprintln!(
        "[WARN] [storage] All rename attempts failed for '{}'. Attempting direct overwrite.",
        target_str
    );
    match fs::write(target, data) {
        Ok(()) => {
            // Clean up the dangling tmp file
            let _ = fs::remove_file(&tmp_path);
            Ok(())
        }
        Err(e) => {
            // Clean up tmp
            let _ = fs::remove_file(&tmp_path);
            Err(StorageError::FileWriteError {
                path: target_str,
                source: e,
            })
        }
    }
}

/// Writes data to a file, flushes, and syncs to disk.
fn write_and_sync(path: &Path, data: &[u8]) -> Result<(), StorageError> {
    let path_str = path.display().to_string();
    let mut file = fs::File::create(path).map_err(|e| StorageError::FileWriteError {
        path: path_str.clone(),
        source: e,
    })?;
    file.write_all(data)
        .map_err(|e| StorageError::FileWriteError {
            path: path_str.clone(),
            source: e,
        })?;
    file.flush().map_err(|e| StorageError::FileWriteError {
        path: path_str.clone(),
        source: e,
    })?;
    file.sync_all().map_err(|e| StorageError::FileWriteError {
        path: path_str,
        source: e,
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_dir() -> PathBuf {
        let dir = std::env::temp_dir().join("textmacro_atomic_tests");
        let _ = fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn test_atomic_write_creates_new_file() {
        let dir = test_dir();
        let target = dir.join("new_file.json");
        let _ = fs::remove_file(&target);

        let data = b"{\"hello\": \"world\"}";
        atomic_write(&target, data).unwrap();

        assert!(target.exists());
        assert_eq!(
            fs::read_to_string(&target).unwrap(),
            "{\"hello\": \"world\"}"
        );

        // Cleanup
        let _ = fs::remove_file(&target);
    }

    #[test]
    fn test_atomic_write_creates_bak() {
        let dir = test_dir();
        let target = dir.join("bak_test.json");
        let bak = dir.join("bak_test.json.bak");
        let _ = fs::remove_file(&target);
        let _ = fs::remove_file(&bak);

        // Write initial version
        fs::write(&target, b"version1").unwrap();

        // Atomic write of version2
        atomic_write(&target, b"version2").unwrap();

        assert_eq!(fs::read_to_string(&target).unwrap(), "version2");
        assert!(bak.exists());
        assert_eq!(fs::read_to_string(&bak).unwrap(), "version1");

        // Cleanup
        let _ = fs::remove_file(&target);
        let _ = fs::remove_file(&bak);
    }

    #[test]
    fn test_atomic_write_overwrites_old_bak() {
        let dir = test_dir();
        let target = dir.join("multi_bak.json");
        let bak = dir.join("multi_bak.json.bak");
        let _ = fs::remove_file(&target);
        let _ = fs::remove_file(&bak);

        fs::write(&target, b"v1").unwrap();
        atomic_write(&target, b"v2").unwrap();
        assert_eq!(fs::read_to_string(&bak).unwrap(), "v1");

        atomic_write(&target, b"v3").unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "v3");
        assert_eq!(fs::read_to_string(&bak).unwrap(), "v2");

        // Cleanup
        let _ = fs::remove_file(&target);
        let _ = fs::remove_file(&bak);
    }

    #[test]
    fn test_atomic_write_tmp_file_cleaned_up() {
        let dir = test_dir();
        let target = dir.join("tmp_cleanup.json");
        let tmp = dir.join("tmp_cleanup.json.tmp");
        let _ = fs::remove_file(&target);
        let _ = fs::remove_file(&tmp);

        atomic_write(&target, b"data").unwrap();

        assert!(
            !tmp.exists(),
            ".tmp file should be cleaned up after successful write"
        );

        // Cleanup
        let _ = fs::remove_file(&target);
    }
}
