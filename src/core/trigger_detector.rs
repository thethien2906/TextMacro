use crate::models::macro_model::Macro;

pub struct TriggerDetector {
    buffer: String,
    max_size: usize,
}

impl TriggerDetector {
    /// Creates a new TriggerDetector with the specified maximum buffer size (in characters).
    pub fn new(max_size: usize) -> Self {
        Self {
            buffer: String::with_capacity(max_size * 4),
            max_size,
        }
    }

    /// Appends a character to the typing buffer. Maintains the rolling buffer limit.
    pub fn add_char(&mut self, c: char) {
        self.buffer.push(c);
        
        while self.buffer.chars().count() > self.max_size {
            if let Some(first_char) = self.buffer.chars().next() {
                self.buffer.drain(0..first_char.len_utf8());
            }
        }
    }

    /// Removes the last character from the buffer (handles Backspace).
    pub fn backspace(&mut self) {
        self.buffer.pop();
    }

    /// Clears the entire buffer. Called on Enter, Tab, Escape, etc.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Gets the current buffer content.
    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    /// Checks if the current buffer ends with any of the enabled macro triggers.
    /// Uses the longest-match rule: if multiple triggers match, the longest one is returned.
    /// Only considers macros where `enabled` is true.
    pub fn check_match<'a>(&self, macros: impl Iterator<Item = &'a Macro>) -> Option<&'a Macro> {
        let mut longest_match: Option<&'a Macro> = None;
        let mut max_trigger_len = 0;

        for m in macros {
            // Text and Prompt macros only use typing buffer
            if !m.enabled {
                continue;
            }

            if self.buffer.ends_with(&m.trigger) {
                let len = m.trigger.chars().count();
                if len > max_trigger_len {
                    max_trigger_len = len;
                    longest_match = Some(m);
                }
            }
        }

        longest_match
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::macro_model::{MacroCategory, ActionType};

    fn make_macro(trigger: &str, enabled: bool) -> Macro {
        Macro {
            id: uuid::Uuid::new_v4().to_string(),
            trigger: trigger.to_string(),
            description: "".into(),
            content: "output".into(),
            enabled,
            category: MacroCategory::Text,
            action_type: ActionType::InsertText,
            preserve_format: true,
            created_at: "".into(),
            updated_at: "".into(),
            tags: vec![],
            shortcut: None,
            event_trigger: None,
        }
    }

    #[test]
    fn test_buffer_addition_and_limit() {
        let mut detector = TriggerDetector::new(5);
        for c in "Hello".chars() {
            detector.add_char(c);
        }
        assert_eq!(detector.buffer(), "Hello");

        detector.add_char('!');
        assert_eq!(detector.buffer(), "ello!");
    }

    #[test]
    fn test_backspace_and_clear() {
        let mut detector = TriggerDetector::new(10);
        detector.add_char('a');
        detector.add_char('b');
        detector.backspace();
        assert_eq!(detector.buffer(), "a");

        detector.clear();
        assert_eq!(detector.buffer(), "");
    }

    #[test]
    fn test_trigger_match() {
        let mut detector = TriggerDetector::new(100);
        for c in "This is a /sig".chars() {
            detector.add_char(c);
        }

        let macros = vec![
            make_macro("/sig", true),
            make_macro("sig", true),
            make_macro("/sign", true),
            make_macro("/sig", false),
        ];

        let matched = detector.check_match(macros.iter()).unwrap();
        assert_eq!(matched.trigger, "/sig");
    }
}
