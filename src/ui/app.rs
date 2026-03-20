use iced::widget::{
    button, column, container, horizontal_space, mouse_area, row, text, Space,
};
use iced::{
    alignment, executor, theme, window, Application, Background, Border, Color, Command, Element,
    Font, Length, Settings, Size, Subscription, Theme,
};
use iced::window::Event as WindowEvent;
use iced::keyboard;
use iced::keyboard::key::Named;
use iced::Event as IcedEvent;

use crate::models::macro_model::{Macro, MacroCategory, ActionType};
use crate::models::config::Config;
use crate::storage::macro_repository::StorageManager;
use crate::ui::macro_list;
use crate::ui::macro_editor;
use crate::ui::settings_panel;
use crate::ui::overlays::{Toast, ToastType, CommandPaletteState};
use iced::widget::text_editor;
pub const BACKGROUND: Color = Color::from_rgb(0.059, 0.067, 0.082); // #0F1115
pub const PANEL: Color = Color::from_rgb(0.086, 0.102, 0.13); // #161A21
pub const CARD: Color = Color::from_rgb(0.118, 0.137, 0.169); // #1E232B
pub const BORDER: Color = Color::from_rgb(0.165, 0.184, 0.22); // #2A2F38
pub const ACCENT: Color = Color::from_rgb(0.486, 0.549, 1.0); // #7C8CFF
pub const TEXT_PRIMARY: Color = Color::from_rgb(0.902, 0.918, 0.949); // #E6EAF2
pub const TEXT_SECONDARY: Color = Color::from_rgb(0.608, 0.639, 0.698); // #9BA3B2
pub const SUCCESS: Color = Color::from_rgb(0.29, 0.871, 0.502); // #4ADE80
pub const ERROR: Color = Color::from_rgb(0.973, 0.443, 0.443); // #F87171
pub const CONTROL_HOVER: Color = Color::from_rgb(0.165, 0.184, 0.22); // #2A2F38

pub fn run() -> iced::Result {
    let settings = Settings {
        window: window::Settings {
            size: Size::new(1200.0, 800.0),
            min_size: Some(Size::new(800.0, 500.0)),
            decorations: false,
            transparent: true,
            position: window::Position::Centered,
            ..window::Settings::default()
        },
        ..Settings::default()
    };
    TextMacroApp::run(settings)
}

struct MainContainerStyle;
impl container::StyleSheet for MainContainerStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(BACKGROUND)),
            text_color: Some(TEXT_PRIMARY),
            ..Default::default()
        }
    }
}

struct SidebarStyle;
impl container::StyleSheet for SidebarStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(PANEL)),
            border: Border {
                color: BORDER,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }
}

struct TitleBarStyle;
impl container::StyleSheet for TitleBarStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(PANEL)),
            ..Default::default()
        }
    }
}

struct AccentBarStyle;
impl container::StyleSheet for AccentBarStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(ACCENT)),
            ..Default::default()
        }
    }
}

struct FocusOutlineStyle;
impl container::StyleSheet for FocusOutlineStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            border: Border {
                color: TEXT_SECONDARY,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }
}

struct CenterPanelStyle;
impl container::StyleSheet for CenterPanelStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(BACKGROUND)),
            border: Border {
                color: BORDER,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }
}

struct WindowControlStyle(bool);
impl button::StyleSheet for WindowControlStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: TEXT_PRIMARY,
            border: Border::default(),
            ..Default::default()
        }
    }
    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(if self.0 { ERROR } else { CONTROL_HOVER })),
            text_color: TEXT_PRIMARY,
            border: Border::default(),
            ..Default::default()
        }
    }
}

struct SidebarItemButtonStyle {
    is_active: bool,
}
impl button::StyleSheet for SidebarItemButtonStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(if self.is_active { CARD } else { Color::TRANSPARENT })),
            text_color: if self.is_active { TEXT_PRIMARY } else { TEXT_SECONDARY },
            border: Border::default(),
            ..Default::default()
        }
    }
    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(CARD)),
            text_color: TEXT_PRIMARY,
            border: Border::default(),
            ..Default::default()
        }
    }
}

pub struct TextMacroApp {
    active_sidebar: usize,
    focused_sidebar: Option<usize>,
    window_maximized: bool,
    window_width: f32,
    macros: Vec<Macro>,
    search_query: String,
    selected_macro_id: Option<String>,
    _storage: StorageManager,
    editor_state: EditorState,
    pending_navigation: Option<Box<Message>>,
    show_delete_dialog: bool,
    config: Config,
    config_validation_errors: std::collections::HashMap<String, String>,
    is_recording_shortcut: bool,
    command_palette: CommandPaletteState,
    toasts: Vec<Toast>,
}

