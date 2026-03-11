use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::config::Config;
use crate::models::macro_model::Macro;
use crate::models::stats::MacroStats;

use super::atomic_writer;
use super::backup;
use super::error::StorageError;

// ──────────────────────────────  Versioned Wrappers  ──────────────────────────────

/// Top-level JSON envelope for `macros.json`.
#[derive(Debug, Serialize, Deserialize)]
pub struct MacrosFile {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub macros: Vec<serde_json::Value>,
}

/// Top-level JSON envelope for `config.json`.
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigFile {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(flatten)]
    pub config: Config,
}

/// Top-level JSON envelope for `stats.json`.
#[derive(Debug, Serialize, Deserialize)]
pub struct StatsFile {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub stats: Vec<MacroStats>,
}

fn default_version() -> u32 {
    1
}

// ──────────────────────────────  macros.json  ──────────────────────────────

/// Loads macros from a JSON file, applying all validation rules:
/// - Invalid individual macros are skipped (logged)
/// - Duplicate triggers: keep first occurrence, skip rest
/// - Invalid UUIDs: regenerate
/// - Invalid enum values: default via serde defaults
///
/// Returns `(valid_macros, warnings)`.
pub fn load_macros(path: &Path) -> Result<(Vec<Macro>, Vec<String>), StorageError> {
    let path_str = path.display().to_string();
    let content = fs::read_to_string(path).map_err(|e| StorageError::FileReadError {
        path: path_str.clone(),
        source: e,
    })?;

    parse_macros_from_str(&content, &path_str)
}

/// Core macro parsing with all validation / skip / dedup logic.
pub fn parse_macros_from_str(
    content: &str,
    source_path: &str,
) -> Result<(Vec<Macro>, Vec<String>), StorageError> {
    let file: MacrosFile =
        serde_json::from_str(content).map_err(|e| StorageError::ParseError {
            path: source_path.into(),
            message: e.to_string(),
        })?;

    // Placeholder for future migration
    let _version = file.version;

    let mut macros: Vec<Macro> = Vec::new();
    let mut seen_triggers: HashSet<String> = HashSet::new();
    let mut warnings: Vec<String> = Vec::new();

    for (i, raw_value) in file.macros.iter().enumerate() {
        // Try to deserialize with lenient defaults
        let mut m: Macro = match serde_json::from_value(raw_value.clone()) {
            Ok(m) => m,
            Err(e) => {
                warnings.push(format!(
                    "Skipped invalid macro at index {}: {}",
                    i,
                    e
                ));
                continue;
            }
        };

        // Validate / regenerate UUID
        if Uuid::parse_str(&m.id).is_err() {
            let old_id = m.id.clone();
            m.id = Uuid::new_v4().to_string();
            warnings.push(format!(
                "Regenerated invalid UUID '{}' → '{}' for trigger '{}'",
                old_id, m.id, m.trigger
            ));
        }

        // Deduplicate triggers
        if seen_triggers.contains(&m.trigger) {
            warnings.push(format!(
                "Skipped duplicate trigger '{}' (macro id: {})",
                m.trigger, m.id
            ));
            continue;
        }
        seen_triggers.insert(m.trigger.clone());

        macros.push(m);
    }

    Ok((macros, warnings))
}

/// Serializes and saves macros using atomic write.
/// Creates a daily backup on the first save of each day.
pub fn save_macros(path: &Path, macros: &[Macro], data_dir: &Path) -> Result<(), StorageError> {
    let path_str = path.display().to_string();

    #[derive(Serialize)]
    struct Out<'a> {
        version: u32,
        macros: &'a [Macro],
    }

    let out = Out { version: 1, macros };
    let json = serde_json::to_string_pretty(&out).map_err(|e| StorageError::SerializationError {
        message: e.to_string(),
    })?;

    // Daily backup (best-effort)
    if let Err(e) = backup::create_daily_backup(data_dir, path) {
        eprintln!("[WARN] [storage] Daily backup failed: {}", e);
    }

    atomic_writer::atomic_write(path, json.as_bytes()).map_err(|_| StorageError::FileWriteError {
        path: path_str.clone(),
        source: std::io::Error::new(std::io::ErrorKind::Other, "Atomic write failed"),
    })?;

    Ok(())
}

// ──────────────────────────────  config.json  ──────────────────────────────

