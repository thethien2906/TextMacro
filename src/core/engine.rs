use std::collections::HashMap;

use crate::core::action_executor::ActionExecutor;
use crate::models::config::Config;
use crate::models::engine_commands::{MacroCreateRequest, MacroUpdateRequest};
use crate::models::engine_responses::{EngineError, ExecutionResult};
use crate::models::macro_model::{ActionType, Macro, MacroCategory};
use crate::storage::macro_repository::StorageManager;

/// The core macro management engine.
pub struct Engine {
    /// Maps trigger string -> Macro. For fast exact match lookups.
    pub trigger_map: HashMap<String, Macro>,
    /// Maps macro UUID -> Macro. For management operations.
    pub id_map: HashMap<String, Macro>,
    /// The runtime application configuration.
    pub config: Config,
    /// Storage layer for persisting data.
    pub storage: StorageManager,
}

impl Engine {
    /// Initialize the engine, loading macros and config from storage.
    pub fn new(storage: StorageManager) -> Self {
        let (macros, _) = storage.load_macros();
        let (config, _) = storage.load_config();
        // Ignoring stats for now

        let mut trigger_map = HashMap::new();
        let mut id_map = HashMap::new();

        for m in macros {
            trigger_map.insert(m.trigger.clone(), m.clone());
            id_map.insert(m.id.clone(), m);
        }

        Self {
            trigger_map,
            id_map,
            config,
            storage,
        }
    }

    /// Retrieve all macros for a given category.
    pub fn get_macros(&self, category: MacroCategory) -> Vec<Macro> {
        let mut result: Vec<Macro> = self
            .id_map
            .values()
            .filter(|m| m.category == category)
            .cloned()
            .collect();

        // Sort by trigger alphabetically
        result.sort_by(|a, b| a.trigger.to_lowercase().cmp(&b.trigger.to_lowercase()));
        result
    }

    /// Retrieve a single macro by ID.
    pub fn get_macro_by_id(&self, id: &str) -> Result<Macro, EngineError> {
        self.id_map.get(id).cloned().ok_or_else(|| EngineError {
            code: "MACRO_NOT_FOUND".into(),
            message: format!("No macro exists with the given ID: {}", id),
        })
    }

    /// Create a new macro and save it.
    pub fn create_macro(&mut self, req: MacroCreateRequest) -> Result<Macro, EngineError> {
        // Validation
        if req.trigger.is_empty() {
            return Err(EngineError {
                code: "VALIDATION_ERROR".into(),
                message: "Trigger is required".into(),
            });
        }
        if req.trigger.len() < 2 || req.trigger.len() > 50 {
            return Err(EngineError {
                code: "INVALID_TRIGGER".into(),
                message: "Trigger must be 2-50 characters".into(),
            });
        }
        if req.action_type == ActionType::InsertText && req.content.is_empty() {
            return Err(EngineError {
                code: "VALIDATION_ERROR".into(),
                message: "Content is required".into(),
            });
        }
        if self.trigger_map.contains_key(&req.trigger) {
            return Err(EngineError {
                code: "TRIGGER_EXISTS".into(),
                message: "Trigger already exists".into(),
            });
        }

        let mut m = Macro::new(req.trigger, req.content);
        m.category = req.category;
        m.action_type = req.action_type;
        m.description = req.description.unwrap_or_default();
        m.preserve_format = req.preserve_format.unwrap_or(true);
        m.tags = req.tags.unwrap_or_default();
        m.shortcut = req.shortcut;
        m.event_trigger = req.event_trigger;

        self.insert_macro(m.clone());

        if let Err(e) = self.persist_macros() {
            return Err(EngineError {
                code: "STORAGE_WRITE_ERROR".into(),
                message: e.to_string(),
            });
        }

        Ok(m)
    }

