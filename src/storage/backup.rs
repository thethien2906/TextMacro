use std::fs;
use std::path::{Path, PathBuf};

use chrono::{NaiveDate, Utc};

use super::paths;

/// Creates a daily backup of `source_file` into the `backups/` directory.
///
/// Naming convention: `macros_YYYY-MM-DD.json`
///
/// Only creates a backup once per day (skips if today's file already exists).
/// After creating a backup, runs retention to keep only the last 7 daily backups.
pub fn create_daily_backup(data_dir: &Path, source_file: &Path) -> Result<(), String> {
    let backups = paths::backups_dir(&data_dir.to_path_buf());
    if !backups.exists() {
        fs::create_dir_all(&backups).map_err(|e| format!("Cannot create backups dir: {}", e))?;
    }

    let today = Utc::now().format("%Y-%m-%d").to_string();
    let stem = source_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("macros");
    let backup_name = format!("{}_{}.json", stem, today);
    let backup_path = backups.join(&backup_name);

    // Only back up once per day
    if backup_path.exists() {
        return Ok(());
    }

    if source_file.exists() {
        fs::copy(source_file, &backup_path)
            .map_err(|e| format!("Failed to create daily backup '{}': {}", backup_name, e))?;
    }

    // Run retention
    if let Err(e) = enforce_retention(&backups, stem, 7) {
        eprintln!("[WARN] [storage] Backup retention cleanup failed: {}", e);
    }

    Ok(())
}

/// Deletes daily backups older than `keep_count` days.
///
/// Scans the backups directory for files matching `<stem>_YYYY-MM-DD.json`,
/// keeps the newest `keep_count` files, and deletes the rest.
fn enforce_retention(backups_dir: &Path, stem: &str, keep_count: usize) -> Result<(), String> {
    let prefix = format!("{}_", stem);
    let mut dated_files: Vec<(NaiveDate, PathBuf)> = Vec::new();

    let entries = fs::read_dir(backups_dir)
        .map_err(|e| format!("Cannot read backups dir: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with(&prefix) && name.ends_with(".json") {
                // Extract <YYYY-MM-DD> from <stem>_YYYY-MM-DD.json
                let date_part = &name[prefix.len()..name.len() - 5]; // strip ".json"
                if let Ok(date) = NaiveDate::parse_from_str(date_part, "%Y-%m-%d") {
                    dated_files.push((date, path));
                }
            }
        }
    }

    // Sort by date descending (newest first)
    dated_files.sort_by(|a, b| b.0.cmp(&a.0));

    // Delete everything past `keep_count`
    for (_date, path) in dated_files.iter().skip(keep_count) {
        if let Err(e) = fs::remove_file(path) {
            eprintln!(
                "[WARN] [storage] Failed to delete old backup '{}': {}",
                path.display(),
                e
            );
        }
    }

    Ok(())
}

/// Attempts to recover from daily backups when `macros.json` and `.bak` are both invalid.
///
/// Scans `backups/` for files matching `macros_YYYY-MM-DD.json`,
/// tries the newest first, and returns the content of the first valid one.
pub fn find_newest_valid_backup(data_dir: &Path) -> Option<String> {
    let backups = paths::backups_dir(&data_dir.to_path_buf());
    if !backups.exists() {
        return None;
    }

    let prefix = "macros_";
    let mut dated_files: Vec<(NaiveDate, PathBuf)> = Vec::new();

    if let Ok(entries) = fs::read_dir(&backups) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with(prefix) && name.ends_with(".json") {
                    let date_part = &name[prefix.len()..name.len() - 5];
                    if let Ok(date) = NaiveDate::parse_from_str(date_part, "%Y-%m-%d") {
                        dated_files.push((date, path));
                    }
                }
            }
        }
    }

    // Sort newest first
    dated_files.sort_by(|a, b| b.0.cmp(&a.0));

    for (_date, path) in &dated_files {
        if let Ok(content) = fs::read_to_string(path) {
            // Quick validity check: must be parseable JSON
            if serde_json::from_str::<serde_json::Value>(&content).is_ok() {
                return Some(content);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("textmacro_backup_tests")
            .join(name);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        // Create backups subdir
        fs::create_dir_all(dir.join("backups")).unwrap();
        dir
    }

    #[test]
    fn test_create_daily_backup_creates_file() {
        let dir = test_dir("create_daily");
        let macros_path = dir.join("macros.json");
        fs::write(&macros_path, r#"{"version":1,"macros":[]}"#).unwrap();

        let result = create_daily_backup(&dir, &macros_path);
        assert!(result.is_ok());

        // Verify backup was created
        let backups = dir.join("backups");
        let entries: Vec<_> = fs::read_dir(&backups)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(entries.len(), 1);

        let name = entries[0].file_name().to_string_lossy().to_string();
        assert!(name.starts_with("macros_"));
        assert!(name.ends_with(".json"));
    }

    #[test]
    fn test_create_daily_backup_idempotent() {
        let dir = test_dir("idempotent");
        let macros_path = dir.join("macros.json");
        fs::write(&macros_path, r#"{"version":1,"macros":[]}"#).unwrap();

        create_daily_backup(&dir, &macros_path).unwrap();
        create_daily_backup(&dir, &macros_path).unwrap();

        // Should still be just 1 backup for today
        let backups = dir.join("backups");
        let entries: Vec<_> = fs::read_dir(&backups)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_retention_keeps_only_newest() {
        let dir = test_dir("retention");
        let backups = dir.join("backups");

        // Create 10 fake backup files with different dates
        for day in 1..=10 {
            let date = format!("2026-03-{:02}", day);
            let name = format!("macros_{}.json", date);
            fs::write(backups.join(&name), r#"{"version":1,"macros":[]}"#).unwrap();
        }

        enforce_retention(&backups, "macros", 7).unwrap();

        let entries: Vec<_> = fs::read_dir(&backups)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(entries.len(), 7);
    }

    #[test]
    fn test_find_newest_valid_backup() {
        let dir = test_dir("find_newest");
        let backups = dir.join("backups");

        // Older valid backup
        fs::write(
            backups.join("macros_2026-03-01.json"),
            r#"{"version":1,"macros":[{"trigger":"/old"}]}"#,
        )
        .unwrap();
        // Newer valid backup
        fs::write(
            backups.join("macros_2026-03-05.json"),
            r#"{"version":1,"macros":[{"trigger":"/new"}]}"#,
        )
        .unwrap();
        // Newest but corrupt
        fs::write(
            backups.join("macros_2026-03-08.json"),
            "NOT VALID JSON!!!",
        )
        .unwrap();

        let result = find_newest_valid_backup(&dir);
        assert!(result.is_some());
        let content = result.unwrap();
        // Should return the 2026-03-05 backup (newest valid)
        assert!(content.contains("/new"));
    }

    #[test]
    fn test_find_newest_valid_backup_no_backups() {
        let dir = test_dir("no_backups");
        // backups dir exists but is empty
        let result = find_newest_valid_backup(&dir);
        assert!(result.is_none());
    }
}
