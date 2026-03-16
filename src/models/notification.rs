use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a notification message displayed to the user.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Notification {
    /// Unique identifier (UUID).
    pub id: String,
    /// Display text.
    pub message: String,
    /// Notification type: success, error, warning, or info.
    pub ntype: NotificationType,
    /// ISO 8601 timestamp of when the notification was created.
    pub timestamp: String,
    /// Whether the user has dismissed this notification.
    pub dismissed: bool,
}

impl Notification {
    /// Creates a new notification with auto-generated id and timestamp.
    pub fn new(message: String, ntype: NotificationType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            message,
            ntype,
            timestamp: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            dismissed: false,
        }
    }
}

/// Type of notification for visual differentiation.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    /// Action completed successfully.
    Success,
    /// Action failed or invalid input.
    Error,
    /// Potential issue or conflict.
    Warning,
    /// General informational message.
    Info,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_new_auto_generates_fields() {
        let n = Notification::new("Test message".into(), NotificationType::Success);
        assert!(!n.id.is_empty());
        assert!(!n.timestamp.is_empty());
        assert_eq!(n.dismissed, false);
        assert_eq!(n.message, "Test message");
    }

    #[test]
    fn test_notification_serialization_roundtrip() {
        let n = Notification {
            id: "notif-001".into(),
            message: "Macro saved successfully".into(),
            ntype: NotificationType::Success,
            timestamp: "2026-01-15T10:31:00Z".into(),
            dismissed: false,
        };

        let json = serde_json::to_string_pretty(&n).unwrap();
        let deserialized: Notification = serde_json::from_str(&json).unwrap();
        assert_eq!(n, deserialized);
    }

    #[test]
    fn test_notification_type_serialization() {
        assert_eq!(
            serde_json::to_string(&NotificationType::Success).unwrap(),
            "\"success\""
        );
        assert_eq!(
            serde_json::to_string(&NotificationType::Error).unwrap(),
            "\"error\""
        );
        assert_eq!(
            serde_json::to_string(&NotificationType::Warning).unwrap(),
            "\"warning\""
        );
        assert_eq!(
            serde_json::to_string(&NotificationType::Info).unwrap(),
            "\"info\""
        );
    }

    #[test]
    fn test_notification_type_deserialization() {
        assert_eq!(
            serde_json::from_str::<NotificationType>("\"success\"").unwrap(),
            NotificationType::Success
        );
        assert_eq!(
            serde_json::from_str::<NotificationType>("\"error\"").unwrap(),
            NotificationType::Error
        );
        assert_eq!(
            serde_json::from_str::<NotificationType>("\"warning\"").unwrap(),
            NotificationType::Warning
        );
        assert_eq!(
            serde_json::from_str::<NotificationType>("\"info\"").unwrap(),
            NotificationType::Info
        );
    }

    #[test]
    fn test_notification_type_unknown_value_errors() {
        let result = serde_json::from_str::<NotificationType>("\"unknown\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_notification_json_field_names() {
        let n = Notification::new("test".into(), NotificationType::Info);
        let json = serde_json::to_string(&n).unwrap();

        assert!(json.contains("\"id\""));
        assert!(json.contains("\"message\""));
        assert!(json.contains("\"ntype\""));
        assert!(json.contains("\"timestamp\""));
        assert!(json.contains("\"dismissed\""));
    }
}
