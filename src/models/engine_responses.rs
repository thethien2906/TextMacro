use serde::{Deserialize, Serialize};

use super::config::Config;
use super::macro_model::Macro;
use super::stats::MacroStats;

/// Message types sent from engine to UI.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EngineResponse {
    /// List of macros for a category.
    MacroList(Vec<Macro>),
    /// A single macro detail.
    MacroDetail(Macro),
    /// A newly created macro.
    MacroCreated(Macro),
    /// An updated macro.
    MacroUpdated(Macro),
    /// ID of the deleted macro.
    MacroDeleted(String),
    /// Toggled macro: (id, new_state).
    MacroToggled(String, bool),
    /// Search results.
    SearchResults(Vec<Macro>),
    /// Result of macro execution.
    MacroExecuted(ExecutionResult),
    /// Current configuration.
    ConfigLoaded(Config),
    /// Updated configuration.
    ConfigUpdated(Config),
    /// Import operation result.
    ImportComplete(ImportResult),
    /// Export operation result.
    ExportComplete(ExportResult),
    /// Macros have been reloaded from storage.
    MacrosReloaded,
    /// Stats for a macro.
    StatsLoaded(MacroStats),
    /// An error occurred.
    Error(EngineError),
}

/// Result of a macro execution.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ExecutionResult {
    /// ID of the executed macro.
    pub macro_id: String,
    /// Whether the execution was successful.
    pub success: bool,
    /// Description of what was done.
    pub action: String,
    /// ISO 8601 timestamp.
    pub timestamp: String,
}

/// Result of an import operation.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ImportResult {
    /// Number of macros successfully imported.
    pub imported_count: u32,
    /// Number of macros skipped (duplicates, etc.).
    pub skipped_count: u32,
    /// Error messages for failed imports.
    pub errors: Vec<String>,
}

/// Result of an export operation.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ExportResult {
    /// Number of macros exported.
    pub exported_count: u32,
    /// Path to the exported file.
    pub file_path: String,
}

/// Engine error with machine-readable code and human-readable message.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EngineError {
    /// Machine-readable error code.
    pub code: String,
    /// Human-readable description.
    pub message: String,
}

/// Unsolicited events from engine to UI.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EngineEvent {
    /// A macro was triggered: (macro_id, trigger, timestamp).
    MacroTriggered {
        macro_id: String,
        trigger: String,
        timestamp: String,
    },
    /// A macro execution completed: (macro_id, success).
    MacroExecutionComplete { macro_id: String, success: bool },
    /// Engine has started.
    EngineStarted,
    /// Engine has stopped.
    EngineStopped,
    /// A storage error occurred.
    StorageError(String),
    /// Configuration has changed.
    ConfigChanged(Config),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::macro_model::{ActionType, Macro, MacroCategory};

    #[test]
    fn test_execution_result_roundtrip() {
        let result = ExecutionResult {
            macro_id: "test-id".into(),
            success: true,
            action: "Inserted text at cursor".into(),
            timestamp: "2026-01-15T10:30:00Z".into(),
        };

        let json = serde_json::to_string_pretty(&result).unwrap();
        let deserialized: ExecutionResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, deserialized);
    }

    #[test]
    fn test_import_result_roundtrip() {
        let result = ImportResult {
            imported_count: 5,
            skipped_count: 2,
            errors: vec!["Duplicate trigger: /sig".into()],
        };

        let json = serde_json::to_string_pretty(&result).unwrap();
        let deserialized: ImportResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, deserialized);
    }

    #[test]
    fn test_export_result_roundtrip() {
        let result = ExportResult {
            exported_count: 10,
            file_path: "C:/exports/macros.json".into(),
        };

        let json = serde_json::to_string_pretty(&result).unwrap();
        let deserialized: ExportResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, deserialized);
    }

    #[test]
    fn test_engine_error_roundtrip() {
        let error = EngineError {
            code: "TRIGGER_EXISTS".into(),
            message: "Trigger already exists".into(),
        };

        let json = serde_json::to_string_pretty(&error).unwrap();
        let deserialized: EngineError = serde_json::from_str(&json).unwrap();
        assert_eq!(error, deserialized);
    }

    #[test]
    fn test_engine_response_error_roundtrip() {
        let resp = EngineResponse::Error(EngineError {
            code: "NOT_FOUND".into(),
            message: "Macro not found".into(),
        });

        let json = serde_json::to_string_pretty(&resp).unwrap();
        let deserialized: EngineResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(resp, deserialized);
    }

    #[test]
    fn test_engine_response_macro_list_roundtrip() {
        let m = Macro {
            id: "test-id".into(),
            trigger: "/test".into(),
            description: "Test macro".into(),
            content: "Hello".into(),
            enabled: true,
            category: MacroCategory::Text,
            action_type: ActionType::InsertText,
            preserve_format: true,
            created_at: "2026-01-15T10:30:00Z".into(),
            updated_at: "2026-01-15T10:30:00Z".into(),
            tags: vec![],
            shortcut: None,
            event_trigger: None,
        };

        let resp = EngineResponse::MacroList(vec![m]);
        let json = serde_json::to_string_pretty(&resp).unwrap();
        let deserialized: EngineResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(resp, deserialized);
    }

    #[test]
    fn test_engine_response_macros_reloaded_roundtrip() {
        let resp = EngineResponse::MacrosReloaded;
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: EngineResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(resp, deserialized);
    }

    #[test]
    fn test_engine_response_macro_toggled_roundtrip() {
        let resp = EngineResponse::MacroToggled("some-id".into(), true);
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: EngineResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(resp, deserialized);
    }

    #[test]
    fn test_engine_event_macro_triggered_roundtrip() {
        let event = EngineEvent::MacroTriggered {
            macro_id: "test-id".into(),
            trigger: "/test".into(),
            timestamp: "2026-01-15T10:30:00Z".into(),
        };

        let json = serde_json::to_string_pretty(&event).unwrap();
        let deserialized: EngineEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_engine_event_engine_started_roundtrip() {
        let event = EngineEvent::EngineStarted;
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: EngineEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_engine_event_storage_error_roundtrip() {
        let event = EngineEvent::StorageError("File not found".into());
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: EngineEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_engine_event_config_changed_roundtrip() {
        let config = Config::default();
        let event = EngineEvent::ConfigChanged(config);
        let json = serde_json::to_string_pretty(&event).unwrap();
        let deserialized: EngineEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_import_result_empty_errors_roundtrip() {
        let result = ImportResult {
            imported_count: 3,
            skipped_count: 0,
            errors: vec![],
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"errors\":[]"));
        let deserialized: ImportResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, deserialized);
    }
}
