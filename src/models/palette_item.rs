use serde::{Deserialize, Serialize};

/// Represents an item in the Command Palette search results.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PaletteItem {
    /// Unique identifier.
    pub id: String,
    /// Display text (trigger or name).
    pub label: String,
    /// Secondary description text.
    #[serde(default)]
    pub description: String,
    /// "macro" or "command".
    pub item_type: String,
    /// Associated macro ID if item_type is "macro".
    #[serde(default)]
    pub macro_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_item_serialization_roundtrip() {
        let item = PaletteItem {
            id: "palette-001".into(),
            label: "/prompt-marketing".into(),
            description: "Marketing campaign prompt template".into(),
            item_type: "macro".into(),
            macro_id: Some("b2c3d4e5-f6a7-8901-bcde-f12345678901".into()),
        };

        let json = serde_json::to_string_pretty(&item).unwrap();
        let deserialized: PaletteItem = serde_json::from_str(&json).unwrap();
        assert_eq!(item, deserialized);
    }

    #[test]
    fn test_palette_item_command_type_roundtrip() {
        let item = PaletteItem {
            id: "palette-002".into(),
            label: "New Macro".into(),
            description: "Create a new macro".into(),
            item_type: "command".into(),
            macro_id: None,
        };

        let json = serde_json::to_string_pretty(&item).unwrap();
        let deserialized: PaletteItem = serde_json::from_str(&json).unwrap();
        assert_eq!(item, deserialized);
    }

    #[test]
    fn test_palette_item_optional_macro_id_null() {
        let item = PaletteItem {
            id: "palette-003".into(),
            label: "Settings".into(),
            description: "Open settings".into(),
            item_type: "command".into(),
            macro_id: None,
        };

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"macro_id\":null"));
    }

    #[test]
    fn test_palette_item_json_field_names() {
        let item = PaletteItem {
            id: "test".into(),
            label: "test".into(),
            description: String::new(),
            item_type: "command".into(),
            macro_id: None,
        };

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"id\""));
        assert!(json.contains("\"label\""));
        assert!(json.contains("\"description\""));
        assert!(json.contains("\"item_type\""));
        assert!(json.contains("\"macro_id\""));
    }
}
