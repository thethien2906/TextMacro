// ui/macro_list.rs — Editorial Macro Suite Design
use iced::widget::{button, column, container, horizontal_space, row, scrollable, text, text_input, Space};
use iced::{alignment, theme, Background, Border, Color, Element, Length, Theme};
use iced_aw::ContextMenu;

use crate::models::macro_model::{Macro, MacroCategory};
use crate::ui::app::{Message, ACCENT, CARD, SURFACE_HIGHEST, SUCCESS, TEXT_PRIMARY, TEXT_SECONDARY};

// ────────────────────────────────
//  Macro card button style
// ────────────────────────────────
struct CardStyle {
    is_disabled: bool,
    is_selected: bool,
}

impl button::StyleSheet for CardStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        let bg = if self.is_selected {
            Color::from_rgba(0.388, 0.408, 0.98, 0.10)   // indigo-500/10
        } else {
            Color::TRANSPARENT
        };
        button::Appearance {
            background: Some(Background::Color(bg)),
            text_color: if self.is_disabled { TEXT_SECONDARY } else { TEXT_PRIMARY },
            border: Border {
                color: if self.is_selected {
                    Color::from_rgba(0.388, 0.408, 0.98, 0.20)  // indigo-500/20
                } else {
                    Color::TRANSPARENT
                },
                width: if self.is_selected { 1.0 } else { 0.0 },
                radius: 12.0.into(),
            },
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        let mut app = self.active(_style);
        if !self.is_selected {
            app.background = Some(Background::Color(CARD));
        }
        app
    }
}

// ────────────────────────────────
//  Search input style
// ────────────────────────────────
struct SearchInputStyle;
impl text_input::StyleSheet for SearchInputStyle {
    type Style = Theme;
    fn active(&self, _: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(CARD),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 12.0.into(),
            },
            icon_color: TEXT_SECONDARY,
        }
    }
    fn focused(&self, _: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(SURFACE_HIGHEST),
            border: Border {
                color: Color::from_rgba(0.639, 0.651, 1.0, 0.2),
                width: 2.0,
                radius: 12.0.into(),
            },
            icon_color: TEXT_PRIMARY,
        }
    }
    fn placeholder_color(&self, _: &Self::Style) -> Color {
        Color::from_rgba(0.678, 0.667, 0.667, 0.4)  // on-surface-variant/40
    }
    fn value_color(&self, _: &Self::Style) -> Color { TEXT_PRIMARY }
    fn disabled_color(&self, _: &Self::Style) -> Color { TEXT_SECONDARY }
    fn selection_color(&self, _: &Self::Style) -> Color { ACCENT }
    fn disabled(&self, style: &Self::Style) -> text_input::Appearance { self.active(style) }
    fn hovered(&self, _: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(SURFACE_HIGHEST),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 12.0.into(),
            },
            icon_color: TEXT_PRIMARY,
        }
    }
}


