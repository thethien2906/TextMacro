use std::path::PathBuf;

use super::error::StorageError;

/// File names used by the storage layer.
pub const MACROS_FILE: &str = "macros.json";
pub const CONFIG_FILE: &str = "config.json";
pub const STATS_FILE: &str = "stats.json";
pub const LOGS_DIR: &str = "logs";
pub const BACKUPS_DIR: &str = "backups";

/// Resolves the platform-specific data directory for TextMacro.
///
/// | OS      | Path                                       |
/// | ------- | ------------------------------------------ |
/// | Windows | `%APPDATA%\TextMacro\`                     |
/// | Linux   | `~/.textmacro/`                            |
/// | macOS   | `~/Library/Application Support/TextMacro/` |
pub fn resolve_data_dir() -> Result<PathBuf, StorageError> {
    let base = data_dir_base()?;
    Ok(base)
}

fn data_dir_base() -> Result<PathBuf, StorageError> {
    #[cfg(target_os = "windows")]
    {
        // %APPDATA%\TextMacro\
        if let Ok(appdata) = std::env::var("APPDATA") {
            return Ok(PathBuf::from(appdata).join("TextMacro"));
        }
        // Fallback: use home dir
        if let Some(home) = home_dir() {
            return Ok(home.join("AppData").join("Roaming").join("TextMacro"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = home_dir() {
            return Ok(home.join("Library").join("Application Support").join("TextMacro"));
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(home) = home_dir() {
            return Ok(home.join(".textmacro"));
        }
    }

    // Ultimate fallback for other/unknown OS
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        if let Some(home) = home_dir() {
            return Ok(home.join(".textmacro"));
        }
    }

    Err(StorageError::DirectoryCreateFail {
        path: "<unknown>".into(),
        source: std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Cannot determine home directory",
        ),
    })
}

/// Cross-platform home directory resolution.
fn home_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE").ok().map(PathBuf::from)
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
}

/// Returns the path to a specific file inside the data directory.
pub fn data_file(data_dir: &PathBuf, file_name: &str) -> PathBuf {
    data_dir.join(file_name)
}

/// Returns the path to the backups subdirectory.
pub fn backups_dir(data_dir: &PathBuf) -> PathBuf {
    data_dir.join(BACKUPS_DIR)
}

/// Returns the path to the logs subdirectory.
pub fn logs_dir(data_dir: &PathBuf) -> PathBuf {
    data_dir.join(LOGS_DIR)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_data_dir_returns_ok() {
        let result = resolve_data_dir();
        assert!(result.is_ok(), "resolve_data_dir should succeed on any supported OS");
        let path = result.unwrap();
        let path_str = path.to_string_lossy().to_lowercase();
        assert!(
            path_str.contains("textmacro"),
            "Data dir should contain 'textmacro': {}",
            path.display()
        );
    }

    #[test]
    fn test_data_file_path() {
        let dir = PathBuf::from("/tmp/test_data");
        assert_eq!(data_file(&dir, MACROS_FILE), PathBuf::from("/tmp/test_data/macros.json"));
        assert_eq!(data_file(&dir, CONFIG_FILE), PathBuf::from("/tmp/test_data/config.json"));
        assert_eq!(data_file(&dir, STATS_FILE), PathBuf::from("/tmp/test_data/stats.json"));
    }

    #[test]
    fn test_backups_dir_path() {
        let dir = PathBuf::from("/tmp/test_data");
        assert_eq!(backups_dir(&dir), PathBuf::from("/tmp/test_data/backups"));
    }

    #[test]
    fn test_logs_dir_path() {
        let dir = PathBuf::from("/tmp/test_data");
        assert_eq!(logs_dir(&dir), PathBuf::from("/tmp/test_data/logs"));
    }
}
