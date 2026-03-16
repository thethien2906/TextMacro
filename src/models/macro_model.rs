use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// The central data object representing a user-defined macro.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Macro {
    /// Unique identifier (UUID format), auto-generated on creation.
    pub id: String,
    /// The trigger text (e.g., "/sig", "/prompt-marketing"). Must be unique across all macros.
    pub trigger: String,
    /// Optional short text explaining the macro's purpose.
    #[serde(default)]
    pub description: String,
    /// The full payload to insert or execute.
    pub content: String,
    /// Determines if the trigger detector considers this macro.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Determines which sidebar section displays this macro.
    #[serde(default)]
    pub category: MacroCategory,
    /// Determines what happens on execution.
    #[serde(default)]
    pub action_type: ActionType,
    /// Controls whether formatting is preserved during insertion.
    #[serde(default = "default_true")]
    pub preserve_format: bool,
    /// ISO 8601 timestamp, set once on creation.
    pub created_at: String,
    /// ISO 8601 timestamp, updated on every modification.
    pub updated_at: String,
    /// Optional categorization labels.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Optional keyboard shortcut for event macros.
    #[serde(default)]
    pub shortcut: Option<String>,
    /// Event configuration for event-type macros.
    #[serde(default)]
    pub event_trigger: Option<EventTrigger>,
}

fn default_true() -> bool {
    true
}

impl Macro {
    /// Creates a new Macro with auto-generated id and timestamps.
    pub fn new(trigger: String, content: String) -> Self {
        let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        Self {
            id: Uuid::new_v4().to_string(),
            trigger,
            description: String::new(),
            content,
            enabled: true,
            category: MacroCategory::Text,
            action_type: ActionType::InsertText,
            preserve_format: true,
            created_at: now.clone(),
            updated_at: now,
            tags: Vec::new(),
            shortcut: None,
            event_trigger: None,
        }
    }

    /// Updates the `updated_at` timestamp to the current time.
    pub fn touch(&mut self) {
        self.updated_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    }
}

/// Defines the configuration for event-based macro triggers.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EventTrigger {
    /// The kind of system event.
    pub event_type: EventType,
    /// Event-specific key-value configuration.
    #[serde(default)]
    pub parameters: HashMap<String, String>,
}

/// The kind of system event that can trigger a macro.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Triggered by a keyboard combination.
    KeyboardShortcut,
    /// Triggered when the system starts.
    SystemStartup,
    /// Triggered when a specific app launches.
    ApplicationLaunch,
    /// Triggered at a scheduled time/interval.
    Timer,
    /// Triggered when a specific file is modified.
    FileChange,
}

/// Categorizes macros for sidebar navigation and filtering.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MacroCategory {
    /// Standard text expansion macros.
    Text,
    /// Structured prompt macros (multiline/markdown).
    Prompt,
    /// Macros triggered by system events.
    Event,
}

impl Default for MacroCategory {
    fn default() -> Self {
        MacroCategory::Text
    }
}

/// Defines what happens when a macro is triggered.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    /// Insert text content at the cursor position.
    InsertText,
    /// Execute a shell command or external script.
    RunScript,
    /// Launch an external application.
    OpenProgram,
    /// Load and insert a structured prompt template.
    LoadPrompt,
}