pub fn view<'a>(
    macros: &'a [Macro],
    category_filter: MacroCategory,
    search_query: &'a str,
    selected_macro_id: Option<&'a str>,
) -> Element<'a, Message> {
    // Search bar ──────────────────
    let search_input = text_input("Search macros...", search_query)
        .on_input(Message::SearchQueryChanged)
        .padding(12)
        .size(13)
        .style(theme::TextInput::Custom(Box::new(SearchInputStyle)));

    let search_bar_content = if !search_query.is_empty() {
        row![
            search_input,
            button(text("×").size(16))
                .style(theme::Button::custom(ClearBtnStyle))
                .on_press(Message::ClearSearch)
        ]
        .align_items(alignment::Alignment::Center)
    } else {
        row![search_input].align_items(alignment::Alignment::Center)
    };

    let search_bar = container(search_bar_content)
        .padding(iced::Padding { top: 20.0, bottom: 12.0, left: 16.0, right: 16.0 });

    // Filter macros ──────────────────
    let mut filtered: Vec<&Macro> = macros
        .iter()
        .filter(|m| m.category == category_filter)
        .filter(|m| {
            if search_query.is_empty() {
                true
            } else {
                let lower_query = search_query.to_lowercase();
                m.trigger.to_lowercase().contains(&lower_query)
                    || m.description.to_lowercase().contains(&lower_query)
                    || m.tags.iter().any(|t| t.to_lowercase().contains(&lower_query))
            }
        })
        .collect();

    filtered.sort_by(|a, b| a.trigger.cmp(&b.trigger));

    // Macro list ──────────────────
    let mut list_col = column![].spacing(4).padding(iced::Padding { top: 4.0, bottom: 8.0, left: 12.0, right: 12.0 });

    if filtered.is_empty() {
        let (title, subtitle, _shows_btn) = if search_query.is_empty() {
            if macros.is_empty() {
                ("Welcome to TextMacro", "Create your first macro to get started.", true)
            } else {
                ("No macros yet", "Switch category or create a new macro.", true)
            }
        } else {
            ("No results", "No macros match your search.", false)
        };

        let empty_icon = container(
            text("✦").size(32).style(theme::Text::Color(Color::from_rgba(0.678, 0.667, 0.667, 0.2)))
        )
        .width(Length::Fixed(64.0))
        .height(Length::Fixed(64.0))
        .center_x()
        .center_y()
        .style(|_theme: &Theme| container::Appearance {
            background: Some(Background::Color(CARD)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 32.0.into(),
            },
            ..Default::default()
        });

        let empty_state = column![
            empty_icon,
            Space::new(0.0, 8.0),
            text(title).size(16).style(theme::Text::Color(TEXT_SECONDARY)),
            text(subtitle).size(13).style(theme::Text::Color(Color::from_rgba(0.678, 0.667, 0.667, 0.4)))
        ]
        .align_items(alignment::Alignment::Center)
        .spacing(6);

        let empty_container = container(empty_state)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y();

        list_col = list_col.push(empty_container);
    } else {
        for m in filtered {
            let is_selected = selected_macro_id == Some(m.id.as_str());
            let is_disabled = !m.enabled;

            // Status dot
            let dot_color = if m.enabled { SUCCESS } else { TEXT_SECONDARY };
            let dot = container(Space::new(6.0, 6.0))
                .style(theme::Container::Custom(Box::new(RadiusBoxStyle(dot_color, 3.0))));

            // Active indicator (right border)
            let left_border = if is_selected {
                container(Space::new(3.0, Length::Fill))
                    .style(theme::Container::Custom(Box::new(RadiusBoxStyle(
                        Color::from_rgb(0.388, 0.408, 0.98),  // indigo-500
                        2.0,
                    ))))
            } else {
                container(Space::new(3.0, Length::Fill))
            };

            let title_text = text(&m.trigger)
                .size(13)
                .style(if is_selected {
                    theme::Text::Color(TEXT_PRIMARY)
                } else if is_disabled {
                    theme::Text::Color(TEXT_SECONDARY)
                } else {
                    theme::Text::Color(Color::from_rgba(0.678, 0.667, 0.667, 1.0))  // on-surface-variant
                });

            let desc_text = text(&m.description)
                .size(12)
                .style(theme::Text::Color(Color::from_rgba(0.678, 0.667, 0.667, 0.6)));

            let content = row![
                left_border,
                column![
                    row![title_text, horizontal_space().width(Length::Fill), dot],
                    desc_text
                ].padding(iced::Padding { top: 10.0, bottom: 10.0, left: 12.0, right: 12.0 })
            ];

            let card_btn = button(content)
                .width(Length::Fill)
                .style(theme::Button::custom(CardStyle { is_disabled, is_selected }))
                .on_press(Message::SelectMacro(m.id.clone()));

            // Context menu
            let m_id_edit = m.id.clone();
            let m_id_dup = m.id.clone();
            let m_id_tog = m.id.clone();
            let m_id_del = m.id.clone();
            let m_enabled = m.enabled;

            let context_card = ContextMenu::new(card_btn, move || {
                let edit_btn = button(text("Edit").size(13))
                    .width(Length::Fill).padding(8).style(theme::Button::custom(MenuBtnStyle)).on_press(Message::SelectMacro(m_id_edit.clone()));
                let dup_btn = button(text("Duplicate").size(13))
                    .width(Length::Fill).padding(8).style(theme::Button::custom(MenuBtnStyle)).on_press(Message::DuplicateMacroReq(m_id_dup.clone()));
                let tog_btn = button(text(if m_enabled { "Disable" } else { "Enable" }).size(13))
                    .width(Length::Fill).padding(8).style(theme::Button::custom(MenuBtnStyle)).on_press(Message::ToggleMacroEnabledReq(m_id_tog.clone()));
                let del_btn = button(text("Delete").size(13).style(theme::Text::Color(crate::ui::app::ERROR)))
                    .width(Length::Fill).padding(8).style(theme::Button::custom(MenuBtnStyle)).on_press(Message::RequestDeleteMacroReq(m_id_del.clone()));

                let menu_col = column![edit_btn, dup_btn, tog_btn, del_btn].width(Length::Fixed(150.0));
                container(menu_col).padding(6).style(theme::Container::Custom(Box::new(ContextMenuContainerStyle))).into()
            });

            list_col = list_col.push(context_card);
        }
    }

    let scrollable_list = scrollable(list_col).height(Length::Fill);

    // New macro button (gradient style) ──────────────────
    let new_btn = container(
        button(
            row![
                text("+").size(16).style(theme::Text::Color(Color::BLACK)),
                Space::new(6.0, 0.0),
                text("New Macro").size(13).style(theme::Text::Color(Color::BLACK))
            ]
            .align_items(alignment::Alignment::Center)
        )
        .width(Length::Fill)
        .padding(12)
        .style(theme::Button::custom(GradientPrimaryButtonStyle))
        .on_press(Message::NewMacroClick)
    )
    .padding(iced::Padding { top: 8.0, bottom: 16.0, left: 16.0, right: 16.0 });

    // Panel container — surface-container-low/50 tint
    container(column![search_bar, scrollable_list, new_btn])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme: &Theme| container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.075, 0.075, 0.075, 0.5))),
            ..Default::default()
        })
        .into()
}