    /// Update an existing macro.
    pub fn update_macro(&mut self, req: MacroUpdateRequest) -> Result<Macro, EngineError> {
        let mut m = self.get_macro_by_id(&req.id)?;

        let old_trigger = m.trigger.clone();
        let mut trigger_changed = false;

        if let Some(new_trigger) = req.trigger {
            if new_trigger != old_trigger {
                if new_trigger.is_empty() {
                    return Err(EngineError {
                        code: "VALIDATION_ERROR".into(),
                        message: "Trigger is required".into(),
                    });
                }
                if new_trigger.len() < 2 || new_trigger.len() > 50 {
                    return Err(EngineError {
                        code: "INVALID_TRIGGER".into(),
                        message: "Trigger must be 2-50 characters".into(),
                    });
                }
                if self.trigger_map.contains_key(&new_trigger) {
                    return Err(EngineError {
                        code: "TRIGGER_EXISTS".into(),
                        message: "Trigger already exists".into(),
                    });
                }
                m.trigger = new_trigger;
                trigger_changed = true;
            }
        }

        if let Some(new_content) = req.content {
            let action_type = req.action_type.clone().unwrap_or(m.action_type.clone());
            if action_type == ActionType::InsertText && new_content.is_empty() {
                return Err(EngineError {
                    code: "VALIDATION_ERROR".into(),
                    message: "Content is required".into(),
                });
            }
            m.content = new_content;
        }

        if let Some(desc) = req.description {
            m.description = desc;
        }
        if let Some(enabled) = req.enabled {
            m.enabled = enabled;
        }
        if let Some(category) = req.category {
            m.category = category;
        }
        if let Some(action) = req.action_type {
            m.action_type = action;
        }
        if let Some(pf) = req.preserve_format {
            m.preserve_format = pf;
        }
        if let Some(tags) = req.tags {
            m.tags = tags;
        }
        if let Some(sc) = req.shortcut {
            m.shortcut = sc;
        }
        if let Some(et) = req.event_trigger {
            m.event_trigger = et;
        }

        m.touch();

        if trigger_changed {
            self.trigger_map.remove(&old_trigger);
        }
        self.insert_macro(m.clone());

        if let Err(e) = self.persist_macros() {
            return Err(EngineError {
                code: "STORAGE_WRITE_ERROR".into(),
                message: e.to_string(),
            });
        }

        Ok(m)
    }

    /// Delete a macro by ID.
    pub fn delete_macro(&mut self, id: &str) -> Result<String, EngineError> {
        let m = self.get_macro_by_id(id)?;

        self.trigger_map.remove(&m.trigger);
        self.id_map.remove(&m.id);

        if let Err(e) = self.persist_macros() {
            return Err(EngineError {
                code: "STORAGE_WRITE_ERROR".into(),
                message: e.to_string(),
            });
        }

        Ok(id.to_string())
    }

    /// Toggle macro enabled state.
    pub fn toggle_macro(
        &mut self,
        id: &str,
        new_state: bool,
    ) -> Result<(String, bool), EngineError> {
        let mut m = self.get_macro_by_id(id)?;
        m.enabled = new_state;
        m.touch();

        self.insert_macro(m);

        if let Err(e) = self.persist_macros() {
            return Err(EngineError {
                code: "STORAGE_WRITE_ERROR".into(),
                message: e.to_string(),
            });
        }

        Ok((id.to_string(), new_state))
    }

    /// Search macros by query string.
    pub fn search_macros(&self, query: &str) -> Vec<Macro> {
        let mut all_macros: Vec<Macro> = self.id_map.values().cloned().collect();
        if query.is_empty() {
            all_macros.sort_by(|a, b| a.trigger.to_lowercase().cmp(&b.trigger.to_lowercase()));
            return all_macros;
        }

        let q = query.to_lowercase();

        let mut results = Vec::new();
        for m in all_macros {
            let t = m.trigger.to_lowercase();
            let d = m.description.to_lowercase();
            let matches_tag = m.tags.iter().any(|tag| tag.to_lowercase().contains(&q));

            if t.contains(&q) || d.contains(&q) || matches_tag {
                results.push(m);
            }
        }

        // Sort by relevance
        results.sort_by(|a, b| {
            let t_a = a.trigger.to_lowercase();
            let t_b = b.trigger.to_lowercase();

            let a_exact = t_a == q;
            let b_exact = t_b == q;

            if a_exact && !b_exact {
                return std::cmp::Ordering::Less;
            }
            if !a_exact && b_exact {
                return std::cmp::Ordering::Greater;
            }

            let a_starts = t_a.starts_with(&q);
            let b_starts = t_b.starts_with(&q);

            if a_starts && !b_starts {
                return std::cmp::Ordering::Less;
            }
            if !a_starts && b_starts {
                return std::cmp::Ordering::Greater;
            }

            let a_contains_trigger = t_a.contains(&q);
            let b_contains_trigger = t_b.contains(&q);

            if a_contains_trigger && !b_contains_trigger {
                return std::cmp::Ordering::Less;
            }
            if !a_contains_trigger && b_contains_trigger {
                return std::cmp::Ordering::Greater;
            }

            t_a.cmp(&t_b)
        });

        results
    }

    /// Get current configuration.
    pub fn get_config(&self) -> Config {
        self.config.clone()
    }

