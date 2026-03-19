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
use crate::storage::macro_repository::StorageManager;
use crate::ui::macro_list;
use crate::ui::macro_editor;
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
    KeyPressed(keyboard::Key),
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
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("TextMacro")
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::event::listen_with(|event, _status| match event {
                IcedEvent::Window(_, WindowEvent::Resized { width: w, height: h }) => {
                    Some(Message::WindowResized(w, h))
                }
                IcedEvent::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                    Some(Message::KeyPressed(key))
                }
                _ => None,
            }),
        ])
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
            Message::KeyPressed(key) => {
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

        let macro_list_view = if self.active_sidebar < 3 {
            macro_list::view(
                &self.macros,
                active_category,
                &self.search_query,
                self.selected_macro_id.as_deref(),
            )
        } else {
            // Settings Panel
            container(text("Settings Panel").style(theme::Text::Color(TEXT_SECONDARY)))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .into()
        };

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
            container(macro_editor::view(&self.editor_state, self.show_delete_dialog))
                .width(Length::FillPortion(65))
                .height(Length::Fill)
                .style(theme::Container::Custom(Box::new(CenterPanelStyle)))
                .into()
        };

        let content = row![sidebar, center_panel, right_panel].height(Length::Fill);
        
        container(column![draggble_title_bar, content])
            .style(theme::Container::Custom(Box::new(MainContainerStyle)))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