// ════════════════════════════════
//  Re‑usable style structs
// ════════════════════════════════

struct RadiusBoxStyle(Color, f32);
impl container::StyleSheet for RadiusBoxStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(self.0)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: self.1.into(),
            },
            ..Default::default()
        }
    }
}

pub struct ClearBtnStyle;
impl button::StyleSheet for ClearBtnStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: TEXT_SECONDARY,
            ..Default::default()
        }
    }
    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: TEXT_PRIMARY,
            ..Default::default()
        }
    }
}

/// Gradient primary button (matches `from-primary to-primary-dim`)
struct GradientPrimaryButtonStyle;
impl button::StyleSheet for GradientPrimaryButtonStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(ACCENT)),
            text_color: Color::BLACK,
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
        app.background = Some(Background::Color(Color::from_rgb(
            (ACCENT.r * 1.1).min(1.0),
            (ACCENT.g * 1.1).min(1.0),
            (ACCENT.b * 1.1).min(1.0),
        )));
        app
    }
}

pub struct MenuBtnStyle;
impl button::StyleSheet for MenuBtnStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            text_color: TEXT_PRIMARY,
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }
    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        let mut app = self.active(_style);
        app.background = Some(Background::Color(CARD));
        app
    }
}

pub struct ContextMenuContainerStyle;
impl container::StyleSheet for ContextMenuContainerStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.125, 0.125, 0.122, 0.9))),
            border: Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.05),
                width: 1.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        }
    }
}