/// Loads config from a JSON file.
/// If the file is missing or corrupt, returns `Config::default()` and a warning.
pub fn load_config(path: &Path) -> (Config, Vec<String>) {
    let mut warnings = Vec::new();

    if !path.exists() {
        warnings.push(format!(
            "Config file '{}' not found, using defaults",
            path.display()
        ));
        return (Config::default(), warnings);
    }

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            warnings.push(format!(
                "Cannot read config '{}': {}. Using defaults.",
                path.display(),
                e
            ));
            return (Config::default(), warnings);
        }
    };

    match serde_json::from_str::<ConfigFile>(&content) {
        Ok(cf) => (cf.config, warnings),
        Err(e) => {
            warnings.push(format!(
                "Config '{}' is corrupt: {}. Using defaults.",
                path.display(),
                e
            ));
            (Config::default(), warnings)
        }
    }
}

/// Saves config using atomic write.
pub fn save_config(path: &Path, config: &Config) -> Result<(), StorageError> {
    let path_str = path.display().to_string();

    #[derive(Serialize)]
    struct Out<'a> {
        version: u32,
        #[serde(flatten)]
        config: &'a Config,
    }

    let out = Out { version: 1, config };
    let json = serde_json::to_string_pretty(&out).map_err(|e| StorageError::SerializationError {
        message: e.to_string(),
    })?;

    atomic_writer::atomic_write(path, json.as_bytes()).map_err(|_| StorageError::FileWriteError {
        path: path_str,
        source: std::io::Error::new(std::io::ErrorKind::Other, "Atomic write failed"),
    })?;

    Ok(())
}

// ──────────────────────────────  stats.json  ──────────────────────────────

/// Loads stats from a JSON file.
/// If the file does not exist, returns an empty vector.
pub fn load_stats(path: &Path) -> (Vec<MacroStats>, Vec<String>) {
    let mut warnings = Vec::new();

    if !path.exists() {
        return (Vec::new(), warnings);
    }

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            warnings.push(format!(
                "Cannot read stats '{}': {}. Starting with empty stats.",
                path.display(),
                e
            ));
            return (Vec::new(), warnings);
        }
    };

    match serde_json::from_str::<StatsFile>(&content) {
        Ok(sf) => (sf.stats, warnings),
        Err(e) => {
            warnings.push(format!(
                "Stats '{}' is corrupt: {}. Starting with empty stats.",
                path.display(),
                e
            ));
            (Vec::new(), warnings)
        }
    }
}

/// Saves stats using atomic write.
pub fn save_stats(path: &Path, stats: &[MacroStats]) -> Result<(), StorageError> {
    let path_str = path.display().to_string();

    #[derive(Serialize)]
    struct Out<'a> {
        version: u32,
        stats: &'a [MacroStats],
    }

    let out = Out { version: 1, stats };
    let json = serde_json::to_string_pretty(&out).map_err(|e| StorageError::SerializationError {
        message: e.to_string(),
    })?;

    atomic_writer::atomic_write(path, json.as_bytes()).map_err(|_| StorageError::FileWriteError {
        path: path_str,
        source: std::io::Error::new(std::io::ErrorKind::Other, "Atomic write failed"),
    })?;

    Ok(())
}

// ──────────────────────────────  Defaults  ──────────────────────────────

/// Returns the default content for an empty `macros.json`.
pub fn default_macros_json() -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "version": 1,
        "macros": []
    }))
    .unwrap()
}

/// Returns the default content for `config.json`.
pub fn default_config_json() -> String {
    let config = Config::default();
    let value = serde_json::json!({
        "version": 1,
        "run_on_startup": config.run_on_startup,
        "enable_background_service": config.enable_background_service,
        "trigger_prefix": config.trigger_prefix,
        "clipboard_mode": config.clipboard_mode,
        "theme": config.theme,
        "ui_density": config.ui_density,
        "editor_font_monospace": config.editor_font_monospace,
        "preserve_formatting": config.preserve_formatting,
        "markdown_support": config.markdown_support,
        "command_palette_shortcut": config.command_palette_shortcut,
        "typing_buffer_size": config.typing_buffer_size,
        "notification_duration_ms": config.notification_duration_ms,
    });
    serde_json::to_string_pretty(&value).unwrap()
}

// ──────────────────────────────  Migration Placeholder  ──────────────────────────────

/// Migration function placeholder. In Phase 2 this is a no-op.
/// Future versions will match on `from_version` and transform data accordingly.
#[allow(dead_code)]
pub fn migrate(_data: serde_json::Value, _from_version: u32) -> serde_json::Value {
    // No-op: return data unchanged
    _data
}