    /// Update configuration.
    pub fn update_config(&mut self, config: Config) -> Result<Config, EngineError> {
        if config.trigger_prefix.is_empty() {
            return Err(EngineError {
                code: "VALIDATION_ERROR".into(),
                message: "Trigger prefix is required".into(),
            });
        }
        if config.typing_buffer_size < 1 || config.typing_buffer_size > 500 {
            return Err(EngineError {
                code: "VALIDATION_ERROR".into(),
                message: "Buffer size out of range".into(),
            });
        }
        if config.notification_duration_ms < 500 || config.notification_duration_ms > 30000 {
            return Err(EngineError {
                code: "VALIDATION_ERROR".into(),
                message: "Duration out of range".into(),
            });
        }
        if config.command_palette_shortcut.is_empty() {
            return Err(EngineError {
                code: "VALIDATION_ERROR".into(),
                message: "Invalid shortcut format".into(),
            });
        }

        self.config = config;
        if let Err(e) = self.storage.save_config(&self.config) {
            return Err(EngineError {
                code: "STORAGE_WRITE_ERROR".into(),
                message: e.to_string(),
            });
        }

        Ok(self.config.clone())
    }

    /// Execute a macro by ID (manual execution from UI / Command Palette).
    pub fn execute_macro(&self, id: &str) -> Result<ExecutionResult, EngineError> {
        let macro_data = self.get_macro_by_id(id)?;
        ActionExecutor::execute_manual(&macro_data)
    }

    /// Execute a macro as if it was triggered by typing (with trigger deletion).
    pub fn execute_typed_trigger(
        &self,
        macro_data: &Macro,
        trigger_length: usize,
    ) -> Result<ExecutionResult, EngineError> {
        ActionExecutor::execute_typed_trigger(macro_data, trigger_length)
    }

    /// Execute a macro as if it was triggered by an event (no trigger deletion).
    pub fn execute_event_trigger(
        &self,
        macro_data: &Macro,
    ) -> Result<ExecutionResult, EngineError> {
        ActionExecutor::execute_event_trigger(macro_data)
    }

    // --- Private Helpers ---

    fn insert_macro(&mut self, m: Macro) {
        self.trigger_map.insert(m.trigger.clone(), m.clone());
        self.id_map.insert(m.id.clone(), m);
    }

    fn persist_macros(&self) -> Result<(), crate::storage::error::StorageError> {
        let macros: Vec<Macro> = self.id_map.values().cloned().collect();
        self.storage.save_macros(&macros)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn get_test_storage(name: &str) -> StorageManager {
        let dir = std::env::temp_dir()
            .join("textmacro_engine_tests")
            .join(name);
        let _ = fs::remove_dir_all(&dir);
        let storage = StorageManager::with_dir(dir);
        storage.initialize();
        storage
    }

    #[test]
    fn test_engine_init() {
        let storage = get_test_storage("init");
        let engine = Engine::new(storage);
        assert!(engine.get_macros(MacroCategory::Text).is_empty());
    }

    #[test]
    fn test_create_and_get_macro() {
        let storage = get_test_storage("create_get");
        let mut engine = Engine::new(storage);

        let req = MacroCreateRequest {
            trigger: "/test".into(),
            content: "Test Content".into(),
            category: MacroCategory::Text,
            action_type: ActionType::InsertText,
            description: None,
            preserve_format: None,
            tags: None,
            shortcut: None,
            event_trigger: None,
        };

        let result = engine.create_macro(req).unwrap();
        assert_eq!(result.trigger, "/test");

        let macros = engine.get_macros(MacroCategory::Text);
        assert_eq!(macros.len(), 1);
        assert_eq!(macros[0].id, result.id);
    }
    #[test]
    fn test_update_macro() {
        let storage = get_test_storage("update");
        let mut engine = Engine::new(storage);

        let m = engine
            .create_macro(MacroCreateRequest {
                trigger: "/upd".into(),
                content: "val".into(),
                category: MacroCategory::Text,
                action_type: ActionType::InsertText,
                description: None,
                preserve_format: None,
                tags: None,
                shortcut: None,
                event_trigger: None,
            })
            .unwrap();

        let req = MacroUpdateRequest {
            id: m.id.clone(),
            trigger: Some("/upd2".into()),
            content: Some("val2".into()),
            description: None,
            enabled: None,
            category: None,
            action_type: None,
            preserve_format: None,
            tags: None,
            shortcut: None,
            event_trigger: None,
        };

        let m2 = engine.update_macro(req).unwrap();
        assert_eq!(m2.trigger, "/upd2");
        assert_eq!(m2.content, "val2");

        assert!(engine.get_macro_by_id(&m.id).is_ok());
    }

    #[test]
    fn test_delete_macro() {
        let storage = get_test_storage("delete");
        let mut engine = Engine::new(storage);

        let m = engine
            .create_macro(MacroCreateRequest {
                trigger: "/del".into(),
                content: "del".into(),
                category: MacroCategory::Text,
                action_type: ActionType::InsertText,
                description: None,
                preserve_format: None,
                tags: None,
                shortcut: None,
                event_trigger: None,
            })
            .unwrap();

        assert!(engine.delete_macro(&m.id).is_ok());
        assert!(engine.get_macro_by_id(&m.id).is_err());
    }
}
