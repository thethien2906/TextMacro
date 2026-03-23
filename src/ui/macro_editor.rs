use iced::widget::{button, column, container, horizontal_space, row, text, text_input, Space, toggler, text_editor};
use iced::{alignment, theme, Background, Border, Color, Element, Length, Theme};

use crate::ui::app::{Message, EditorState, ACCENT, BACKGROUND, BORDER, CARD, ERROR, PANEL, TEXT_PRIMARY, TEXT_SECONDARY};

struct EditorInputStyle {
    is_invalid: bool,
}

impl text_input::StyleSheet for EditorInputStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(PANEL),
            border: Border {
                color: if self.is_invalid { ERROR } else { BORDER },
                width: 1.0,
                radius: 6.0.into(),
            },
            icon_color: TEXT_SECONDARY,
        }
    }
    fn focused(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(PANEL),
            border: Border {
                color: if self.is_invalid { ERROR } else { ACCENT },
                width: 1.0,
                radius: 6.0.into(),
            },
            icon_color: TEXT_PRIMARY,
        }
    }
    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        TEXT_SECONDARY
    }
    fn value_color(&self, _style: &Self::Style) -> Color {
        TEXT_PRIMARY
    }
    fn disabled_color(&self, _style: &Self::Style) -> Color {
        TEXT_SECONDARY
    }
    fn disabled(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(BACKGROUND),
            border: Border {
                color: BORDER,
                width: 1.0,
                radius: 6.0.into(),
            },
            icon_color: TEXT_SECONDARY,
        }
    }
    fn selection_color(&self, _style: &Self::Style) -> Color {
        ACCENT
    }
    fn hovered(&self, _style: &Self::Style) -> text_input::Appearance {
         text_input::Appearance {
            background: Background::Color(PANEL),
            border: Border {
                color: if self.is_invalid { ERROR } else { TEXT_SECONDARY },
                width: 1.0,
                radius: 6.0.into(),
            },
            icon_color: TEXT_PRIMARY,
        }
    }
}

struct ContentEditorStyle {
    is_invalid: bool,
}

impl text_editor::StyleSheet for ContentEditorStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> text_editor::Appearance {
        text_editor::Appearance {
             background: Background::Color(PANEL),
             border: Border {
                 color: if self.is_invalid { ERROR } else { BORDER },
                 width: 1.0,
                 radius: 6.0.into(),
             },
         }
    }
    fn focused(&self, _style: &Self::Style) -> text_editor::Appearance {
        text_editor::Appearance {
            background: Background::Color(PANEL),
            border: Border {
                color: if self.is_invalid { ERROR } else { ACCENT },
                width: 1.0,
                radius: 6.0.into(),
            },
        }
    }
    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        TEXT_SECONDARY
    }
    fn value_color(&self, _style: &Self::Style) -> Color {
        TEXT_PRIMARY
    }
    fn disabled_color(&self, _style: &Self::Style) -> Color {
        TEXT_SECONDARY
    }
    fn disabled(&self, _style: &Self::Style) -> text_editor::Appearance {
        text_editor::Appearance {
             background: Background::Color(BACKGROUND),
             border: Border {
                 color: BORDER,
                 width: 1.0,
                 radius: 6.0.into(),
             },
         }
    }
    fn selection_color(&self, _style: &Self::Style) -> Color {
        ACCENT
    }
    fn hovered(&self, _style: &Self::Style) -> text_editor::Appearance {
        text_editor::Appearance {
             background: Background::Color(PANEL),
             border: Border {
                 color: if self.is_invalid { ERROR } else { TEXT_SECONDARY },
                 width: 1.0,
                 radius: 6.0.into(),
             },
         }
    }
}

struct PrimaryButtonStyle {
    is_disabled: bool,
}
impl button::StyleSheet for PrimaryButtonStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(if self.is_disabled { CARD } else { ACCENT })),
            text_color: if self.is_disabled { TEXT_SECONDARY } else { Color::BLACK },
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }
    }
    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        if self.is_disabled {
            return self.active(_style);
        }
        let mut app = self.active(_style);
        app.background = Some(Background::Color(Color::from_rgb(
            (ACCENT.r * 1.1).min(1.0),
            (ACCENT.g * 1.1).min(1.0),
            (ACCENT.b * 1.1).min(1.0),
        )));
        app
    }
}

pub struct DangerButtonStyle;
impl button::StyleSheet for DangerButtonStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.3, 0.1, 0.1))), // Tinted dark red
            text_color: ERROR,
            border: Border {
                color: ERROR,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }
    }
    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        let mut app = self.active(_style);
        app.background = Some(Background::Color(Color::from_rgb(0.8, 0.2, 0.2)));
        app.text_color = Color::WHITE;
        app
    }
}