#[derive(Debug)]
pub struct EditorState {
    pub is_active: bool,
    pub is_new: bool,
    pub original_id: Option<String>,
    pub trigger: String,
    pub description: String,
    pub content: text_editor::Content,
    pub enabled: bool,
    pub has_unsaved_changes: bool,
    pub validation_error: Option<String>,
}

impl EditorState {
    pub fn default() -> Self {
        Self {
            is_active: false,
            is_new: false,
            original_id: None,
            trigger: String::new(),
            description: String::new(),
            content: text_editor::Content::new(),
            enabled: true,
            has_unsaved_changes: false,
            validation_error: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    SidebarSelected(usize),
    TitleBarDragged,
    MinimizeClicked,
    MaximizeClicked,
    CloseClicked,
    WindowResized(u32, u32),
    KeyPressed(keyboard::Key, keyboard::Modifiers),
    SearchQueryChanged(String),
    ClearSearch,
    SelectMacro(String),
    NewMacroClick,
    EditorTriggerChanged(String),
    EditorDescriptionChanged(String),
    EditorContentAction(text_editor::Action),
    EditorEnabledToggled(bool),
    SaveMacro,
    DeleteMacroClick,
    ConfirmDelete,
    CancelDelete,
    ConfirmDiscard,
    CancelDiscard,
    // Settings
    ToggleRunOnStartup(bool),
    ToggleBackgroundService(bool),
    TriggerPrefixChanged(String),
    TriggerPrefixSubmit,
    ToggleEditorFontMonospace(bool),
    TogglePreserveFormatting(bool),
    ToggleMarkdownSupport(bool),
    ThemeSelected(String),
    UIDensitySelected(String),
    StartShortcutRecording,
    CancelShortcutRecording,
    // Overlays
    ToggleCommandPalette,
    CommandPaletteQueryChanged(String),
    CommandPaletteExecute,
    CommandPaletteSelectUp,
    CommandPaletteSelectDown,
    ToggleMacroEnabledReq(String),
    DuplicateMacroReq(String),
    RequestDeleteMacroReq(String),
    DismissToast(uuid::Uuid),
    AddToast(ToastType, String),
    TickToasts(std::time::Instant),
}

const SIDEBAR_ITEMS: &[(&str, &str)] = &[
    ("⚡", "Macros"),
    ("📄", "Prompts"),
    ("🕐", "Events"),
    ("⚙️", "Settings"),
];

impl Application for TextMacroApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let storage = StorageManager::new().expect("Failed to initialize storage");
        let _ = storage.initialize(); // Ignore warnings
        let (macros, _) = storage.load_macros();
        let (config, _) = storage.load_config();
        
        (
            Self {
                active_sidebar: 0,
                focused_sidebar: None,
                window_maximized: false,
                window_width: 1200.0,
                macros,
                search_query: String::new(),
                selected_macro_id: None,
                _storage: storage,
                editor_state: EditorState::default(),
                pending_navigation: None,
                show_delete_dialog: false,
                config,
                config_validation_errors: std::collections::HashMap::new(),
                is_recording_shortcut: false,
                command_palette: CommandPaletteState::default(),
                toasts: Vec::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("TextMacro")
    }

    fn theme(&self) -> Theme {
        match self.config.theme.as_str() {
            "light" => Theme::Light,
            _ => Theme::Dark,
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let subs = vec![
            iced::event::listen_with(|event, _status| match event {
                IcedEvent::Window(_, WindowEvent::Resized { width, height }) => {
                    Some(Message::WindowResized(width, height))
                }
                IcedEvent::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                    Some(Message::KeyPressed(key.clone(), modifiers))
                }
                _ => None,
            }),
        ];
        
        Subscription::batch(subs)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SidebarSelected(idx) => {
                if self.editor_state.has_unsaved_changes {
                    self.pending_navigation = Some(Box::new(Message::SidebarSelected(idx)));
                    return Command::none();
                }
                self.active_sidebar = idx;
                self.search_query.clear();
                self.selected_macro_id = None;
                self.editor_state = EditorState::default();
                Command::none()
            }
            Message::TitleBarDragged => window::drag(window::Id::MAIN),
            Message::MinimizeClicked => window::minimize(window::Id::MAIN, true),
            Message::MaximizeClicked => {
                self.window_maximized = !self.window_maximized;
                window::maximize(window::Id::MAIN, self.window_maximized)
            }
            Message::CloseClicked => window::close(window::Id::MAIN),
            Message::WindowResized(w, _h) => {
                self.window_width = w as f32;
                Command::none()
            }
            Message::KeyPressed(key, modifiers) => {
                let mut current_combo = String::new();
                if modifiers.control() { current_combo.push_str("Ctrl+"); }
                if modifiers.alt() { current_combo.push_str("Alt+"); }
                if modifiers.shift() { current_combo.push_str("Shift+"); }
                if modifiers.logo() { current_combo.push_str("Meta+"); }
                
                let key_str = match key.as_ref() {
                    keyboard::Key::Named(Named::Escape) => Some("Esc"),
                    keyboard::Key::Named(Named::Enter) => Some("Enter"),
                    keyboard::Key::Named(Named::Space) => Some("Space"),
                    keyboard::Key::Character(s) => Some(s.as_ref()),
                    _ => None,
                };
                
                if let Some(s) = key_str {
                    let shortcut = format!("{}{}", current_combo, s.to_uppercase());
                    let target = if self.config.command_palette_shortcut.is_empty() { "Ctrl+Shift+P" } else { &self.config.command_palette_shortcut };
                    if shortcut == target && !self.is_recording_shortcut {
                        return self.update(Message::ToggleCommandPalette);
                    }
                }

                if self.command_palette.is_open {
                    match key.as_ref() {
                        keyboard::Key::Named(Named::Escape) => {
                            self.command_palette.is_open = false;
                            return Command::none();
                        }
                        keyboard::Key::Named(Named::ArrowUp) => return self.update(Message::CommandPaletteSelectUp),
                        keyboard::Key::Named(Named::ArrowDown) => return self.update(Message::CommandPaletteSelectDown),
                        keyboard::Key::Named(Named::Enter) => return self.update(Message::CommandPaletteExecute),
                        _ => {}
                    }
                    return Command::none();
                }

                if self.is_recording_shortcut {
                    if matches!(key.as_ref(), keyboard::Key::Named(Named::Escape)) {
                        self.is_recording_shortcut = false;
                        return Command::none();
                    }
                    if !modifiers.is_empty() {
                        let mut combo = String::new();
                        if modifiers.control() { combo.push_str("Ctrl+"); }
                        if modifiers.alt() { combo.push_str("Alt+"); }
                        if modifiers.shift() { combo.push_str("Shift+"); }
                        if modifiers.logo() { combo.push_str("Meta+"); }
                        
                        let key_str = match key.as_ref() {
                            keyboard::Key::Named(Named::Escape) => Some("Esc"),
                            keyboard::Key::Named(Named::Enter) => Some("Enter"),
                            keyboard::Key::Named(Named::Space) => Some("Space"),
                            keyboard::Key::Character(s) => Some(s.as_ref()),
                            _ => None,
                        };
                        
                        if let Some(s) = key_str {
                            let shortcut = format!("{}{}", combo, s.to_uppercase());
                            self.config.command_palette_shortcut = shortcut;
                            self.is_recording_shortcut = false;
                            self.config_validation_errors.remove("command_palette_shortcut");
                            let _ = self._storage.save_config(&self.config);
                        }
                    } else if matches!(key.as_ref(), keyboard::Key::Character(_)) || matches!(key.as_ref(), keyboard::Key::Named(_)) {
                        // User pressed a key without modifier
                        self.config_validation_errors.insert(
                            "command_palette_shortcut".to_string(), 
                            "Must include Ctrl, Alt, or Shift".to_string()
                        );
                    }
                    return Command::none();
                }

                match key.as_ref() {
                    keyboard::Key::Named(Named::ArrowUp) => {
                        if window::Id::MAIN == window::Id::MAIN { // Dummy check just an excuse for a block, but we want to focus center if selected macro exists.
                            // Actually, let's just make up/down toggle the macro selection if we are on the center panel.
                            // Better: if center is focused, or we just globally use up/down if sidebar isn't active.
                            // To keep it simple, if no macro is selected, we let sidebar move. If macro selected, macro list moves.
                            if self.selected_macro_id.is_some() {
                                let category = match self.active_sidebar {
                                    1 => MacroCategory::Prompt,
                                    2 => MacroCategory::Event,
                                    _ => MacroCategory::Text,
                                };
                                let filtered: Vec<&Macro> = self.macros.iter().filter(|m| m.category == category && (self.search_query.is_empty() || m.trigger.to_lowercase().contains(&self.search_query.to_lowercase()) || m.description.to_lowercase().contains(&self.search_query.to_lowercase()))).collect();
                                if let Some(id) = &self.selected_macro_id {
                                    if let Some(pos) = filtered.iter().position(|m| &m.id == id) {
                                        if pos > 0 {
                                            self.selected_macro_id = Some(filtered[pos - 1].id.clone());
                                        }
                                    }
                                }
                            } else {
                                let current = self.focused_sidebar.unwrap_or(self.active_sidebar);
                                if current > 0 {
                                    self.focused_sidebar = Some(current - 1);
                                }
                            }
                        }
                    }
                    keyboard::Key::Named(Named::ArrowDown) => {
                        if self.selected_macro_id.is_some() {
                            let category = match self.active_sidebar {
                                1 => MacroCategory::Prompt,
                                2 => MacroCategory::Event,
                                _ => MacroCategory::Text,
                            };
                            let filtered: Vec<&Macro> = self.macros.iter().filter(|m| m.category == category && (self.search_query.is_empty() || m.trigger.to_lowercase().contains(&self.search_query.to_lowercase()) || m.description.to_lowercase().contains(&self.search_query.to_lowercase()))).collect();
                            if let Some(id) = &self.selected_macro_id {
                                if let Some(pos) = filtered.iter().position(|m| &m.id == id) {
                                    if pos < filtered.len() - 1 {
                                        self.selected_macro_id = Some(filtered[pos + 1].id.clone());
                                    }
                                }
                            }
                        } else {
                            let current = self.focused_sidebar.unwrap_or(self.active_sidebar);
                            if current < 3 {
                                self.focused_sidebar = Some(current + 1);
                            }
                        }
                    }
                    keyboard::Key::Named(Named::Enter) => {
                        if let Some(focused) = self.focused_sidebar {
                            self.active_sidebar = focused;
                        }
                    }
                    keyboard::Key::Named(Named::Home) => {
                        if self.selected_macro_id.is_some() {
                            let category = match self.active_sidebar {
                                1 => MacroCategory::Prompt,
                                2 => MacroCategory::Event,
                                _ => MacroCategory::Text,
                            };
                            let filtered: Vec<&Macro> = self.macros.iter().filter(|m| m.category == category).collect();
                            if let Some(first) = filtered.first() {
                                self.selected_macro_id = Some(first.id.clone());
                            }
                        }
                    }
                    keyboard::Key::Named(Named::End) => {
                        if self.selected_macro_id.is_some() {
                            let category = match self.active_sidebar {
                                1 => MacroCategory::Prompt,
                                2 => MacroCategory::Event,
                                _ => MacroCategory::Text,
                            };
                            let filtered: Vec<&Macro> = self.macros.iter().filter(|m| m.category == category).collect();
                            if let Some(last) = filtered.last() {
                                self.selected_macro_id = Some(last.id.clone());
                            }
                        }
                    }
                    keyboard::Key::Named(Named::Escape) => {
                        self.search_query.clear();
                    }
                    _ => {}
                }
                Command::none()
            }
            Message::SearchQueryChanged(q) => {
                self.search_query = q;
                Command::none()
            }
            Message::ClearSearch => {
                self.search_query.clear();
                Command::none()
            }
            Message::SelectMacro(id) => {
                if self.editor_state.has_unsaved_changes {
                    self.pending_navigation = Some(Box::new(Message::SelectMacro(id)));
                    return Command::none();
                }
                
                self.selected_macro_id = Some(id.clone());
                self.show_delete_dialog = false;
                
                if let Some(m) = self.macros.iter().find(|m| m.id == id) {
                    self.editor_state = EditorState {
                        is_active: true,
                        is_new: false,
                        original_id: Some(id),
                        trigger: m.trigger.clone(),
                        description: m.description.clone(),
                        content: text_editor::Content::with_text(&m.content),
                        enabled: m.enabled,
                        has_unsaved_changes: false,
                        validation_error: None,
                    };
                }
                Command::none()
            }
            Message::NewMacroClick => {
                if self.editor_state.has_unsaved_changes {
                    self.pending_navigation = Some(Box::new(Message::NewMacroClick));
                    return Command::none();
                }
                self.selected_macro_id = None;
                self.show_delete_dialog = false;
                self.editor_state = EditorState {
                    is_active: true,
                    is_new: true,
                    original_id: None,
                    trigger: String::new(),
                    description: String::new(),
                    content: text_editor::Content::new(),
                    enabled: true,
                    has_unsaved_changes: true, // Mark a new macro as having unsaved changes to enable save button maybe? No, let's wait until they type.
                    validation_error: None,
                };
                self.editor_state.has_unsaved_changes = false;
                Command::none()
            }
            Message::EditorTriggerChanged(trigger) => {
                self.editor_state.trigger = trigger;
                self.editor_state.has_unsaved_changes = true;
                if let Some(err) = &self.editor_state.validation_error {
                    if err.contains("Trigger") {
                        self.editor_state.validation_error = None;
                    }
                }
                Command::none()
            }
            Message::EditorDescriptionChanged(desc) => {
                self.editor_state.description = desc;
                self.editor_state.has_unsaved_changes = true;
                Command::none()
            }
            Message::EditorContentAction(action) => {
                self.editor_state.content.perform(action);
                self.editor_state.has_unsaved_changes = true;
                if let Some(err) = &self.editor_state.validation_error {
                    if err.contains("Content") {
                        self.editor_state.validation_error = None;
                    }
                }
                Command::none()
            }
            Message::EditorEnabledToggled(enabled) => {
                self.editor_state.enabled = enabled;
                self.editor_state.has_unsaved_changes = true;
                Command::none()
            }
            Message::SaveMacro => {
                if self.editor_state.trigger.is_empty() {
                    self.editor_state.validation_error = Some("Trigger is required".to_string());
                    return Command::none();
                }
                
                let is_duplicate = self.macros.iter().any(|m| {
                    m.trigger == self.editor_state.trigger && 
                    Some(&m.id) != self.editor_state.original_id.as_ref()
                });
                
                if is_duplicate {
                    self.editor_state.validation_error = Some("Trigger already exists".to_string());
                    return Command::none();
                }

                let content_str = self.editor_state.content.text();
                // We could validate content empty here, but sometimes empty content is okay for event macros.
                // Let's just create/update the macro
                if self.editor_state.is_new {
                    let mut new_macro = Macro::new(self.editor_state.trigger.clone(), content_str);
                    new_macro.description = self.editor_state.description.clone();
                    new_macro.enabled = self.editor_state.enabled;
                    new_macro.category = match self.active_sidebar {
                        1 => MacroCategory::Prompt,
                        2 => MacroCategory::Event,
                        _ => MacroCategory::Text,
                    };
                    self.selected_macro_id = Some(new_macro.id.clone());
                    self.macros.push(new_macro);
                    self.editor_state.is_new = false;
                    self.editor_state.original_id = self.selected_macro_id.clone();
                } else {
                    if let Some(id) = &self.editor_state.original_id {
                        if let Some(m) = self.macros.iter_mut().find(|m| &m.id == id) {
                            m.trigger = self.editor_state.trigger.clone();
                            m.description = self.editor_state.description.clone();
                            m.content = content_str;
                            m.enabled = self.editor_state.enabled;
                            m.touch();
                        }
                    }
                }
                
                self.editor_state.has_unsaved_changes = false;
                self.editor_state.validation_error = None;
                
                let _ = self._storage.save_macros(&self.macros);
                // Also trigger macro_engine refresh, but this is future phase.
                Command::none()
            }
            Message::DeleteMacroClick => {
                self.show_delete_dialog = true;
                Command::none()
            }
            Message::ConfirmDelete => {
                if let Some(id) = &self.editor_state.original_id {
                    self.macros.retain(|m| &m.id != id);
                    let _ = self._storage.save_macros(&self.macros);
                }
                self.editor_state = EditorState::default();
                self.selected_macro_id = None;
                self.show_delete_dialog = false;
                Command::none()
            }
            Message::CancelDelete => {
                self.show_delete_dialog = false;
                Command::none()
            }
            Message::ConfirmDiscard => {
                self.editor_state.has_unsaved_changes = false;
                if let Some(msg) = self.pending_navigation.take() {
                    return self.update(*msg);
                }
                Command::none()
            }
            Message::CancelDiscard => {
                self.pending_navigation = None;
                Command::none()
            }
            // Settings Implementations
            Message::ToggleRunOnStartup(b) => {
                self.config.run_on_startup = b;
                let _ = self._storage.save_config(&self.config);
                Command::none()
            }
            Message::ToggleBackgroundService(b) => {
                self.config.enable_background_service = b;
                let _ = self._storage.save_config(&self.config);
                Command::none()
            }
            Message::TriggerPrefixChanged(val) => {
                self.config.trigger_prefix = val;
                if self.config.trigger_prefix.is_empty() {
                    self.config_validation_errors.insert("trigger_prefix".into(), "Trigger prefix is required".into());
                } else {
                    self.config_validation_errors.remove("trigger_prefix");
                    let _ = self._storage.save_config(&self.config);
                }
                Command::none()
            }
            Message::TriggerPrefixSubmit => {
                Command::none()
            }
            Message::ToggleEditorFontMonospace(b) => {
                self.config.editor_font_monospace = b;
                let _ = self._storage.save_config(&self.config);
                Command::none()
            }
            Message::TogglePreserveFormatting(b) => {
                self.config.preserve_formatting = b;
                let _ = self._storage.save_config(&self.config);
                Command::none()
            }
            Message::ToggleMarkdownSupport(b) => {
                self.config.markdown_support = b;
                let _ = self._storage.save_config(&self.config);
                Command::none()
            }
            Message::ThemeSelected(val) => {
                self.config.theme = val;
                let _ = self._storage.save_config(&self.config);
                Command::none()
            }
            Message::UIDensitySelected(val) => {
                self.config.ui_density = val;
                let _ = self._storage.save_config(&self.config);
                Command::none()
            }
            Message::StartShortcutRecording => {
                self.is_recording_shortcut = true;
                Command::none()
            }
            Message::CancelShortcutRecording => {
                self.is_recording_shortcut = false;
                Command::none()
            }
            Message::ToggleCommandPalette => {
                self.command_palette.is_open = !self.command_palette.is_open;
                if self.command_palette.is_open {
                    self.command_palette.query.clear();
                    self.command_palette.selected_index = 0;
                }
                Command::none()
            }
            Message::CommandPaletteQueryChanged(q) => {
                self.command_palette.query = q;
                self.command_palette.selected_index = 0;
                Command::none()
            }
            Message::CommandPaletteSelectUp => {
                if self.command_palette.selected_index > 0 {
                    self.command_palette.selected_index -= 1;
                }
                Command::none()
            }
            Message::CommandPaletteSelectDown => {
                self.command_palette.selected_index += 1;
                Command::none()
            }
            Message::CommandPaletteExecute => {
                let lower_query = self.command_palette.query.to_lowercase();
                let mut filtered: Vec<&Macro> = self.macros.iter().filter(|m| {
                    if lower_query.is_empty() { true } else {
                        m.trigger.to_lowercase().contains(&lower_query) || m.description.to_lowercase().contains(&lower_query)
                    }
                }).collect();
                filtered.sort_by(|a, b| a.trigger.cmp(&b.trigger));
                
                if let Some(m) = filtered.get(self.command_palette.selected_index) {
                    let t_id = uuid::Uuid::new_v4();
                    let mut t = Toast::new(format!("Executed: {}", m.trigger), ToastType::Success);
                    t.id = t_id.clone();
                    self.toasts.push(t);
                    self.command_palette.is_open = false;
                    return Command::perform(async move {
                        std::thread::sleep(std::time::Duration::from_millis(3000));
                        t_id
                    }, Message::DismissToast);
                }
                
                self.command_palette.is_open = false;
                Command::none()
            }
            Message::ToggleMacroEnabledReq(id) => {
                let mut enabled_str = None;
                if let Some(m) = self.macros.iter_mut().find(|m| m.id == id) {
                    m.enabled = !m.enabled;
                    enabled_str = Some(if m.enabled { "Macro enabled".to_string() } else { "Macro disabled".to_string() });
                }
                
                if let Some(s) = enabled_str {
                    let _ = self._storage.save_macros(&self.macros);
                    let m_id = id.clone();
                    self.toasts.push(Toast::new(s, ToastType::Info));
                    return Command::perform(async move {
                        std::thread::sleep(std::time::Duration::from_millis(3000));
                        m_id
                    }, |i| Message::DismissToast(uuid::Uuid::parse_str(&i).unwrap_or_default()));
                }
                Command::none()
            }
            Message::DuplicateMacroReq(id) => {
                if let Some(m) = self.macros.iter().find(|m| m.id == id).cloned() {
                    let mut duplicate = m.clone();
                    let new_id = uuid::Uuid::new_v4();
                    duplicate.id = new_id.to_string();
                    duplicate.trigger = format!("{}-copy", m.trigger);
                    duplicate.created_at = chrono::Utc::now().to_rfc3339();
                    duplicate.updated_at = duplicate.created_at.clone();
                    self.macros.push(duplicate);
                    let _ = self._storage.save_macros(&self.macros);
                    self.toasts.push(Toast::new("Macro duplicated".into(), ToastType::Success));
                    return Command::perform(async move {
                        std::thread::sleep(std::time::Duration::from_millis(3000));
                        new_id
                    }, Message::DismissToast);
                }
                Command::none()
            }
            Message::RequestDeleteMacroReq(id) => {
                self.macros.retain(|m| m.id != id);
                let _ = self._storage.save_macros(&self.macros);
                if self.selected_macro_id.as_deref() == Some(id.as_str()) {
                    self.selected_macro_id = None;
                    self.editor_state = EditorState::default();
                }
                let t_id = uuid::Uuid::new_v4();
                let mut t = Toast::new("Macro deleted".into(), ToastType::Warning);
                t.id = t_id.clone();
                self.toasts.push(t);
                return Command::perform(async move {
                    std::thread::sleep(std::time::Duration::from_millis(3000));
                    t_id
                }, Message::DismissToast);
            }
            Message::DismissToast(id) => {
                self.toasts.retain(|t| t.id != id);
                Command::none()
            }
            Message::AddToast(t_type, msg) => {
                self.toasts.push(Toast::new(msg, t_type));
                Command::none()
            }
            Message::TickToasts(_) => {
                self.toasts.retain(|t| t.created_at.elapsed() < t.duration);
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let is_collapsed = self.window_width > 800.0 && self.window_width < 1000.0;
        let is_hidden_right = self.window_width <= 800.0;
        let is_mobile = self.window_width < 800.0;
        let sidebar_width = if is_mobile || is_collapsed { 56.0 } else { 220.0 };

        // Title bar controls
        let btn_minimize = button(text("─").size(16).horizontal_alignment(alignment::Horizontal::Center))
            .width(Length::Fixed(46.0))
            .height(Length::Fill)
            .style(theme::Button::custom(WindowControlStyle(false)))
            .on_press(Message::MinimizeClicked);

        let btn_maximize = button(text(if self.window_maximized { "❐" } else { "□" }).size(16).horizontal_alignment(alignment::Horizontal::Center))
            .width(Length::Fixed(46.0))
            .height(Length::Fill)
            .style(theme::Button::custom(WindowControlStyle(false)))
            .on_press(Message::MaximizeClicked);

        let btn_close = button(text("✕").size(16).horizontal_alignment(alignment::Horizontal::Center))
            .width(Length::Fixed(46.0))
            .height(Length::Fill)
            .style(theme::Button::custom(WindowControlStyle(true)))
            .on_press(Message::CloseClicked);

        let title_bar = container(
            row![
                horizontal_space().width(Length::Fixed(16.0)),
                text("TextMacro").size(14).style(theme::Text::Color(TEXT_SECONDARY)),
                horizontal_space().width(Length::Fill),
                btn_minimize,
                btn_maximize,
                btn_close,
            ]
            .align_items(alignment::Alignment::Center)
            .height(Length::Fixed(40.0)),
        )
        .style(theme::Container::Custom(Box::new(TitleBarStyle)));

        let draggble_title_bar = mouse_area(title_bar).on_press(Message::TitleBarDragged);

        // Sidebar rendering
        let mut sidebar_column = column![].spacing(4);
        for (idx, (icon, label)) in SIDEBAR_ITEMS.iter().enumerate() {
            let is_active = self.active_sidebar == idx;
            let is_focused = self.focused_sidebar == Some(idx);
            
            let mut item_row = row![text(*icon).size(16)].align_items(alignment::Alignment::Center);

            if !is_collapsed && !is_mobile {
                item_row = item_row.push(horizontal_space().width(Length::Fixed(12.0)));
                item_row = item_row.push(
                    text(*label)
                        .size(15)
                        .style(if is_active { theme::Text::Color(TEXT_PRIMARY) } else { theme::Text::Color(TEXT_SECONDARY) })
                );
            }

            let btn = button(
                container(item_row)
                    .width(Length::Fill)
                    .padding(iced::Padding { top: 12.0, bottom: 12.0, left: 16.0, right: 16.0 })
            )
            .width(Length::Fill)
            // Passing `is_active` for primary styling; could extend stylesheet to handle `is_focused` explicitly
            // But standard iced button also adds slight outline when natively focused.
            // For custom drawn focus, we can modify the container's appearance, but let's just draw an outline via container if focused.
            .style(theme::Button::custom(SidebarItemButtonStyle { is_active }))
            .on_press(Message::SidebarSelected(idx));

            let final_btn_container = if is_focused && !is_active {
                // simple simulated focus outline
                container(btn).style(theme::Container::Custom(Box::new(FocusOutlineStyle)))
            } else {
                container(btn).style(theme::Container::Transparent)
            };

            let final_item = if is_active {
                let accent = container(Space::new(3.0, Length::Shrink))
                    .height(Length::Fill)
                    .style(theme::Container::Custom(Box::new(AccentBarStyle)));
                row![accent, final_btn_container].height(Length::Shrink)
            } else {
                row![Space::new(3.0, Length::Shrink), final_btn_container].height(Length::Shrink)
            };

            sidebar_column = sidebar_column.push(final_item);
        }

        let sidebar = container(sidebar_column)
            .width(Length::Fixed(sidebar_width))
            .height(Length::Fill)
            .style(theme::Container::Custom(Box::new(SidebarStyle)));

        let active_category = match self.active_sidebar {
            0 => MacroCategory::Text,
            1 => MacroCategory::Prompt,
            2 => MacroCategory::Event,
            _ => MacroCategory::Text, // Default to Text if settings/other
        };

        let content = if self.active_sidebar == 3 {
            let settings_view = settings_panel::view(
                &self.config,
                &self.config_validation_errors,
                self.is_recording_shortcut,
            );
            
            let settings_container = container(settings_view)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(theme::Container::Custom(Box::new(CenterPanelStyle)));
                
            row![sidebar, settings_container].height(Length::Fill)
        } else {
            let macro_list_view = macro_list::view(
                &self.macros,
                active_category,
                &self.search_query,
                self.selected_macro_id.as_deref(),
            );

            let center_panel = container(macro_list_view)
                .width(Length::FillPortion(35))
                .height(Length::Fill)
                .style(theme::Container::Custom(Box::new(CenterPanelStyle)));

            let right_panel: Element<'_, Message> = if is_hidden_right {
                Space::new(0.0, 0.0).into()
            } else if self.pending_navigation.is_some() {
                let dialog_box = column![
                    text("Unsaved Changes").size(20).style(theme::Text::Color(TEXT_PRIMARY)),
                    text("You have unsaved changes. Do you want to discard them?").size(14).style(theme::Text::Color(TEXT_SECONDARY)),
                    Space::new(0.0, 20.0),
                    row![
                        horizontal_space().width(Length::Fill),
                        button(text("Cancel").horizontal_alignment(alignment::Horizontal::Center).style(theme::Text::Color(TEXT_PRIMARY)))
                            .padding(10)
                            .width(Length::Fixed(100.0))
                            .style(theme::Button::custom(crate::ui::macro_list::ClearBtnStyle))
                            .on_press(Message::CancelDiscard),
                        Space::new(10.0, 0.0),
                        button(text("Discard").horizontal_alignment(alignment::Horizontal::Center).style(theme::Text::Color(ERROR)))
                            .padding(10)
                            .width(Length::Fixed(100.0))
                            .style(theme::Button::custom(crate::ui::macro_editor::DangerButtonStyle))
                            .on_press(Message::ConfirmDiscard),
                    ].align_items(alignment::Alignment::Center)
                ].padding(24);
                
                container(dialog_box)
                    .width(Length::FillPortion(65))
                    .height(Length::Fill)
                    .center_x()
                    .center_y()
                    .style(theme::Container::Custom(Box::new(CenterPanelStyle)))
                    .into()
            } else {
                container(macro_editor::view(&self.editor_state, self.show_delete_dialog, self.config.editor_font_monospace))
                    .width(Length::FillPortion(65))
                    .height(Length::Fill)
                    .style(theme::Container::Custom(Box::new(CenterPanelStyle)))
                    .into()
            };

            row![sidebar, center_panel, right_panel].height(Length::Fill)
        };
        
        let main_container: Element<'_, Message> = container(column![draggble_title_bar, content])
            .style(theme::Container::Custom(Box::new(MainContainerStyle)))
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
            
        let modal = crate::ui::overlays::view_command_palette(main_container, &self.command_palette, &self.macros);
        crate::ui::overlays::view_toasts(modal, &self.toasts)
    }
}
