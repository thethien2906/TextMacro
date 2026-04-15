use iced::widget::{
    button, column, container, horizontal_space, mouse_area, row, text, Space,
};
use iced::{
    alignment, executor, theme, window, Application, Background, Border, Color, Command, Element,
    Font, Length, Settings, Size, Subscription, Theme,
};
use iced::window::Event as WindowEvent;
use iced::window::Mode;
use iced::futures::SinkExt;
use iced::keyboard;
use iced::keyboard::key::Named;
use iced::Event as IcedEvent;

use crate::models::macro_model::{Macro, MacroCategory, ActionType};
use crate::models::config::Config;
use crate::storage::macro_repository::StorageManager;
use crate::ui::macro_list;
use crate::ui::macro_editor;
use crate::ui::settings_panel;
use crate::ui::overlays::{CommandPaletteState};
use iced::widget::text_editor;
// Editorial Macro Suite — Design System Color Tokens
pub const BACKGROUND: Color = Color::from_rgb(0.055, 0.055, 0.055);          // #0e0e0e  surface
pub const PANEL: Color = Color::from_rgb(0.075, 0.075, 0.075);              // #131313  surface-container-low
pub const CARD: Color = Color::from_rgb(0.125, 0.125, 0.122);               // #20201f  surface-container-high
pub const SURFACE_HIGHEST: Color = Color::from_rgb(0.149, 0.149, 0.149);    // #262626  surface-container-highest
pub const SURFACE_BRIGHT: Color = Color::from_rgb(0.173, 0.173, 0.173);     // #2c2c2c  surface-bright
pub const BORDER: Color = Color::from_rgba(0.282, 0.282, 0.278, 0.15);      // #484847 at 15% — ghost border
pub const ACCENT: Color = Color::from_rgb(0.639, 0.651, 1.0);               // #a3a6ff  primary (Electric Indigo)
pub const ACCENT_DIM: Color = Color::from_rgb(0.361, 0.373, 0.992);         // #5c5ffd  primary-dim
pub const SECONDARY: Color = Color::from_rgb(0.635, 0.557, 0.988);          // #a28efc  secondary (Muted lavender)
pub const TEXT_PRIMARY: Color = Color::from_rgb(1.0, 1.0, 1.0);             // #ffffff  on-surface
pub const TEXT_SECONDARY: Color = Color::from_rgb(0.678, 0.667, 0.667);     // #adaaaa  on-surface-variant
pub const SUCCESS: Color = Color::from_rgb(0.196, 0.784, 0.325);            // #32C853  emerald-500
pub const ERROR: Color = Color::from_rgb(1.0, 0.431, 0.518);                // #ff6e84  error
pub const ERROR_DIM: Color = Color::from_rgb(0.843, 0.2, 0.341);            // #d73357  error-dim
pub const CONTROL_HOVER: Color = Color::from_rgb(0.125, 0.125, 0.122);      // #20201f  surface-container-high

pub fn run(flags: (std::sync::mpsc::Sender<crate::models::engine_commands::EngineCommand>, std::sync::mpsc::Receiver<crate::models::engine_responses::EngineResponse>), rgba: Vec<u8>, width: u32, height: u32) -> iced::Result {
    let mut settings = Settings::with_flags((flags, rgba.clone(), width, height));
    settings.fonts.push(std::borrow::Cow::Borrowed(iced_aw::BOOTSTRAP_FONT_BYTES));
    settings.default_font = Font::with_name("Segoe UI");
    settings.window = window::Settings {
        size: Size::new(1000.0, 700.0),
        min_size: Some(Size::new(700.0, 450.0)),
        decorations: false,
        transparent: true,
        icon: Some(window::icon::from_rgba(rgba, width, height).expect("Failed to create icon")),
        position: window::Position::Centered,
        ..window::Settings::default()
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
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
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
                color: Color::TRANSPARENT,
                width: 0.0,
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
            background: Some(Background::Color(Color::from_rgb(0.388, 0.408, 0.98))),  // indigo-500 tint
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 2.0.into(),
            },
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
                color: Color::TRANSPARENT,
                width: 0.0,
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
        // Active = indigo tint bg; Inactive = transparent
        let bg = if self.is_active {
            Color::from_rgba(0.388, 0.408, 0.98, 0.10)   // indigo-500/10
        } else {
            Color::TRANSPARENT
        };
        let text_col = if self.is_active {
            Color::from_rgb(0.576, 0.596, 1.0)            // indigo-400
        } else {
            TEXT_SECONDARY
        };
        button::Appearance {
            background: Some(Background::Color(bg)),
            text_color: text_col,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        }
    }
    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(CARD)),
            text_color: TEXT_PRIMARY,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 12.0.into(),
            },
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
    engine_tx: std::sync::mpsc::Sender<crate::models::engine_commands::EngineCommand>,
    engine_rx: std::sync::mpsc::Receiver<crate::models::engine_responses::EngineResponse>,
    editor_state: EditorState,
    pending_navigation: Option<Box<Message>>,
    show_delete_dialog: bool,
    config: Config,
    config_validation_errors: std::collections::HashMap<String, String>,
    is_recording_shortcut: bool,
    command_palette: CommandPaletteState,
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
    PollEngine(std::time::Instant),
    ImportMacrosClicked,
    ExportMacrosClicked,
    FilePickedForImport(Option<std::path::PathBuf>),
    FilePickedForExport(Option<std::path::PathBuf>),
    TrayMenuEvent(tray_icon::menu::MenuEvent),
    TrayIconEvent(tray_icon::TrayIconEvent),
}

