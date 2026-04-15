// ui/overlays.rs — Editorial Macro Suite Design
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{alignment, theme, Background, Border, Color, Element, Length, Theme};
use iced_aw::{Modal, FloatingElement};

use crate::models::macro_model::Macro;
use crate::ui::app::{
    Message, CARD, ACCENT, TEXT_PRIMARY, TEXT_SECONDARY,
};

// ════════════════════════════════
//  Command Palette State
// ════════════════════════════════

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

// ────────────────────────────────
//  Command palette item style
// ────────────────────────────────
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
                radius: 12.0.into(),
            },
            ..Default::default()
        }
    }
    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        let mut app = self.active(_style);
        if self.0 != ACCENT {
            app.background = Some(Background::Color(CARD));
        }
        app
    }
}

// ────────────────────────────────
//  Palette modal container style
// ────────────────────────────────
struct PaletteContainerStyle;
impl container::StyleSheet for PaletteContainerStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.055, 0.055, 0.055, 0.95))),
            border: Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.05),
                width: 1.0,
                radius: 16.0.into(),
            },
            ..Default::default()
        }
    }
}

// ════════════════════════════════
//  Command Palette View
// ════════════════════════════════
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

    let search_input: Element<'a, Message> = text_input("\u{F52A} search macro...", &state.query)
        .on_input(Message::CommandPaletteQueryChanged)
        .size(15)
        .padding(14)
        .into();

    let mut palette_col = column![search_input].spacing(6);

    if filtered.is_empty() {
        palette_col = palette_col.push(
            container(
                text("No results found")
                    .size(13)
                    .style(theme::Text::Color(Color::from_rgba(0.678, 0.667, 0.667, 0.4)))
            )
            .padding(20)
            .width(Length::Fill)
            .center_x()
        );
    } else {
        for (idx, m) in filtered.into_iter().enumerate().take(max_results) {
            let is_selected = idx == state.selected_index;
            let bg = if is_selected { ACCENT } else { Color::TRANSPARENT };
            let text_col = if is_selected { Color::BLACK } else { TEXT_PRIMARY };
            let desc_col = if is_selected { Color::from_rgb(0.15, 0.15, 0.15) } else { TEXT_SECONDARY };

            let row_content = row![
                text(&m.trigger)
                    .size(14)
                    .style(theme::Text::Color(text_col))
                    .width(Length::FillPortion(1)),
                text(&m.description)
                    .size(12)
                    .style(theme::Text::Color(desc_col))
                    .width(Length::FillPortion(2))
            ]
            .padding(iced::Padding { top: 10.0, bottom: 10.0, left: 12.0, right: 12.0 })
            .align_items(alignment::Alignment::Center);

            let btn = button(row_content)
                .style(theme::Button::custom(PaletteCardStyle(bg)))
                .width(Length::Fill)
                .on_press(Message::CommandPaletteExecute);

            palette_col = palette_col.push(btn);
        }
    }

    let palette_container = container(palette_col)
        .width(Length::Fixed(520.0))
        .padding(16)
        .style(theme::Container::Custom(Box::new(PaletteContainerStyle)));

    let overlay: Option<Element<'a, Message>> = if state.is_open {
        Some(palette_container.into())
    } else {
        None
    };

    Modal::new(content, overlay)
        .backdrop(Message::ToggleCommandPalette)
        .into()
}

// ════════════════════════════════
//  Toasts View
// ════════════════════════════════
