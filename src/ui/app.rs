use iced::widget::{
    button, column, container, horizontal_space, mouse_area, row, text, Space,
};
use iced::{
    alignment, executor, theme, window, Application, Background, Border, Color, Command, Element,
    Event, Font, Length, Settings, Size, Subscription, Theme,
};
use iced::window::Event as WindowEvent;
use iced::keyboard;
use iced::keyboard::key::Named;
use iced::Event as IcedEvent;

// Design tokens
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
        (
            Self {
                active_sidebar: 0,
                focused_sidebar: None,
                window_maximized: false,
                window_width: 1200.0,
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
                self.active_sidebar = idx;
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
                        let current = self.focused_sidebar.unwrap_or(self.active_sidebar);
                        if current > 0 {
                            self.focused_sidebar = Some(current - 1);
                        }
                    }
                    keyboard::Key::Named(Named::ArrowDown) => {
                        let current = self.focused_sidebar.unwrap_or(self.active_sidebar);
                        if current < 3 {
                            self.focused_sidebar = Some(current + 1);
                        }
                    }
                    keyboard::Key::Named(Named::Enter) => {
                        if let Some(focused) = self.focused_sidebar {
                            self.active_sidebar = focused;
                        }
                    }
                    _ => {}
                }
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

        let center_panel = container(
            text("MacroList will be here")
                .size(16)
                .style(theme::Text::Color(TEXT_SECONDARY)),
        )
        .width(Length::FillPortion(35))
        .height(Length::Fill)
        .center_x()
        .center_y()
        .style(theme::Container::Custom(Box::new(CenterPanelStyle)));

        let right_panel: Element<'_, Message> = if is_hidden_right {
            Space::new(0.0, 0.0).into()
        } else {
            container(text("MacroEditor will be here").size(16).style(theme::Text::Color(TEXT_SECONDARY)))
                .width(Length::FillPortion(65))
                .height(Length::Fill)
                .center_x()
                .center_y()
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
