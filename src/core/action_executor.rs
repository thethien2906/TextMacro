use chrono::Utc;
use std::process::Command;
use std::time::Duration;

use crate::injector::text_injector::{InjectorError, TextInjector};
use crate::models::engine_responses::{EngineError, ExecutionResult};
use crate::models::macro_model::{ActionType, Macro};

/// Timeout for script execution (30 seconds).
const SCRIPT_TIMEOUT_SECS: u64 = 30;

/// Executes macro actions: text insertion, script execution, and program launching.
pub struct ActionExecutor;

impl ActionExecutor {
    /// Execute a macro that was triggered by typing (trigger text needs deletion).
    pub fn execute_typed_trigger(
        macro_data: &Macro,
        trigger_length: usize,
    ) -> Result<ExecutionResult, EngineError> {
        if !macro_data.enabled {
            return Err(EngineError {
                code: "MACRO_DISABLED".into(),
                message: format!("Macro '{}' is disabled", macro_data.trigger),
            });
        }

        match macro_data.action_type {
            ActionType::InsertText | ActionType::LoadPrompt => {
                Self::execute_text_insert(&macro_data.id, &macro_data.content, trigger_length)
            }
            ActionType::RunScript => {
                Self::execute_script(&macro_data.id, &macro_data.content)
            }
            ActionType::OpenProgram => {
                Self::execute_program(&macro_data.id, &macro_data.content)
            }
        }
    }

    /// Execute a macro triggered by event (no trigger text deletion).
    pub fn execute_event_trigger(macro_data: &Macro) -> Result<ExecutionResult, EngineError> {
        if !macro_data.enabled {
            return Err(EngineError {
                code: "MACRO_DISABLED".into(),
                message: format!("Macro '{}' is disabled", macro_data.trigger),
            });
        }

        match macro_data.action_type {
            ActionType::InsertText | ActionType::LoadPrompt => {
                Self::execute_text_insert_no_delete(&macro_data.id, &macro_data.content)
            }
            ActionType::RunScript => {
                Self::execute_script(&macro_data.id, &macro_data.content)
            }
            ActionType::OpenProgram => {
                Self::execute_program(&macro_data.id, &macro_data.content)
            }
        }
    }

    /// Execute a macro by ID from the UI (Command Palette). No trigger deletion.
    pub fn execute_manual(macro_data: &Macro) -> Result<ExecutionResult, EngineError> {
        Self::execute_event_trigger(macro_data)
    }

    /// Insert text with trigger deletion (typed trigger).
    fn execute_text_insert(
        macro_id: &str,
        content: &str,
        trigger_length: usize,
    ) -> Result<ExecutionResult, EngineError> {
        log::info!(target: "injector", "Inserting text for macro: {}", macro_id);
        TextInjector::inject_text(content, trigger_length).map_err(|e| match e {
            InjectorError::ClipboardAccessFailed(msg) => {
                log::error!(target: "injector", "Text insert failed (clipboard): {}", msg);
                EngineError {
                    code: "EXECUTION_FAILED".into(),
                    message: format!("Failed to insert text: {}", msg),
                }
            },
            InjectorError::SimulationFailed(msg) => {
                log::error!(target: "injector", "Text insert failed (simulation): {}", msg);
                EngineError {
                    code: "EXECUTION_FAILED".into(),
                    message: format!("Failed to insert text: {}", msg),
                }
            },
        })?;

        Ok(ExecutionResult {
            macro_id: macro_id.to_string(),
            success: true,
            action: format!("Text inserted via clipboard ({} characters)", content.len()),
            timestamp: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        })
    }

    /// Insert text without trigger deletion (event/manual trigger).
    fn execute_text_insert_no_delete(
        macro_id: &str,
        content: &str,
    ) -> Result<ExecutionResult, EngineError> {
        log::info!(target: "injector", "Inserting text (no delete) for macro: {}", macro_id);
        TextInjector::inject_text_no_delete(content).map_err(|e| match e {
            InjectorError::ClipboardAccessFailed(msg) => {
                log::error!(target: "injector", "Text insert failed (clipboard): {}", msg);
                EngineError {
                    code: "EXECUTION_FAILED".into(),
                    message: format!("Failed to insert text: {}", msg),
                }
            },
            InjectorError::SimulationFailed(msg) => {
                log::error!(target: "injector", "Text insert failed (simulation): {}", msg);
                EngineError {
                    code: "EXECUTION_FAILED".into(),
                    message: format!("Failed to insert text: {}", msg),
                }
            },
        })?;

        Ok(ExecutionResult {
            macro_id: macro_id.to_string(),
            success: true,
            action: format!("Text inserted via clipboard ({} characters)", content.len()),
            timestamp: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        })
    }