const SIDEBAR_ITEMS: &[(&str, &str)] = &[
    ("\u{F3FB}", "Macros"),
    ("\u{F24F}", "Prompts"),
    ("\u{F46C}", "Events"),
    ("\u{F3E2}", "Settings"),
];

impl TextMacroApp {
    fn tray_events() -> Subscription<Message> {
        iced::subscription::channel(
            std::any::TypeId::of::<()>(),
            100,
            |mut output| async move {
                let menu_receiver = tray_icon::menu::MenuEvent::receiver();
                let icon_receiver = tray_icon::TrayIconEvent::receiver();
                loop {
                    while let Ok(event) = menu_receiver.try_recv() {
                        let _ = output.send(Message::TrayMenuEvent(event)).await;
                    }
                    while let Ok(event) = icon_receiver.try_recv() {
                        let _ = output.send(Message::TrayIconEvent(event)).await;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
            }
        )
    }
}

impl Application for TextMacroApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ((std::sync::mpsc::Sender<crate::models::engine_commands::EngineCommand>, std::sync::mpsc::Receiver<crate::models::engine_responses::EngineResponse>), Vec<u8>, u32, u32);

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        let ((engine_tx, engine_rx), _rgba, _width, _height) = flags;
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
                engine_tx,
                engine_rx,
                editor_state: EditorState::default(),
                pending_navigation: None,
                show_delete_dialog: false,
                config,
                config_validation_errors: std::collections::HashMap::new(),
                is_recording_shortcut: false,
                command_palette: CommandPaletteState::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("TextMacro")
    }

