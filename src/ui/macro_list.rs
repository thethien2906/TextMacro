// ui/macro_list.rs
use iced::widget::{button, column, container, horizontal_space, row, scrollable, text, text_input, Space};
use iced::{alignment, theme, Background, Border, Color, Element, Length, Theme};
use iced_aw::ContextMenu;

use crate::models::macro_model::{Macro, MacroCategory};
use crate::ui::app::{Message, ACCENT, CARD, SUCCESS, TEXT_PRIMARY, TEXT_SECONDARY};

struct CardStyle {
    is_disabled: bool,
}

impl button::StyleSheet for CardStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(CARD)),
            text_color: if self.is_disabled { TEXT_SECONDARY } else { TEXT_PRIMARY },
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
        app.background = Some(Background::Color(Color::from_rgb(0.145, 0.165, 0.2))); // #252A33
        app
    }
}

pub fn view<'a>(
    macros: &'a [Macro],
    category_filter: MacroCategory,
    search_query: &'a str,
    selected_macro_id: Option<&'a str>,
) -> Element<'a, Message> {
    let search_input = text_input(">>  Search macros...", search_query)
        .on_input(Message::SearchQueryChanged)
        .padding(10)
        .size(14);
    
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
        .padding(iced::Padding { top: 16.0, bottom: 8.0, left: 16.0, right: 16.0 });

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

    let mut list_col = column![].spacing(8).padding(iced::Padding { top: 8.0, bottom: 8.0, left: 16.0, right: 16.0 });

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
        
        let empty_state = column![
            text(title).size(18).style(theme::Text::Color(TEXT_PRIMARY)),
            text(subtitle).size(14).style(theme::Text::Color(TEXT_SECONDARY))
        ]
        .align_items(alignment::Alignment::Center)
        .spacing(8);
        
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

            let dot_color = if m.enabled { SUCCESS } else { TEXT_SECONDARY };
            let dot = container(Space::new(8.0, 8.0))
                .style(theme::Container::Custom(Box::new(RadiusBoxStyle(dot_color, 4.0))));

            let left_border = if is_selected {
                container(Space::new(4.0, Length::Fill))
                    .style(theme::Container::Custom(Box::new(RadiusBoxStyle(ACCENT, 2.0))))
            } else {
                container(Space::new(4.0, Length::Fill))
            };

            let title_text = text(&m.trigger)
                .size(14)
                .style(if is_disabled { theme::Text::Color(TEXT_SECONDARY) } else { theme::Text::Color(TEXT_PRIMARY) });
            
            let desc_text = text(&m.description)
                .size(13)
                .style(theme::Text::Color(TEXT_SECONDARY));

            let content = row![
                left_border,
                column![
                    row![title_text, horizontal_space().width(Length::Fill), dot],
                    desc_text
                ].padding(iced::Padding { top: 12.0, bottom: 12.0, left: 12.0, right: 16.0 })
            ];

            let card_btn = button(content)
                .width(Length::Fill)
                .style(theme::Button::custom(CardStyle { is_disabled }))
                .on_press(Message::SelectMacro(m.id.clone()));

            let m_id_edit = m.id.clone();
            let m_id_dup = m.id.clone();
            let m_id_tog = m.id.clone();
            let m_id_del = m.id.clone();
            let m_enabled = m.enabled;

            let context_card = ContextMenu::new(card_btn, move || {
                let edit_btn = button(text("Edit").size(14))
                    .width(Length::Fill).padding(6).style(theme::Button::custom(MenuBtnStyle)).on_press(Message::SelectMacro(m_id_edit.clone()));
                let dup_btn = button(text("Duplicate").size(14))
                    .width(Length::Fill).padding(6).style(theme::Button::custom(MenuBtnStyle)).on_press(Message::DuplicateMacroReq(m_id_dup.clone()));
                let tog_btn = button(text(if m_enabled { "Disable" } else { "Enable" }).size(14))
                    .width(Length::Fill).padding(6).style(theme::Button::custom(MenuBtnStyle)).on_press(Message::ToggleMacroEnabledReq(m_id_tog.clone()));
                let del_btn = button(text("Delete").size(14).style(theme::Text::Color(crate::ui::app::ERROR)))
                    .width(Length::Fill).padding(6).style(theme::Button::custom(MenuBtnStyle)).on_press(Message::RequestDeleteMacroReq(m_id_del.clone()));
                    
                let menu_col = column![edit_btn, dup_btn, tog_btn, del_btn].width(Length::Fixed(150.0));
                container(menu_col).padding(4).style(theme::Container::Custom(Box::new(ContextMenuContainerStyle))).into()
            });
            
            list_col = list_col.push(context_card);
        }
    }

    let scrollable_list = scrollable(list_col).height(Length::Fill);

    let new_btn = container(
        button(text("+ New Macro").horizontal_alignment(alignment::Horizontal::Center).style(theme::Text::Color(Color::BLACK)))
            .width(Length::Fill)
            .padding(12)
            .style(theme::Button::custom(PrimaryButtonStyle))
            .on_press(Message::NewMacroClick)
    )
    .padding(16);

    container(column![search_bar, scrollable_list, new_btn])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

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

struct PrimaryButtonStyle;
impl button::StyleSheet for PrimaryButtonStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(ACCENT)),
            text_color: Color::BLACK,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }
    }
    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        let mut app = self.active(_style);
        // Slightly brighter accent
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
            border: Border::default(),
            ..Default::default()
        }
    }
    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        let mut app = self.active(_style);
        app.background = Some(Background::Color(Color::from_rgb(0.25, 0.28, 0.35)));
        app
    }
}

pub struct ContextMenuContainerStyle;
impl container::StyleSheet for ContextMenuContainerStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(CARD)),
            border: Border {
                color: TEXT_SECONDARY,
                width: 1.0,
                radius: 6.0.into()
            },
            ..Default::default()
        }
    }
}