    /// Execute a shell script with timeout.
    fn execute_script(macro_id: &str, script: &str) -> Result<ExecutionResult, EngineError> {
        log::info!(target: "executor", "Executing script for macro: {}", macro_id);
        let mut command = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd.exe");
            c.args(["/C", script]);
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                c.creation_flags(0x08000000); // CREATE_NO_WINDOW
            }
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", script]);
            c
        };

        let child = command
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        let mut child = match child {
            Ok(c) => c,
            Err(e) => {
                return Ok(ExecutionResult {
                    macro_id: macro_id.to_string(),
                    success: false,
                    action: format!("Script execution failed: {}", e),
                    timestamp: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                });
            }
        };

        // Wait with timeout
        let timeout = Duration::from_secs(SCRIPT_TIMEOUT_SECS);
        let start = std::time::Instant::now();

        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let exit_code = status.code().unwrap_or(-1);
                    let success = exit_code == 0;
                    let action = if success {
                        format!("Script executed successfully (exit code {})", exit_code)
                    } else {
                        format!("Script failed (exit code {})", exit_code)
                    };
                    return Ok(ExecutionResult {
                        macro_id: macro_id.to_string(),
                        success,
                        action,
                        timestamp: Utc::now()
                            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                    });
                }
                Ok(None) => {
                    if start.elapsed() >= timeout {
                        let _ = child.kill();
                        return Ok(ExecutionResult {
                            macro_id: macro_id.to_string(),
                            success: false,
                            action: "Script timed out after 30 seconds".to_string(),
                            timestamp: Utc::now()
                                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                        });
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    return Err(EngineError {
                        code: "SCRIPT_ERROR".into(),
                        message: format!("Failed to check script status: {}", e),
                    });
                }
            }
        }
    }

    /// Launch an external program (detached, non-blocking).
    fn execute_program(macro_id: &str, program: &str) -> Result<ExecutionResult, EngineError> {
        let mut command = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd.exe");
            c.args(["/C", "start", "", program]);
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                c.creation_flags(0x08000000); // CREATE_NO_WINDOW
            }
            c
        } else {
            let app = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
            let mut c = Command::new(app);
            c.arg(program);
            c
        };

        let result = command
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();

        match result {
            Ok(_) => Ok(ExecutionResult {
                macro_id: macro_id.to_string(),
                success: true,
                action: format!("Program launched: {}", program),
                timestamp: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            }),
            Err(e) => Ok(ExecutionResult {
                macro_id: macro_id.to_string(),
                success: false,
                action: format!("Failed to launch program '{}': {}", program, e),
                timestamp: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::macro_model::{ActionType, Macro, MacroCategory};

    fn make_text_macro(trigger: &str, content: &str) -> Macro {
        Macro {
            id: "test-id".into(),
            trigger: trigger.into(),
            description: String::new(),
            content: content.into(),
            enabled: true,
            category: MacroCategory::Text,
            action_type: ActionType::InsertText,
            preserve_format: true,
            created_at: "2026-01-15T10:30:00Z".into(),
            updated_at: "2026-01-15T10:30:00Z".into(),
            tags: vec![],
            shortcut: None,
            event_trigger: None,
        }
    }

    fn make_script_macro(script: &str) -> Macro {
        Macro {
            id: "script-id".into(),
            trigger: "/run".into(),
            description: String::new(),
            content: script.into(),
            enabled: true,
            category: MacroCategory::Event,
            action_type: ActionType::RunScript,
            preserve_format: false,
            created_at: "2026-01-15T10:30:00Z".into(),
            updated_at: "2026-01-15T10:30:00Z".into(),
            tags: vec![],
            shortcut: None,
            event_trigger: None,
        }
    }

    fn make_program_macro(program: &str) -> Macro {
        Macro {
            id: "prog-id".into(),
            trigger: "/open".into(),
            description: String::new(),
            content: program.into(),
            enabled: true,
            category: MacroCategory::Event,
            action_type: ActionType::OpenProgram,
            preserve_format: false,
            created_at: "2026-01-15T10:30:00Z".into(),
            updated_at: "2026-01-15T10:30:00Z".into(),
            tags: vec![],
            shortcut: None,
            event_trigger: None,
        }
    }

    #[test]
    fn test_disabled_macro_returns_error() {
        let mut m = make_text_macro("/sig", "Best regards");
        m.enabled = false;

        let result = ActionExecutor::execute_typed_trigger(&m, 4);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "MACRO_DISABLED");
    }

    #[test]
    fn test_disabled_macro_event_returns_error() {
        let mut m = make_text_macro("/sig", "Best regards");
        m.enabled = false;

        let result = ActionExecutor::execute_event_trigger(&m);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "MACRO_DISABLED");
    }

    #[test]
    fn test_script_execution_echo() {
        let m = make_script_macro("echo hello");
        let result = ActionExecutor::execute_event_trigger(&m);
        assert!(result.is_ok());
        let exec_result = result.unwrap();
        assert!(exec_result.success);
        assert!(exec_result.action.contains("exit code 0"));
    }

    #[test]
    fn test_script_execution_failing_command() {
        let m = make_script_macro("exit 1");
        let result = ActionExecutor::execute_event_trigger(&m);
        assert!(result.is_ok());
        let exec_result = result.unwrap();
        assert!(!exec_result.success);
        assert!(exec_result.action.contains("exit code 1"));
    }

    #[test]
    fn test_program_launch_nonexistent() {
        let m = make_program_macro("totally_nonexistent_program_xyz_123");
        let result = ActionExecutor::execute_event_trigger(&m);
        // start command may still succeed even if target doesn't exist —
        // the result depends on the OS. Just verify no panic.
        assert!(result.is_ok());
    }

    #[test]
    fn test_execution_result_has_valid_fields() {
        let m = make_script_macro("echo test");
        let result = ActionExecutor::execute_event_trigger(&m).unwrap();
        assert_eq!(result.macro_id, "script-id");
        assert!(!result.timestamp.is_empty());
        assert!(!result.action.is_empty());
    }

    #[test]
    fn test_manual_execution_delegates_to_event() {
        let m = make_script_macro("echo manual");
        let result = ActionExecutor::execute_manual(&m);
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }
}