    fn theme(&self) -> Theme {
        match self.config.theme.as_str() {
            "light" | "Editorial Light" => Theme::Light,
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
            iced::time::every(std::time::Duration::from_millis(50)).map(Message::PollEngine),
            Self::tray_events(),
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
            Message::CloseClicked => {
                if self.config.enable_background_service {
                    log::info!("Minimizing since background service is enabled.");
                    window::change_mode(window::Id::MAIN, Mode::Hidden)
                } else {
                    log::info!("Exiting TextMacro.");
                    window::close(window::Id::MAIN)
                }
            }
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
                            let _ = self.engine_tx.send(crate::models::engine_commands::EngineCommand::UpdateConfig(self.config.clone()));
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
                if self.editor_state.trigger != trigger {
                    self.editor_state.trigger = trigger;
                    self.editor_state.has_unsaved_changes = true;
                    if let Some(err) = &self.editor_state.validation_error {
                        if err.contains("Trigger") {
                            self.editor_state.validation_error = None;
                        }
                    }
                }
                Command::none()
            }
            Message::EditorDescriptionChanged(desc) => {
                if self.editor_state.description != desc {
                    self.editor_state.description = desc;
                    self.editor_state.has_unsaved_changes = true;
                }
                Command::none()
            }
            Message::EditorContentAction(action) => {
                let is_edit = action.is_edit();
                self.editor_state.content.perform(action);
                if is_edit {
                    self.editor_state.has_unsaved_changes = true;
                    if let Some(err) = &self.editor_state.validation_error {
                        if err.contains("Content") {
                            self.editor_state.validation_error = None;
                        }
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
                
                if self.editor_state.is_new {
                    let req = crate::models::engine_commands::MacroCreateRequest {
                        trigger: self.editor_state.trigger.clone(),
                        content: content_str,
                        category: match self.active_sidebar {
                            1 => MacroCategory::Prompt,
                            2 => MacroCategory::Event,
                            _ => MacroCategory::Text,
                        },
                        action_type: ActionType::InsertText,
                        description: Some(self.editor_state.description.clone()),
                        preserve_format: None,
                        tags: None,
                        shortcut: None,
                        event_trigger: None,
                    };
                    let _ = self.engine_tx.send(crate::models::engine_commands::EngineCommand::CreateMacro(req));
                } else {
                    if let Some(id) = &self.editor_state.original_id {
                        let req = crate::models::engine_commands::MacroUpdateRequest {
                            id: id.clone(),
                            trigger: Some(self.editor_state.trigger.clone()),
                            description: Some(self.editor_state.description.clone()),
                            content: Some(content_str),
                            enabled: Some(self.editor_state.enabled),
                            category: None,
                            action_type: None,
                            preserve_format: None,
                            tags: None,
                            shortcut: None,
                            event_trigger: None,
                        };
                        let _ = self.engine_tx.send(crate::models::engine_commands::EngineCommand::UpdateMacro(req));
                    }
                }
                
                self.editor_state.has_unsaved_changes = false;
                self.editor_state.validation_error = None;
                
                Command::none()
            }
            Message::DeleteMacroClick => {
                self.show_delete_dialog = true;
                Command::none()
            }
            Message::ConfirmDelete => {
                if let Some(id) = &self.editor_state.original_id {
                    let _ = self.engine_tx.send(crate::models::engine_commands::EngineCommand::DeleteMacro(id.clone()));
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
                let _ = self.engine_tx.send(crate::models::engine_commands::EngineCommand::UpdateConfig(self.config.clone()));
                Command::none()
            }
            Message::ToggleBackgroundService(b) => {
                self.config.enable_background_service = b;
                let _ = self.engine_tx.send(crate::models::engine_commands::EngineCommand::UpdateConfig(self.config.clone()));
                Command::none()
            }
            Message::TriggerPrefixChanged(val) => {
                self.config.trigger_prefix = val;
                if self.config.trigger_prefix.is_empty() {
                    self.config_validation_errors.insert("trigger_prefix".into(), "Trigger prefix is required".into());
                } else {
                    self.config_validation_errors.remove("trigger_prefix");
                    let _ = self.engine_tx.send(crate::models::engine_commands::EngineCommand::UpdateConfig(self.config.clone()));
                }
                Command::none()
            }
            Message::TriggerPrefixSubmit => {
                Command::none()
            }
            Message::ToggleEditorFontMonospace(b) => {
                self.config.editor_font_monospace = b;
                let _ = self.engine_tx.send(crate::models::engine_commands::EngineCommand::UpdateConfig(self.config.clone()));
                Command::none()
            }
            Message::TogglePreserveFormatting(b) => {
                self.config.preserve_formatting = b;
                let _ = self.engine_tx.send(crate::models::engine_commands::EngineCommand::UpdateConfig(self.config.clone()));
                Command::none()
            }
            Message::ToggleMarkdownSupport(b) => {
                self.config.markdown_support = b;
                let _ = self.engine_tx.send(crate::models::engine_commands::EngineCommand::UpdateConfig(self.config.clone()));
                Command::none()
            }
            Message::ThemeSelected(val) => {
                self.config.theme = val;
                let _ = self.engine_tx.send(crate::models::engine_commands::EngineCommand::UpdateConfig(self.config.clone()));
                Command::none()
            }
            Message::UIDensitySelected(val) => {
                self.config.ui_density = val;
                let _ = self.engine_tx.send(crate::models::engine_commands::EngineCommand::UpdateConfig(self.config.clone()));
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
            Message::TrayMenuEvent(event) => {
                if event.id == "show" {
                    return Command::batch(vec![
                        window::change_mode(window::Id::MAIN, Mode::Windowed),
                        window::gain_focus(window::Id::MAIN),
                    ]);
                } else if event.id == "quit" {
                    return window::close(window::Id::MAIN);
                }
                Command::none()
            }
            Message::TrayIconEvent(event) => {
                match event {
                    tray_icon::TrayIconEvent::Click { button: tray_icon::MouseButton::Left, .. } |
                    tray_icon::TrayIconEvent::DoubleClick { button: tray_icon::MouseButton::Left, .. } => {
                        Command::batch(vec![
                            window::change_mode(window::Id::MAIN, Mode::Windowed),
                            window::gain_focus(window::Id::MAIN),
                        ])
                    }
                    _ => Command::none()
                }
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
                    
                    self.command_palette.is_open = false;
                    
                }
                
                self.command_palette.is_open = false;
                Command::none()
            }
            Message::ToggleMacroEnabledReq(id) => {
                let mut enabled_str = None;
                if let Some(m) = self.macros.iter().find(|m| m.id == id) {
                    let _ = self.engine_tx.send(crate::models::engine_commands::EngineCommand::ToggleMacro(id.clone(), !m.enabled));
                    // We don't modify locally; we let PollEngine handle it. 
                    // But we can show toast for optimism or wait for engine. Let's do optimism toast.
                    enabled_str = Some(if !m.enabled { "Macro enabled".to_string() } else { "Macro disabled".to_string() });
                }
                
                if let Some(s) = enabled_str {
                    let m_id = id.clone();
                    
                    
                }
                Command::none()
            }
            Message::DuplicateMacroReq(id) => {
                if let Some(m) = self.macros.iter().find(|m| m.id == id).cloned() {
                    let req = crate::models::engine_commands::MacroCreateRequest {
                        trigger: format!("{}-copy", m.trigger),
                        content: m.content.clone(),
                        category: m.category.clone(),
                        action_type: m.action_type.clone(),
                        description: Some(m.description.clone()),
                        preserve_format: Some(m.preserve_format),
                        tags: Some(m.tags.clone()),
                        shortcut: m.shortcut.clone(),
                        event_trigger: m.event_trigger.clone(),
                    };
                    let _ = self.engine_tx.send(crate::models::engine_commands::EngineCommand::CreateMacro(req));
                    
                    
                    let t_id = uuid::Uuid::new_v4();
                    
                }
                Command::none()
            }
            Message::RequestDeleteMacroReq(id) => {
                let _ = self.engine_tx.send(crate::models::engine_commands::EngineCommand::DeleteMacro(id.clone()));
                if self.selected_macro_id.as_deref() == Some(id.as_str()) {
                    self.selected_macro_id = None;
                    self.editor_state = EditorState::default();
                }
                Command::none()
            }
            Message::PollEngine(_) => {
                while let Ok(response) = self.engine_rx.try_recv() {
                    match response {
                        crate::models::engine_responses::EngineResponse::MacroCreated(m) => {
                            self.macros.push(m.clone());
                            if self.editor_state.is_new && self.editor_state.trigger == m.trigger {
                                self.selected_macro_id = Some(m.id.clone());
                                self.editor_state.is_new = false;
                                self.editor_state.original_id = Some(m.id.clone());
                                self.search_query.clear();
                            }
                        }
                        crate::models::engine_responses::EngineResponse::MacroUpdated(m) => {
                            if let Some(existing) = self.macros.iter_mut().find(|ext| ext.id == m.id) {
                                *existing = m;
                            }
                            
                        }
                        crate::models::engine_responses::EngineResponse::MacroDeleted(id) => {
                            self.macros.retain(|m| m.id != id);
                            if self.selected_macro_id == Some(id) {
                                self.selected_macro_id = None;
                                self.editor_state.is_active = false;
                            }
                            
                        }
                        crate::models::engine_responses::EngineResponse::MacroToggled(id, state) => {
                            if let Some(existing) = self.macros.iter_mut().find(|ext| ext.id == id) {
                                existing.enabled = state;
                                if self.selected_macro_id == Some(id) {
                                    self.editor_state.enabled = state;
                                }
                            }
                        }
                        crate::models::engine_responses::EngineResponse::Error(err) => {
                            
                        }
                        crate::models::engine_responses::EngineResponse::ImportComplete(res) => {
                            
                            // Request refresh of macros
                            let _ = self.engine_tx.send(crate::models::engine_commands::EngineCommand::SearchMacros("".into()));
                        }
                        crate::models::engine_responses::EngineResponse::ExportComplete(res) => {
                            
                        }
                        crate::models::engine_responses::EngineResponse::SearchResults(loaded_macros) => {
                            // If it's a full reload (like after import when search_query is empty, or during active search)
                            if self.search_query.is_empty() {
                                // Full state update
                                self.macros = loaded_macros;
                            } else {
                                // Don't wipe everything if the user actually typed a query, though technically it's fine 
                                // since the UI re-filters based on self.macros and the active tab anyway.
                                // Actually wait, if the UI natively handles search through standard filtering of self.macros,
                                // we should just replace self.macros completely here.
                                self.macros = loaded_macros;
                            }
                        }
                        _ => {}
                    }
                }
                Command::none()
            }
            Message::ImportMacrosClicked => {
                Command::perform(async {
                    let file = rfd::AsyncFileDialog::new()
                        .add_filter("JSON Files", &["json"])
                        .pick_file()
                        .await;
                    file.map(|f| f.path().to_path_buf())
                }, Message::FilePickedForImport)
            }
            Message::ExportMacrosClicked => {
                Command::perform(async {
                    let file = rfd::AsyncFileDialog::new()
                        .add_filter("JSON Files", &["json"])
                        .set_file_name("textmacro_export.json")
                        .save_file()
                        .await;
                    file.map(|f| f.path().to_path_buf())
                }, Message::FilePickedForExport)
            }
            Message::FilePickedForImport(Some(path)) => {
                let path_str = path.to_string_lossy().to_string();
                let _ = self.engine_tx.send(crate::models::engine_commands::EngineCommand::ImportMacros(path_str));
                Command::none()
            }
            Message::FilePickedForImport(None) => Command::none(),
            Message::FilePickedForExport(Some(path)) => {
                let path_str = path.to_string_lossy().to_string();
                let _ = self.engine_tx.send(crate::models::engine_commands::EngineCommand::ExportMacros(path_str));
                Command::none()
            }
            Message::FilePickedForExport(None) => Command::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let is_collapsed = self.window_width > 800.0 && self.window_width < 1000.0;
        let is_hidden_right = self.window_width <= 800.0;
        let is_mobile = self.window_width < 800.0;
        let sidebar_width = if is_mobile || is_collapsed { 56.0 } else { 240.0 };

        // Title bar controls
        let btn_minimize = button(text("\u{F63B}").font(iced_aw::BOOTSTRAP_FONT).size(14).horizontal_alignment(alignment::Horizontal::Center))
            .width(Length::Fixed(46.0))
            .height(Length::Fill)
            .style(theme::Button::custom(WindowControlStyle(false)))
            .on_press(Message::MinimizeClicked);

        let max_icon = if self.window_maximized { "\u{F6D2}" } else { "\u{F584}" }; // WindowStack / Square
        let btn_maximize = button(text(max_icon).font(iced_aw::BOOTSTRAP_FONT).size(12).horizontal_alignment(alignment::Horizontal::Center))
            .width(Length::Fixed(46.0))
            .height(Length::Fill)
            .style(theme::Button::custom(WindowControlStyle(false)))
            .on_press(Message::MaximizeClicked);

        let btn_close = button(text("\u{F62A}").font(iced_aw::BOOTSTRAP_FONT).size(16).horizontal_alignment(alignment::Horizontal::Center))
            .width(Length::Fixed(46.0))
            .height(Length::Fill)
            .style(theme::Button::custom(WindowControlStyle(true)))
            .on_press(Message::CloseClicked);

        // Breadcrumb-style title bar matching the design
        let tab_label = match self.active_sidebar {
            0 => "Macros",
            1 => "Prompts",
            2 => "Events",
            3 => "Settings",
            _ => "Macros",
        };

        let title_bar = container(
            row![
                horizontal_space().width(Length::Fixed(16.0)),
                text("Workspace").size(13).style(theme::Text::Color(TEXT_SECONDARY)),
                text(" / ").size(13).style(theme::Text::Color(Color::from_rgba(0.678, 0.667, 0.667, 0.5))),
                text(tab_label).size(13).style(theme::Text::Color(ACCENT)),
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
            
            let mut item_row = row![
                text(*icon).size(16).font(iced_aw::BOOTSTRAP_FONT)
            ].align_items(alignment::Alignment::Center);

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
        modal
    }
}