pub fn view<'a>(state: &'a EditorState, show_delete_dialog: bool, editor_font_monospace: bool) -> Element<'a, Message> {
    if !state.is_active {
        return container(
            text("Select a macro to edit or create a new one")
                .size(16)
                .style(theme::Text::Color(TEXT_SECONDARY))
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into();
    }

    let is_trigger_invalid = state.validation_error.as_ref().map_or(false, |e| e.contains("Trigger"));
    let is_content_invalid = state.validation_error.as_ref().map_or(false, |e| e.contains("Content"));

    // Header with unsaved changes indicator
    let header_text = if state.is_new { "Create Macro" } else { "Edit Macro" };
    let mut header_row = row![text(header_text).size(20).style(theme::Text::Color(TEXT_PRIMARY))];
    
    if state.has_unsaved_changes {
        header_row = header_row.push(Space::new(10.0, Length::Shrink));
        header_row = header_row.push(
            row![
                text("●").size(12).style(theme::Text::Color(ACCENT)),
                Space::new(4.0, Length::Shrink),
                text("Unsaved changes").size(12).style(theme::Text::Color(TEXT_SECONDARY))
            ].align_items(alignment::Alignment::Center)
        );
    }

    // Trigger field
    let trigger_label = text("Trigger").size(14).style(theme::Text::Color(TEXT_SECONDARY));
    let trigger_input = text_input("Enter trigger (e.g., /sig)", &state.trigger)
        .on_input(Message::EditorTriggerChanged)
        .padding(10)
        .style(theme::TextInput::Custom(Box::new(EditorInputStyle { is_invalid: is_trigger_invalid })));
    
    let mut trigger_col = column![trigger_label, trigger_input].spacing(8);
    if is_trigger_invalid {
        if let Some(err) = &state.validation_error {
            trigger_col = trigger_col.push(text(err).size(12).style(theme::Text::Color(ERROR)));
        }
    }

    // Description field
    let desc_label = text("Description").size(14).style(theme::Text::Color(TEXT_SECONDARY));
    let desc_input = text_input("Short description (optional)", &state.description)
        .on_input(Message::EditorDescriptionChanged)
        .padding(10)
        .style(theme::TextInput::Custom(Box::new(EditorInputStyle { is_invalid: false })));
    let desc_col = column![desc_label, desc_input].spacing(8);

    // Content field
    let content_label = text("Content").size(14).style(theme::Text::Color(TEXT_SECONDARY));
    let content_editor = text_editor(&state.content)
        .on_action(Message::EditorContentAction)
        .height(Length::FillPortion(1))
        .font(if editor_font_monospace { iced::Font::MONOSPACE } else { iced::Font::DEFAULT })
        .style(theme::TextEditor::Custom(Box::new(ContentEditorStyle { is_invalid: is_content_invalid })));
        
    let mut content_col = column![content_label, content_editor].spacing(8).height(Length::FillPortion(1));
    if is_content_invalid {
        if let Some(err) = &state.validation_error {
            content_col = content_col.push(text(err).size(12).style(theme::Text::Color(ERROR)));
        }
    }

    // Enabled toggle
    let toggle = toggler(
        Some("Enabled".to_string()),
        state.enabled,
        Message::EditorEnabledToggled,
    )
    .width(Length::Shrink);

    // Buttons
    let save_btn = button(
        text(if state.is_new { "Create" } else { "Save" })
            .horizontal_alignment(alignment::Horizontal::Center)
    )
    .width(Length::Fixed(100.0))
    .padding(8)
    .style(theme::Button::custom(PrimaryButtonStyle { is_disabled: !state.has_unsaved_changes }))
    .on_press_maybe(if state.has_unsaved_changes { Some(Message::SaveMacro) } else { None });

    let mut actions_row = row![toggle, horizontal_space().width(Length::Fill), save_btn].align_items(alignment::Alignment::Center);

    if !state.is_new {
        actions_row = actions_row.push(Space::new(10.0, Length::Shrink));
        if show_delete_dialog {
            let cancel_btn = button(
                text("Cancel").horizontal_alignment(alignment::Horizontal::Center)
            )
            .width(Length::Fixed(80.0))
            .padding(8)
            .style(theme::Button::custom(crate::ui::macro_list::ClearBtnStyle))
            .on_press(Message::CancelDelete);

            let confirm_btn = button(
                text("Confirm").horizontal_alignment(alignment::Horizontal::Center)
            )
            .width(Length::Fixed(80.0))
            .padding(8)
            .style(theme::Button::custom(DangerButtonStyle))
            .on_press(Message::ConfirmDelete);
            
            actions_row = actions_row.push(row![text("Are you sure? ").size(14).style(theme::Text::Color(TEXT_SECONDARY)), cancel_btn, Space::new(4.0, 0.0), confirm_btn].align_items(alignment::Alignment::Center));
        } else {
            let delete_btn = button(
                text("Delete").horizontal_alignment(alignment::Horizontal::Center)
            )
            .width(Length::Fixed(100.0))
            .padding(8)
            .style(theme::Button::custom(DangerButtonStyle))
            .on_press(Message::DeleteMacroClick);
            
            actions_row = actions_row.push(delete_btn);
        }
    }

    let editor_form = column![
        header_row,
        Space::new(0.0, 10.0),
        trigger_col,
        desc_col,
        content_col,
        actions_row
    ]
    .spacing(16)
    .padding(24)
    .height(Length::Fill);

    container(editor_form)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