// ─────────────────────────────────  Tests  ─────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::models::macro_model::{ActionType, EventTrigger, EventType, MacroCategory};

    fn test_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join("textmacro_json_tests")
            .join(name);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::create_dir_all(dir.join("backups")).unwrap();
        dir
    }

    // ── macros.json ──

    #[test]
    fn test_load_valid_macros() {
        let dir = test_dir("load_valid");
        let path = dir.join("macros.json");
        let json = r#"{
            "version": 1,
            "macros": [
                {
                    "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                    "trigger": "/sig",
                    "description": "Email signature",
                    "content": "Best regards,\nJohn Doe",
                    "enabled": true,
                    "category": "text",
                    "action_type": "insert_text",
                    "preserve_format": true,
                    "created_at": "2026-01-15T10:30:00Z",
                    "updated_at": "2026-01-15T10:30:00Z",
                    "tags": ["email"],
                    "shortcut": null,
                    "event_trigger": null
                }
            ]
        }"#;
        fs::write(&path, json).unwrap();

        let (macros, warnings) = load_macros(&path).unwrap();
        assert_eq!(macros.len(), 1);
        assert_eq!(macros[0].trigger, "/sig");
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_load_macros_skips_invalid_entries() {
        let dir = test_dir("skip_invalid");
        let path = dir.join("macros.json");
        // Second entry has no trigger (required field)
        let json = r#"{
            "version": 1,
            "macros": [
                {
                    "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                    "trigger": "/good",
                    "content": "works",
                    "created_at": "2026-01-01T00:00:00Z",
                    "updated_at": "2026-01-01T00:00:00Z"
                },
                {
                    "id": "bad-entry",
                    "content": "no trigger field"
                }
            ]
        }"#;
        fs::write(&path, json).unwrap();

        let (macros, warnings) = load_macros(&path).unwrap();
        assert_eq!(macros.len(), 1);
        assert_eq!(macros[0].trigger, "/good");
        assert!(!warnings.is_empty());
    }

    #[test]
    fn test_load_macros_dedup_triggers() {
        let dir = test_dir("dedup");
        let path = dir.join("macros.json");
        let json = r#"{
            "version": 1,
            "macros": [
                {
                    "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                    "trigger": "/dup",
                    "content": "first",
                    "created_at": "2026-01-01T00:00:00Z",
                    "updated_at": "2026-01-01T00:00:00Z"
                },
                {
                    "id": "b2c3d4e5-f6a7-8901-bcde-f12345678901",
                    "trigger": "/dup",
                    "content": "second",
                    "created_at": "2026-01-01T00:00:00Z",
                    "updated_at": "2026-01-01T00:00:00Z"
                }
            ]
        }"#;
        fs::write(&path, json).unwrap();

        let (macros, warnings) = load_macros(&path).unwrap();
        assert_eq!(macros.len(), 1);
        assert_eq!(macros[0].content, "first");
        assert!(warnings.iter().any(|w| w.contains("duplicate trigger")));
    }

    #[test]
    fn test_load_macros_regenerates_invalid_uuid() {
        let dir = test_dir("bad_uuid");
        let path = dir.join("macros.json");
        let json = r#"{
            "version": 1,
            "macros": [
                {
                    "id": "NOT-A-VALID-UUID",
                    "trigger": "/test",
                    "content": "hello",
                    "created_at": "2026-01-01T00:00:00Z",
                    "updated_at": "2026-01-01T00:00:00Z"
                }
            ]
        }"#;
        fs::write(&path, json).unwrap();

        let (macros, warnings) = load_macros(&path).unwrap();
        assert_eq!(macros.len(), 1);
        assert_ne!(macros[0].id, "NOT-A-VALID-UUID");
        // The new ID should be a valid UUID
        assert!(Uuid::parse_str(&macros[0].id).is_ok());
        assert!(warnings.iter().any(|w| w.contains("Regenerated invalid UUID")));
    }

    #[test]
    fn test_load_macros_empty_file() {
        let dir = test_dir("empty_macros");
        let path = dir.join("macros.json");
        fs::write(&path, default_macros_json()).unwrap();

        let (macros, warnings) = load_macros(&path).unwrap();
        assert!(macros.is_empty());
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_load_macros_missing_version_defaults_to_1() {
        let dir = test_dir("no_version");
        let path = dir.join("macros.json");
        let json = r#"{
            "macros": [
                {
                    "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                    "trigger": "/test",
                    "content": "hello",
                    "created_at": "2026-01-01T00:00:00Z",
                    "updated_at": "2026-01-01T00:00:00Z"
                }
            ]
        }"#;
        fs::write(&path, json).unwrap();

        let (macros, _) = load_macros(&path).unwrap();
        assert_eq!(macros.len(), 1);
    }

    #[test]
    fn test_save_macros_roundtrip() {
        let dir = test_dir("save_roundtrip");
        let path = dir.join("macros.json");
        let macros = vec![
            Macro::new("/sig".into(), "Best regards".into()),
            Macro::new("/addr".into(), "123 Main St".into()),
        ];

        save_macros(&path, &macros, &dir).unwrap();
        let (loaded, warnings) = load_macros(&path).unwrap();

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].trigger, "/sig");
        assert_eq!(loaded[1].trigger, "/addr");
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_save_macros_pretty_printed() {
        let dir = test_dir("pretty_print");
        let path = dir.join("macros.json");
        let macros = vec![Macro::new("/test".into(), "data".into())];

        save_macros(&path, &macros, &dir).unwrap();
        let content = fs::read_to_string(&path).unwrap();

        // Pretty printed means indentation
        assert!(content.contains("  "));
        assert!(content.contains('\n'));
    }

    #[test]
    fn test_save_macros_includes_version() {
        let dir = test_dir("version_field");
        let path = dir.join("macros.json");
        let macros = vec![Macro::new("/x".into(), "y".into())];

        save_macros(&path, &macros, &dir).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        let value: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert_eq!(value["version"], 1);
    }

    // ── config.json ──

    #[test]
    fn test_load_config_valid() {
        let dir = test_dir("config_valid");
        let path = dir.join("config.json");
        fs::write(&path, default_config_json()).unwrap();

        let (config, warnings) = load_config(&path);
        assert_eq!(config, Config::default());
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_load_config_missing_returns_defaults() {
        let dir = test_dir("config_missing");
        let path = dir.join("config.json");

        let (config, warnings) = load_config(&path);
        assert_eq!(config, Config::default());
        assert!(!warnings.is_empty());
    }

    #[test]
    fn test_load_config_corrupt_returns_defaults() {
        let dir = test_dir("config_corrupt");
        let path = dir.join("config.json");
        fs::write(&path, "NOT VALID JSON").unwrap();

        let (config, warnings) = load_config(&path);
        assert_eq!(config, Config::default());
        assert!(warnings.iter().any(|w| w.contains("corrupt")));
    }

    #[test]
    fn test_save_config_roundtrip() {
        let dir = test_dir("config_roundtrip");
        let path = dir.join("config.json");
        let config = Config {
            theme: "light".into(),
            typing_buffer_size: 200,
            ..Config::default()
        };

        save_config(&path, &config).unwrap();
        let (loaded, _) = load_config(&path);
        assert_eq!(loaded.theme, "light");
        assert_eq!(loaded.typing_buffer_size, 200);
    }

    #[test]
    fn test_save_config_includes_version() {
        let dir = test_dir("config_version");
        let path = dir.join("config.json");

        save_config(&path, &Config::default()).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        let value: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(value["version"], 1);
    }

    // ── stats.json ──

    #[test]
    fn test_load_stats_missing_returns_empty() {
        let dir = test_dir("stats_missing");
        let path = dir.join("stats.json");

        let (stats, warnings) = load_stats(&path);
        assert!(stats.is_empty());
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_save_stats_roundtrip() {
        let dir = test_dir("stats_roundtrip");
        let path = dir.join("stats.json");
        let stats = vec![
            MacroStats {
                macro_id: "test-id".into(),
                trigger_count: 42,
                last_triggered: Some("2026-03-09T14:20:00Z".into()),
            },
        ];

        save_stats(&path, &stats).unwrap();
        let (loaded, warnings) = load_stats(&path);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].trigger_count, 42);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_save_stats_includes_version() {
        let dir = test_dir("stats_version");
        let path = dir.join("stats.json");

        save_stats(&path, &[]).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        let value: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(value["version"], 1);
    }

    // ── Default content ──

    #[test]
    fn test_default_macros_json_is_valid() {
        let json = default_macros_json();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["version"], 1);
        assert!(value["macros"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_default_config_json_is_valid() {
        let json = default_config_json();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["version"], 1);
        assert_eq!(value["theme"], "dark");
        assert_eq!(value["trigger_prefix"], "/");
    }

    // ── Migration placeholder ──

    #[test]
    fn test_migrate_is_noop() {
        let data = serde_json::json!({"version": 1, "macros": []});
        let result = migrate(data.clone(), 1);
        assert_eq!(data, result);
    }
}
