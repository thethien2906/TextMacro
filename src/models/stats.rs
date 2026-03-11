use serde::{Deserialize, Serialize};

/// Tracks usage statistics for a macro.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MacroStats {
    /// Reference to the macro.
    pub macro_id: String,
    /// Number of times the macro was triggered.
    pub trigger_count: u64,
    /// ISO 8601 timestamp of the last trigger, or None if never triggered.
    #[serde(default)]
    pub last_triggered: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_stats_serialization_roundtrip() {
        let stats = MacroStats {
            macro_id: "a1b2c3d4-e5f6-7890-abcd-ef1234567890".into(),
            trigger_count: 42,
            last_triggered: Some("2026-03-09T14:20:00Z".into()),
        };

        let json = serde_json::to_string_pretty(&stats).unwrap();
        let deserialized: MacroStats = serde_json::from_str(&json).unwrap();
        assert_eq!(stats, deserialized);
    }

    #[test]
    fn test_macro_stats_never_triggered_roundtrip() {
        let stats = MacroStats {
            macro_id: "test-id".into(),
            trigger_count: 0,
            last_triggered: None,
        };

        let json = serde_json::to_string_pretty(&stats).unwrap();
        let deserialized: MacroStats = serde_json::from_str(&json).unwrap();
        assert_eq!(stats, deserialized);
    }

    #[test]
    fn test_macro_stats_optional_last_triggered_null() {
        let stats = MacroStats {
            macro_id: "test-id".into(),
            trigger_count: 0,
            last_triggered: None,
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"last_triggered\":null"));
    }

    #[test]
    fn test_macro_stats_json_field_names() {
        let stats = MacroStats {
            macro_id: "test".into(),
            trigger_count: 0,
            last_triggered: None,
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"macro_id\""));
        assert!(json.contains("\"trigger_count\""));
        assert!(json.contains("\"last_triggered\""));
    }
}
