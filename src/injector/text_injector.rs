use arboard::Clipboard;
use rdev::{simulate, EventType, Key, SimulateError};
use std::thread;
use std::time::Duration;

/// Delays between simulated keystrokes.
const BACKSPACE_DELAY_MS: u64 = 8;
const POST_BACKSPACE_DELAY_MS: u64 = 10;
const POST_CLIPBOARD_SET_DELAY_MS: u64 = 10;
const POST_PASTE_DELAY_MS: u64 = 150;

/// Handles text injection via clipboard-based paste and simulated keystrokes.
pub struct TextInjector;

/// Errors that can occur during text injection.
#[derive(Debug)]
pub enum InjectorError {
    ClipboardAccessFailed(String),
    SimulationFailed(String),
}

impl std::fmt::Display for InjectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InjectorError::ClipboardAccessFailed(msg) => {
                write!(f, "Clipboard access failed: {}", msg)
            }
            InjectorError::SimulationFailed(msg) => {
                write!(f, "Key simulation failed: {}", msg)
            }
        }
    }
}

impl std::error::Error for InjectorError {}

fn simulate_key(event_type: &EventType) -> Result<(), SimulateError> {
    simulate(event_type)
}

fn delay(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}

impl TextInjector {
    /// Perform the full text insertion flow for a typed trigger:
    /// 1. Backup clipboard
    /// 2. Delete trigger text (backspaces)
    /// 3. Set clipboard to content
    /// 4. Paste (Ctrl+V)
    /// 5. Restore clipboard
    pub fn inject_text(content: &str, trigger_length: usize) -> Result<(), InjectorError> {
        // Step 1: Backup clipboard
        let clipboard_backup = Self::backup_clipboard();

        // Step 2: Delete trigger text
        Self::delete_trigger(trigger_length)?;

        // Step 3 + 4: Copy content and paste
        Self::set_clipboard_and_paste(content)?;

        // Step 5: Restore clipboard (best-effort)
        delay(POST_PASTE_DELAY_MS);
        Self::restore_clipboard(clipboard_backup);

        Ok(())
    }

    /// Perform text insertion for event-triggered macros (no trigger deletion):
    /// 1. Backup clipboard
    /// 2. Set clipboard to content
    /// 3. Paste (Ctrl+V)
    /// 4. Restore clipboard
    pub fn inject_text_no_delete(content: &str) -> Result<(), InjectorError> {
        let clipboard_backup = Self::backup_clipboard();

        Self::set_clipboard_and_paste(content)?;

        delay(POST_PASTE_DELAY_MS);
        Self::restore_clipboard(clipboard_backup);

        Ok(())
    }

    /// Backup current clipboard text. Returns None if clipboard is empty or inaccessible.
    fn backup_clipboard() -> Option<String> {
        Clipboard::new()
            .ok()
            .and_then(|mut cb| cb.get_text().ok())
            .filter(|text| !text.is_empty())
    }

    /// Simulate N backspaces to delete the trigger text.
    fn delete_trigger(count: usize) -> Result<(), InjectorError> {
        for _ in 0..count {
            simulate_key(&EventType::KeyPress(Key::Backspace))
                .map_err(|e| InjectorError::SimulationFailed(format!("Backspace press: {:?}", e)))?;
            simulate_key(&EventType::KeyRelease(Key::Backspace))
                .map_err(|e| {
                    InjectorError::SimulationFailed(format!("Backspace release: {:?}", e))
                })?;
            delay(BACKSPACE_DELAY_MS);
        }
        delay(POST_BACKSPACE_DELAY_MS);
        Ok(())
    }

    /// Set clipboard content and simulate Ctrl+V paste.
    fn set_clipboard_and_paste(content: &str) -> Result<(), InjectorError> {
        // Set clipboard
        let mut clipboard = Clipboard::new().map_err(|e| {
            InjectorError::ClipboardAccessFailed(format!("Failed to open clipboard: {}", e))
        })?;
        clipboard.set_text(content).map_err(|e| {
            InjectorError::ClipboardAccessFailed(format!("Failed to set clipboard text: {}", e))
        })?;

        delay(POST_CLIPBOARD_SET_DELAY_MS);

        // Simulate Ctrl+V (or Cmd+V on macOS)
        let modifier = if cfg!(target_os = "macos") {
            Key::MetaLeft
        } else {
            Key::ControlLeft
        };

        simulate_key(&EventType::KeyPress(modifier)).map_err(|e| {
            InjectorError::SimulationFailed(format!("Modifier press: {:?}", e))
        })?;
        simulate_key(&EventType::KeyPress(Key::KeyV)).map_err(|e| {
            InjectorError::SimulationFailed(format!("V press: {:?}", e))
        })?;
        simulate_key(&EventType::KeyRelease(Key::KeyV)).map_err(|e| {
            InjectorError::SimulationFailed(format!("V release: {:?}", e))
        })?;
        simulate_key(&EventType::KeyRelease(modifier)).map_err(|e| {
            InjectorError::SimulationFailed(format!("Modifier release: {:?}", e))
        })?;

        Ok(())
    }

    /// Restore the clipboard to its previous content (best-effort).
    fn restore_clipboard(backup: Option<String>) {
        if let Ok(mut clipboard) = Clipboard::new() {
            match backup {
                Some(text) => {
                    let _ = clipboard.set_text(text);
                }
                None => {
                    let _ = clipboard.clear();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_injector_error_display_clipboard() {
        let err = InjectorError::ClipboardAccessFailed("test".into());
        assert!(err.to_string().contains("Clipboard access failed"));
    }

    #[test]
    fn test_injector_error_display_simulation() {
        let err = InjectorError::SimulationFailed("test".into());
        assert!(err.to_string().contains("Key simulation failed"));
    }

    #[test]
    fn test_backup_clipboard_returns_option() {
        // This test just verifies the backup function doesn't panic.
        // Actual clipboard content depends on system state.
        let _backup = TextInjector::backup_clipboard();
    }

    #[test]
    fn test_restore_clipboard_does_not_panic_on_none() {
        TextInjector::restore_clipboard(None);
    }

    #[test]
    fn test_restore_clipboard_does_not_panic_on_some() {
        TextInjector::restore_clipboard(Some("test content".into()));
    }
}
