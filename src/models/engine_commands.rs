use serde::{Deserialize, Serialize};

use super::macro_model::{MacroCategory, EventTrigger, ActionType};
use super::config::Config;

/// Message types sent from UI to engine.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EngineCommand {
    /// Get all macros for a category.
    GetMacros(MacroCategory),
    /// Get a macro by its ID.
    GetMacroById(String),
    /// Create a new macro.
    CreateMacro(MacroCreateRequest),
    /// Update an existing macro.
    UpdateMacro(MacroUpdateRequest),
    /// Delete a macro by ID.
    DeleteMacro(String),
    /// Toggle macro enabled state: (id, new_state).
    ToggleMacro(String, bool),
    /// Search macros by query string.
    SearchMacros(String),
    /// Execute a macro by ID.
    ExecuteMacro(String),
    /// Get current configuration.
    GetConfig,
    /// Update configuration.
    UpdateConfig(Config),
    /// Import macros from file path.
    ImportMacros(String),
    /// Export macros to file path.
    ExportMacros(String),
    /// Reload macros from storage.
    ReloadMacros,
    /// Get stats for a macro by ID.
    GetStats(String),
}

/// Input structure for creating a new macro.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MacroCreateRequest {
    /// The trigger text (required).
    pub trigger: String,
    /// The full content/payload (required).
    pub content: String,
    /// The category (required).
    pub category: MacroCategory,
    /// The action type (required).
    pub action_type: ActionType,
    /// Optional short description.
    #[serde(default)]
    pub description: Option<String>,
    /// Whether to preserve formatting.
    #[serde(default)]
    pub preserve_format: Option<bool>,
    /// Optional tags.
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    /// Optional keyboard shortcut.
    #[serde(default)]
    pub shortcut: Option<String>,
    /// Optional event trigger configuration.
    #[serde(default)]
    pub event_trigger: Option<EventTrigger>,
}

/// Input structure for updating an existing macro.
/// Only non-None fields are applied during update.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MacroUpdateRequest {
    /// The ID of the macro to update (required).
    pub id: String,
    /// New trigger text.
    #[serde(default)]
    pub trigger: Option<String>,
    /// New description.
    #[serde(default)]
    pub description: Option<String>,
    /// New content.
    #[serde(default)]
    pub content: Option<String>,
    /// New enabled state.
    #[serde(default)]
    pub enabled: Option<bool>,
    /// New category.
    #[serde(default)]
    pub category: Option<MacroCategory>,
    /// New action type.
    #[serde(default)]
    pub action_type: Option<ActionType>,
    /// New preserve_format value.
    #[serde(default)]
    pub preserve_format: Option<bool>,
    /// New tags.
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    /// New shortcut.
    #[serde(default)]
    pub shortcut: Option<Option<String>>,
    /// New event trigger.
    #[serde(default)]
    pub event_trigger: Option<Option<EventTrigger>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_engine_command_get_macros_roundtrip() {
        let cmd = EngineCommand::GetMacros(MacroCategory::Text);
        let json = serde_json::to_string(&cmd).unwrap();
        let deserialized: EngineCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, deserialized);
    }

    #[test]
    fn test_engine_command_create_macro_roundtrip() {
        let cmd = EngineCommand::CreateMacro(MacroCreateRequest {
            trigger: "/test".into(),
            content: "Test content".into(),
            category: MacroCategory::Text,
            action_type: ActionType::InsertText,
            description: Some("A test macro".into()),
            preserve_format: Some(true),
            tags: Some(vec!["test".into()]),
            shortcut: None,
            event_trigger: None,
        });

        let json = serde_json::to_string_pretty(&cmd).unwrap();
        let deserialized: EngineCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, deserialized);
    }

    #[test]
    fn test_engine_command_toggle_macro_roundtrip() {
        let cmd = EngineCommand::ToggleMacro("macro-id".into(), false);
        let json = serde_json::to_string(&cmd).unwrap();
        let deserialized: EngineCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, deserialized);
    }

    #[test]
    fn test_engine_command_no_payload_variants_roundtrip() {
        let commands = vec![
            EngineCommand::GetConfig,
            EngineCommand::ReloadMacros,
        ];

        for cmd in commands {
            let json = serde_json::to_string(&cmd).unwrap();
            let deserialized: EngineCommand = serde_json::from_str(&json).unwrap();
            assert_eq!(cmd, deserialized);
        }
    }

    #[test]
    fn test_macro_create_request_roundtrip() {
        let req = MacroCreateRequest {
            trigger: "/sig".into(),
            content: "Best regards,\nJohn Doe".into(),
            category: MacroCategory::Text,
            action_type: ActionType::InsertText,
            description: Some("Email signature".into()),
            preserve_format: Some(true),
            tags: Some(vec!["email".into()]),
            shortcut: None,
            event_trigger: None,
        };

        let json = serde_json::to_string_pretty(&req).unwrap();
        let deserialized: MacroCreateRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req, deserialized);
    }

    #[test]
    fn test_macro_create_request_minimal() {
        let req = MacroCreateRequest {
            trigger: "/test".into(),
            content: "content".into(),
            category: MacroCategory::Text,
            action_type: ActionType::InsertText,
            description: None,
            preserve_format: None,
            tags: None,
            shortcut: None,
            event_trigger: None,
        };

        let json = serde_json::to_string_pretty(&req).unwrap();
        let deserialized: MacroCreateRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req, deserialized);
    }

    #[test]
    fn test_macro_update_request_partial_update() {
        let req = MacroUpdateRequest {
            id: "some-id".into(),
            trigger: Some("/new-trigger".into()),
            description: None,
            content: None,
            enabled: Some(false),
            category: None,
            action_type: None,
            preserve_format: None,
            tags: None,
            shortcut: None,
            event_trigger: None,
        };

        let json = serde_json::to_string_pretty(&req).unwrap();
        let deserialized: MacroUpdateRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req, deserialized);
    }

    #[test]
    fn test_macro_update_request_with_event_trigger() {
        let req = MacroUpdateRequest {
            id: "some-id".into(),
            trigger: None,
            description: None,
            content: None,
            enabled: None,
            category: Some(MacroCategory::Event),
            action_type: Some(ActionType::RunScript),
            preserve_format: None,
            tags: None,
            shortcut: None,
            event_trigger: Some(Some(EventTrigger {
                event_type: super::super::macro_model::EventType::KeyboardShortcut,
                parameters: {
                    let mut map = HashMap::new();
                    map.insert("keys".into(), "Ctrl+Alt+E".into());
                    map
                },
            })),
        };

        let json = serde_json::to_string_pretty(&req).unwrap();
        let deserialized: MacroUpdateRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req, deserialized);
    }
}
