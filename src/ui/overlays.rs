use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{alignment, theme, Background, Border, Color, Element, Length, Theme};
use iced_aw::{Modal, FloatingElement};
use std::time::{Instant, Duration};
use uuid::Uuid;

use crate::models::macro_model::Macro;
use crate::ui::app::{
    Message, BACKGROUND, CARD, ACCENT, TEXT_PRIMARY, TEXT_SECONDARY, SUCCESS, ERROR
};

#[derive(Clone, Debug, PartialEq)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

#[derive(Clone, Debug)]
pub struct Toast {
    pub id: Uuid,
    pub message: String,
    pub toast_type: ToastType,
    pub created_at: Instant,
    pub duration: Duration,
}

impl Toast {
    pub fn new(message: String, toast_type: ToastType) -> Self {
        Self {
            id: Uuid::new_v4(),
            message,
            toast_type,
            created_at: Instant::now(),
            duration: Duration::from_millis(3000),
        }
    }
}

pub struct CommandPaletteState {
    pub is_open: bool,
    pub query: String,
    pub selected_index: usize,
}

impl Default for CommandPaletteState {
    fn default() -> Self {
        Self {
             is_open: false,
             query: String::new(),
             selected_index: 0,
        }
    }
}

struct ToastCardStyle(Color);
impl container::StyleSheet for ToastCardStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(CARD)),
            border: Border {
                color: self.0,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }
}

struct PaletteCardStyle(Color);
impl button::StyleSheet for PaletteCardStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(self.0)),
            text_color: TEXT_PRIMARY,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }
    }
    // minimal hover effect since mostly keyboard driven
    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        let mut app = self.active(_style);
        if self.0 != ACCENT {
            app.background = Some(Background::Color(Color::from_rgb(0.145, 0.165, 0.2))); 
        }
        app
    }
}

pub fn view_command_palette<'a>(
    content: Element<'a, Message>,
    state: &'a CommandPaletteState,
    macros: &'a [Macro],
) -> Element<'a, Message> {
    let lower_q = state.query.to_lowercase();
    let mut filtered: Vec<&Macro> = macros.iter().filter(|m| {
        if lower_q.is_empty() { true } else {
            m.trigger.to_lowercase().contains(&lower_q) || m.description.to_lowercase().contains(&lower_q)
        }
    }).collect();
    filtered.sort_by(|a, b| a.trigger.cmp(&b.trigger));

    let max_results = 8;
    
    let search_input: Element<'a, Message> = text_input("> search macro...", &state.query)
        .on_input(Message::CommandPaletteQueryChanged)
        .size(16)
        .padding(12)
        .into();

    let mut palette_col = column![search_input].spacing(8);

    if filtered.is_empty() {
        palette_col = palette_col.push(
            container(text("No results found").style(theme::Text::Color(TEXT_SECONDARY)))
                .padding(16)
                .center_x()
        );
    } else {
        for (idx, m) in filtered.into_iter().enumerate().take(max_results) {
            let is_selected = idx == state.selected_index;
            let bg = if is_selected { ACCENT } else { Color::TRANSPARENT };
            let text_col = if is_selected { Color::BLACK } else { TEXT_PRIMARY };
            let desc_col = if is_selected { Color::from_rgb(0.2, 0.2, 0.2) } else { TEXT_SECONDARY };
            
            let row_content = row![
                text(&m.trigger).size(14).style(theme::Text::Color(text_col)).width(Length::FillPortion(1)),
                text(&m.description).size(13).style(theme::Text::Color(desc_col)).width(Length::FillPortion(2))
            ]
            .padding(8)
            .align_items(alignment::Alignment::Center);
            let btn = button(row_content)
                .style(theme::Button::custom(PaletteCardStyle(bg)))
                .width(Length::Fill)
                .on_press(Message::CommandPaletteExecute); 
                
            palette_col = palette_col.push(btn);
        }
    }

    let palette_container = container(palette_col)
        .width(Length::Fixed(500.0))
        .padding(16)
        .style(theme::Container::Custom(Box::new(ToastCardStyle(BACKGROUND))));

    let overlay: Option<Element<'a, Message>> = if state.is_open {
        Some(palette_container.into())
    } else {
        None
    };

    Modal::new(content, overlay)
        .backdrop(Message::ToggleCommandPalette)
        .into()
}

pub fn view_toasts<'a>(
    content: Element<'a, Message>,
    toasts: &'a [Toast],
) -> Element<'a, Message> {
    if toasts.is_empty() {
        return content;
    }
    
    let mut col = column![].spacing(8);
    for toast in toasts.iter().rev().take(3).rev() { 
        let color = match toast.toast_type {
            ToastType::Success => SUCCESS,
            ToastType::Error => ERROR,
            ToastType::Warning => Color::from_rgb(0.984, 0.749, 0.141),
            ToastType::Info => ACCENT,
        };
        let icon = match toast.toast_type {
            ToastType::Success => "✓",
            ToastType::Error => "✗",
            ToastType::Warning => "⚠",
            ToastType::Info => "ℹ",
        };
        
        let row_content = row![
            text(icon).style(theme::Text::Color(color)).size(16),
            Space::new(8.0, 0.0),
            text(&toast.message).style(theme::Text::Color(TEXT_PRIMARY)).size(14),
            Space::new(Length::Fill, 0.0),
            button(text("×").size(16).style(theme::Text::Color(TEXT_SECONDARY)))
                .style(theme::Button::custom(crate::ui::macro_list::ClearBtnStyle))
                .on_press(Message::DismissToast(toast.id))
        ]
        .align_items(alignment::Alignment::Center)
        .padding(12);
        
        col = col.push(
            container(row_content)
                .width(Length::Fixed(300.0))
                .style(theme::Container::Custom(Box::new(ToastCardStyle(color))))
        );
    }
    
    let toast_container = container(col)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Right)
        .align_y(alignment::Vertical::Top);

    FloatingElement::new(
        content,
        toast_container
    )
    .into()
}
