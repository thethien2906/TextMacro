use serde::{Deserialize, Serialize};

/// Application-wide configuration settings.
/// All fields are required with defaults populated via `Default` implementation.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Config {
    /// Launch TextMacro at system startup.
    pub run_on_startup: bool,
    /// Keep macro engine running in background.
    pub enable_background_service: bool,
    /// Default prefix for text triggers.
    pub trigger_prefix: String,
    /// Use clipboard paste for text injection.
    pub clipboard_mode: bool,
    /// UI theme ("dark" or "light").
    pub theme: String,
    /// Layout density ("compact" or "comfortable").
    pub ui_density: String,
    /// Use monospace font in content editor.
    pub editor_font_monospace: bool,
    /// Global default for format preservation.
    pub preserve_formatting: bool,
    /// Enable markdown rendering support.
    pub markdown_support: bool,
    /// Keyboard shortcut for command palette.
    pub command_palette_shortcut: String,
    /// Maximum typing buffer size in characters.
    pub typing_buffer_size: u32,
    /// Auto-dismiss duration for notifications (ms).
    pub notification_duration_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            run_on_startup: false,
            enable_background_service: true,
            trigger_prefix: "/".into(),
            clipboard_mode: true,
            theme: "dark".into(),
            ui_density: "comfortable".into(),
            editor_font_monospace: false,
            preserve_formatting: true,
            markdown_support: true,
            command_palette_shortcut: "Ctrl+Shift+P".into(),
            typing_buffer_size: 100,
            notification_duration_ms: 3000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default_values() {
        let config = Config::default();
        assert_eq!(config.run_on_startup, false);
        assert_eq!(config.enable_background_service, true);
        assert_eq!(config.trigger_prefix, "/");
        assert_eq!(config.clipboard_mode, true);
        assert_eq!(config.theme, "dark");
        assert_eq!(config.ui_density, "comfortable");
        assert_eq!(config.editor_font_monospace, false);
        assert_eq!(config.preserve_formatting, true);
        assert_eq!(config.markdown_support, true);
        assert_eq!(config.command_palette_shortcut, "Ctrl+Shift+P");
        assert_eq!(config.typing_buffer_size, 100);
        assert_eq!(config.notification_duration_ms, 3000);
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = Config::default();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_config_json_field_names() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();

        assert!(json.contains("\"run_on_startup\""));
        assert!(json.contains("\"enable_background_service\""));
        assert!(json.contains("\"trigger_prefix\""));
        assert!(json.contains("\"clipboard_mode\""));
        assert!(json.contains("\"theme\""));
        assert!(json.contains("\"ui_density\""));
        assert!(json.contains("\"editor_font_monospace\""));
        assert!(json.contains("\"preserve_formatting\""));
        assert!(json.contains("\"markdown_support\""));
        assert!(json.contains("\"command_palette_shortcut\""));
        assert!(json.contains("\"typing_buffer_size\""));
        assert!(json.contains("\"notification_duration_ms\""));
    }

    #[test]
    fn test_config_json_matches_expected_format() {
        let config = Config::default();
        let json = serde_json::to_string_pretty(&config).unwrap();

        // Verify it produces the exact expected JSON structure
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["run_on_startup"], false);
        assert_eq!(value["enable_background_service"], true);
        assert_eq!(value["trigger_prefix"], "/");
        assert_eq!(value["clipboard_mode"], true);
        assert_eq!(value["theme"], "dark");
        assert_eq!(value["ui_density"], "comfortable");
        assert_eq!(value["editor_font_monospace"], false);
        assert_eq!(value["preserve_formatting"], true);
        assert_eq!(value["markdown_support"], true);
        assert_eq!(value["command_palette_shortcut"], "Ctrl+Shift+P");
        assert_eq!(value["typing_buffer_size"], 100);
        assert_eq!(value["notification_duration_ms"], 3000);
    }

    #[test]
    fn test_config_custom_values_roundtrip() {
        let config = Config {
            run_on_startup: true,
            enable_background_service: false,
            trigger_prefix: "!".into(),
            clipboard_mode: false,
            theme: "light".into(),
            ui_density: "compact".into(),
            editor_font_monospace: true,
            preserve_formatting: false,
            markdown_support: false,
            command_palette_shortcut: "Ctrl+K".into(),
            typing_buffer_size: 200,
            notification_duration_ms: 5000,
        };

        let json = serde_json::to_string_pretty(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }
}