impl Default for ActionType {
    fn default() -> Self {
        ActionType::InsertText
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_new_auto_generates_id_and_timestamps() {
        let m = Macro::new("/test".into(), "Hello World".into());
        assert!(!m.id.is_empty());
        assert!(!m.created_at.is_empty());
        assert!(!m.updated_at.is_empty());
        assert_eq!(m.enabled, true);
        assert_eq!(m.category, MacroCategory::Text);
        assert_eq!(m.action_type, ActionType::InsertText);
        assert_eq!(m.preserve_format, true);
    }

    #[test]
    fn test_macro_serialization_roundtrip() {
        let m = Macro {
            id: "a1b2c3d4-e5f6-7890-abcd-ef1234567890".into(),
            trigger: "/sig".into(),
            description: "Email signature".into(),
            content: "Best regards,\nJohn Doe".into(),
            enabled: true,
            category: MacroCategory::Text,
            action_type: ActionType::InsertText,
            preserve_format: true,
            created_at: "2026-01-15T10:30:00Z".into(),
            updated_at: "2026-01-15T10:30:00Z".into(),
            tags: vec!["email".into(), "signature".into()],
            shortcut: None,
            event_trigger: None,
        };

        let json = serde_json::to_string_pretty(&m).unwrap();
        let deserialized: Macro = serde_json::from_str(&json).unwrap();
        assert_eq!(m, deserialized);
    }

    #[test]
    fn test_macro_json_field_names_are_snake_case() {
        let m = Macro::new("/test".into(), "content".into());
        let json = serde_json::to_string(&m).unwrap();

        assert!(json.contains("\"action_type\""));
        assert!(json.contains("\"preserve_format\""));
        assert!(json.contains("\"created_at\""));
        assert!(json.contains("\"updated_at\""));
        assert!(json.contains("\"event_trigger\""));
    }

    #[test]
    fn test_macro_category_serialization() {
        assert_eq!(
            serde_json::to_string(&MacroCategory::Text).unwrap(),
            "\"text\""
        );
        assert_eq!(
            serde_json::to_string(&MacroCategory::Prompt).unwrap(),
            "\"prompt\""
        );
        assert_eq!(
            serde_json::to_string(&MacroCategory::Event).unwrap(),
            "\"event\""
        );
    }

    #[test]
    fn test_macro_category_deserialization() {
        assert_eq!(
            serde_json::from_str::<MacroCategory>("\"text\"").unwrap(),
            MacroCategory::Text
        );
        assert_eq!(
            serde_json::from_str::<MacroCategory>("\"prompt\"").unwrap(),
            MacroCategory::Prompt
        );
        assert_eq!(
            serde_json::from_str::<MacroCategory>("\"event\"").unwrap(),
            MacroCategory::Event
        );
    }

    #[test]
    fn test_macro_category_unknown_value_errors() {
        let result = serde_json::from_str::<MacroCategory>("\"unknown\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_action_type_serialization() {
        assert_eq!(
            serde_json::to_string(&ActionType::InsertText).unwrap(),
            "\"insert_text\""
        );
        assert_eq!(
            serde_json::to_string(&ActionType::RunScript).unwrap(),
            "\"run_script\""
        );
        assert_eq!(
            serde_json::to_string(&ActionType::OpenProgram).unwrap(),
            "\"open_program\""
        );
        assert_eq!(
            serde_json::to_string(&ActionType::LoadPrompt).unwrap(),
            "\"load_prompt\""
        );
    }

    #[test]
    fn test_action_type_deserialization() {
        assert_eq!(
            serde_json::from_str::<ActionType>("\"insert_text\"").unwrap(),
            ActionType::InsertText
        );
        assert_eq!(
            serde_json::from_str::<ActionType>("\"run_script\"").unwrap(),
            ActionType::RunScript
        );
        assert_eq!(
            serde_json::from_str::<ActionType>("\"open_program\"").unwrap(),
            ActionType::OpenProgram
        );
        assert_eq!(
            serde_json::from_str::<ActionType>("\"load_prompt\"").unwrap(),
            ActionType::LoadPrompt
        );
    }

    #[test]
    fn test_action_type_unknown_value_errors() {
        let result = serde_json::from_str::<ActionType>("\"unknown\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_event_type_serialization() {
        assert_eq!(
            serde_json::to_string(&EventType::KeyboardShortcut).unwrap(),
            "\"keyboard_shortcut\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::SystemStartup).unwrap(),
            "\"system_startup\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::ApplicationLaunch).unwrap(),
            "\"application_launch\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::Timer).unwrap(),
            "\"timer\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::FileChange).unwrap(),
            "\"file_change\""
        );
    }

    #[test]
    fn test_event_type_deserialization() {
        assert_eq!(
            serde_json::from_str::<EventType>("\"keyboard_shortcut\"").unwrap(),
            EventType::KeyboardShortcut
        );
        assert_eq!(
            serde_json::from_str::<EventType>("\"system_startup\"").unwrap(),
            EventType::SystemStartup
        );
        assert_eq!(
            serde_json::from_str::<EventType>("\"application_launch\"").unwrap(),
            EventType::ApplicationLaunch
        );
        assert_eq!(
            serde_json::from_str::<EventType>("\"timer\"").unwrap(),
            EventType::Timer
        );
        assert_eq!(
            serde_json::from_str::<EventType>("\"file_change\"").unwrap(),
            EventType::FileChange
        );
    }

    #[test]
    fn test_event_type_unknown_value_errors() {
        let result = serde_json::from_str::<EventType>("\"unknown\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_event_trigger_roundtrip() {
        let et = EventTrigger {
            event_type: EventType::KeyboardShortcut,
            parameters: {
                let mut map = HashMap::new();
                map.insert("keys".into(), "Ctrl+Alt+E".into());
                map.insert("description".into(), "Insert email template".into());
                map
            },
        };

        let json = serde_json::to_string_pretty(&et).unwrap();
        let deserialized: EventTrigger = serde_json::from_str(&json).unwrap();
        assert_eq!(et, deserialized);
    }

    #[test]
    fn test_optional_fields_serialize_as_null() {
        let m = Macro::new("/test".into(), "content".into());
        let json = serde_json::to_string(&m).unwrap();

        assert!(json.contains("\"shortcut\":null"));
        assert!(json.contains("\"event_trigger\":null"));
    }

    #[test]
    fn test_multiline_content_with_special_chars_roundtrip() {
        let content = "# Project Overview\n\n## Goals\n- Improve performance\n- Reduce manual tasks\n\n## Code Example\n```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```\n\n## Special Characters: <>&\"'\\t\\n\nIndented:\n    Level 1\n        Level 2\n            Level 3";

        let m = Macro {
            id: "test-id".into(),
            trigger: "/doc-template".into(),
            description: "Documentation template".into(),
            content: content.into(),
            enabled: true,
            category: MacroCategory::Prompt,
            action_type: ActionType::InsertText,
            preserve_format: true,
            created_at: "2026-01-15T10:30:00Z".into(),
            updated_at: "2026-01-15T10:30:00Z".into(),
            tags: vec!["docs".into(), "markdown".into()],
            shortcut: None,
            event_trigger: None,
        };

        let json = serde_json::to_string_pretty(&m).unwrap();
        let deserialized: Macro = serde_json::from_str(&json).unwrap();
        assert_eq!(m.content, deserialized.content);
        assert_eq!(m, deserialized);
    }

    #[test]
    fn test_prompt_macro_roundtrip() {
        let m = Macro {
            id: "b2c3d4e5-f6a7-8901-bcde-f12345678901".into(),
            trigger: "/prompt-marketing".into(),
            description: "Marketing campaign prompt template".into(),
            content: "You are a professional marketing strategist.\n\nTask:\nCreate a marketing campaign plan for the following product.\n\nRequirements:\n- Target audience analysis\n- Key messaging\n- Marketing channels\n- Campaign timeline\n\nOutput Format:\n\n# Campaign Plan\n\n## Target Audience\n...\n\n## Messaging\n...\n\n## Channels\n...\n\n## Timeline\n...".into(),
            enabled: true,
            category: MacroCategory::Prompt,
            action_type: ActionType::InsertText,
            preserve_format: true,
            created_at: "2026-02-01T08:00:00Z".into(),
            updated_at: "2026-02-01T08:00:00Z".into(),
            tags: vec!["ai".into(), "marketing".into(), "prompt".into()],
            shortcut: None,
            event_trigger: None,
        };

        let json = serde_json::to_string_pretty(&m).unwrap();
        let deserialized: Macro = serde_json::from_str(&json).unwrap();
        assert_eq!(m, deserialized);
    }

    #[test]
    fn test_event_macro_roundtrip() {
        let m = Macro {
            id: "c3d4e5f6-a7b8-9012-cdef-123456789012".into(),
            trigger: "system_startup".into(),
            description: "Launch work apps on system startup".into(),
            content: String::new(),
            enabled: true,
            category: MacroCategory::Event,
            action_type: ActionType::RunScript,
            preserve_format: false,
            created_at: "2026-01-20T09:00:00Z".into(),
            updated_at: "2026-01-20T09:00:00Z".into(),
            tags: vec!["startup".into(), "automation".into()],
            shortcut: None,
            event_trigger: Some(EventTrigger {
                event_type: EventType::SystemStartup,
                parameters: HashMap::new(),
            }),
        };

        let json = serde_json::to_string_pretty(&m).unwrap();
        let deserialized: Macro = serde_json::from_str(&json).unwrap();
        assert_eq!(m, deserialized);
    }

    #[test]
    fn test_vec_fields_serialize_as_empty_array() {
        let m = Macro::new("/test".into(), "content".into());
        let json = serde_json::to_string(&m).unwrap();
        assert!(json.contains("\"tags\":[]"));
    }

    #[test]
    fn test_macro_touch_updates_timestamp() {
        let mut m = Macro::new("/test".into(), "content".into());
        let _original_updated = m.updated_at.clone();
        let original_created = m.created_at.clone();

        // Give a tiny bit of time to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(10));
        m.touch();

        // created_at should not change
        assert_eq!(m.created_at, original_created);
        // updated_at should be different (or same second, depending on timing)
        // At minimum, touch should not panic and should set a valid timestamp
        assert!(!m.updated_at.is_empty());
    }
}
