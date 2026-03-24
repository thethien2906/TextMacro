// ui/macro_editor.rs — Editorial Macro Suite Design
use iced::widget::{button, column, container, horizontal_space, row, text, text_input, Space, toggler, text_editor};
use iced::{alignment, theme, Background, Border, Color, Element, Length, Theme};

use crate::ui::app::{
    Message, EditorState,
    ACCENT, BACKGROUND, CARD, SURFACE_HIGHEST,
    ERROR, ERROR_DIM, TEXT_PRIMARY, TEXT_SECONDARY,
};

// ────────────────────────────────
//  Input field style (filled style per design bible)
// ────────────────────────────────
struct EditorInputStyle {
    is_invalid: bool,
}

impl text_input::StyleSheet for EditorInputStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(CARD),
            border: Border {
                color: if self.is_invalid { ERROR } else { Color::TRANSPARENT },
                width: if self.is_invalid { 1.0 } else { 0.0 },
                radius: 12.0.into(),
            },
            icon_color: TEXT_SECONDARY,
        }
    }
    fn focused(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(Color::from_rgb(0.102, 0.102, 0.102)), // surface-container
            border: Border {
                color: if self.is_invalid { ERROR } else { Color::TRANSPARENT },
                width: if self.is_invalid { 1.0 } else { 0.0 },
                radius: 12.0.into(),
            },
            icon_color: TEXT_PRIMARY,
        }
    }
    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgba(0.678, 0.667, 0.667, 0.4) // on-surface-variant/40
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
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 12.0.into(),
            },
            icon_color: TEXT_SECONDARY,
        }
    }
    fn selection_color(&self, _style: &Self::Style) -> Color {
        ACCENT
    }
    fn hovered(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(SURFACE_HIGHEST),
            border: Border {
                color: if self.is_invalid { ERROR } else { Color::TRANSPARENT },
                width: if self.is_invalid { 1.0 } else { 0.0 },
                radius: 12.0.into(),
            },
            icon_color: TEXT_PRIMARY,
        }
    }
}

// ────────────────────────────────
//  Content editor (textarea) style
// ────────────────────────────────
struct ContentEditorStyle {
    is_invalid: bool,
}

impl text_editor::StyleSheet for ContentEditorStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> text_editor::Appearance {
        text_editor::Appearance {
             background: Background::Color(CARD),
             border: Border {
                 color: if self.is_invalid { ERROR } else { Color::TRANSPARENT },
                 width: if self.is_invalid { 1.0 } else { 0.0 },
                 radius: 12.0.into(),
             },
        }
    }
    fn focused(&self, _style: &Self::Style) -> text_editor::Appearance {
        text_editor::Appearance {
            background: Background::Color(Color::from_rgb(0.102, 0.102, 0.102)),
            border: Border {
                color: if self.is_invalid { ERROR } else { Color::TRANSPARENT },
                width: if self.is_invalid { 1.0 } else { 0.0 },
                radius: 12.0.into(),
            },
        }
    }
    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgba(0.678, 0.667, 0.667, 0.4)
    }
    fn value_color(&self, _style: &Self::Style) -> Color {
        ACCENT  // primary color for code content
    }
    fn disabled_color(&self, _style: &Self::Style) -> Color {
        TEXT_SECONDARY
    }
    fn disabled(&self, _style: &Self::Style) -> text_editor::Appearance {
        text_editor::Appearance {
            background: Background::Color(BACKGROUND),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 12.0.into(),
            },
        }
    }
    fn selection_color(&self, _style: &Self::Style) -> Color {
        ACCENT
    }
    fn hovered(&self, _style: &Self::Style) -> text_editor::Appearance {
        text_editor::Appearance {
            background: Background::Color(SURFACE_HIGHEST),
            border: Border {
                color: if self.is_invalid { ERROR } else { Color::TRANSPARENT },
                width: if self.is_invalid { 1.0 } else { 0.0 },
                radius: 12.0.into(),
            },
        }
    }
}

// ────────────────────────────────
//  Save button (gradient primary)
// ────────────────────────────────
struct PrimaryButtonStyle {
    is_disabled: bool,
}
impl button::StyleSheet for PrimaryButtonStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(if self.is_disabled { SURFACE_HIGHEST } else { ACCENT })),
            text_color: if self.is_disabled { TEXT_SECONDARY } else { Color::BLACK },
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 12.0.into(),
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
            (ACCENT.r * 1.08).min(1.0),
            (ACCENT.g * 1.08).min(1.0),
            (ACCENT.b * 1.08).min(1.0),
        )));
        app
    }
}

// ────────────────────────────────
//  Delete button (danger)
// ────────────────────────────────
pub struct DangerButtonStyle;
impl button::StyleSheet for DangerButtonStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: ERROR_DIM,
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
            background: Some(Background::Color(Color::from_rgba(1.0, 0.431, 0.518, 0.1))),
            text_color: ERROR,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        }
    }
}

// ────────────────────────────────
//  Toggle pill style (for enabled toggle)
// ────────────────────────────────


// ═══════════════════════════════════════
//  Main view function
// ═══════════════════════════════════════
pub fn view<'a>(state: &'a EditorState, show_delete_dialog: bool, editor_font_monospace: bool) -> Element<'a, Message> {
    if !state.is_active {
        // Empty state matching design: centered icon + text
        let empty_icon = container(
            text("\u{F4C9}").size(36).font(iced_aw::BOOTSTRAP_FONT).style(theme::Text::Color(Color::from_rgba(0.678, 0.667, 0.667, 0.2)))
        )
        .width(Length::Fixed(80.0))
        .height(Length::Fixed(80.0))
        .center_x()
        .center_y()
        .style(|_theme: &Theme| container::Appearance {
            background: Some(Background::Color(CARD)),
            border: Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.05),
                width: 1.0,
                radius: 40.0.into(),
            },
            ..Default::default()
        });

        let empty_state = column![
            empty_icon,
            Space::new(0.0, 16.0),
            text("Select a macro to edit or create a new one")
                .size(20)
                .style(theme::Text::Color(TEXT_SECONDARY)),
            Space::new(0.0, 4.0),
            text("Select an entry from the list on the left to view and modify its code.")
                .size(13)
                .style(theme::Text::Color(Color::from_rgba(0.678, 0.667, 0.667, 0.4))),
        ]
        .align_items(alignment::Alignment::Center);

        return container(empty_state)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into();
    }

    let is_trigger_invalid = state.validation_error.as_ref().map_or(false, |e| e.contains("Trigger"));
    let is_content_invalid = state.validation_error.as_ref().map_or(false, |e| e.contains("Content"));

    // Header  ──────────────────
    let header_text = if state.is_new { "Create Macro" } else { "Edit Macro" };
    let mut header_row = row![
        text(header_text).size(28).style(theme::Text::Color(TEXT_PRIMARY))
    ];

    if state.has_unsaved_changes {
        header_row = header_row.push(Space::new(12.0, Length::Shrink));
        header_row = header_row.push(
            container(
                row![
                    container(Space::new(6.0, 6.0))
                        .style(|_theme: &Theme| container::Appearance {
                            background: Some(Background::Color(ACCENT)),
                            border: Border {
                                color: Color::TRANSPARENT,
                                width: 0.0,
                                radius: 3.0.into(),
                            },
                            ..Default::default()
                        }),
                    Space::new(6.0, Length::Shrink),
                    text("Unsaved").size(10).style(theme::Text::Color(TEXT_SECONDARY))
                ].align_items(alignment::Alignment::Center)
            )
            .padding(iced::Padding { top: 4.0, bottom: 4.0, left: 10.0, right: 12.0 })
            .style(|_theme: &Theme| container::Appearance {
                background: Some(Background::Color(CARD)),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            })
        );
    }

    // Trigger field  ──────────────────
    let trigger_label = text("TRIGGER")
        .size(10)
        .style(theme::Text::Color(TEXT_SECONDARY));
    let trigger_input = text_input("Enter trigger (e.g., /sig)", &state.trigger)
        .on_input(Message::EditorTriggerChanged)
        .padding(iced::Padding { top: 14.0, bottom: 14.0, left: 16.0, right: 16.0 })
        .size(16)
        .style(theme::TextInput::Custom(Box::new(EditorInputStyle { is_invalid: is_trigger_invalid })));

    let mut trigger_col = column![trigger_label, trigger_input].spacing(8);
    if is_trigger_invalid {
        if let Some(err) = &state.validation_error {
            trigger_col = trigger_col.push(text(err).size(12).style(theme::Text::Color(ERROR)));
        }
    }

    // Description field  ──────────────────
    let desc_label = text("DESCRIPTION")
        .size(10)
        .style(theme::Text::Color(TEXT_SECONDARY));
    let desc_input = text_input("Short description (optional)", &state.description)
        .on_input(Message::EditorDescriptionChanged)
        .padding(iced::Padding { top: 14.0, bottom: 14.0, left: 16.0, right: 16.0 })
        .size(16)
        .style(theme::TextInput::Custom(Box::new(EditorInputStyle { is_invalid: false })));
    let desc_col = column![desc_label, desc_input].spacing(8);

    // Content field  ──────────────────
    let content_label = text("CONTENT")
        .size(10)
        .style(theme::Text::Color(TEXT_SECONDARY));
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

    // Enabled toggle  ──────────────────
    let toggle = toggler(
        Some("Enabled".to_string()),
        state.enabled,
        Message::EditorEnabledToggled,
    )
    .width(Length::Shrink);

    // Action buttons  ──────────────────
    let save_btn = button(
        row![
            text("\u{F26B}").font(iced_aw::BOOTSTRAP_FONT).size(14),
            Space::new(6.0, 0.0),
            text(if state.is_new { "Create" } else { "Save Changes" }).size(13),
        ]
        .align_items(alignment::Alignment::Center)
    )
    .padding(iced::Padding { top: 10.0, bottom: 10.0, left: 20.0, right: 24.0 })
    .style(theme::Button::custom(PrimaryButtonStyle { is_disabled: !state.has_unsaved_changes }))
    .on_press_maybe(if state.has_unsaved_changes { Some(Message::SaveMacro) } else { None });

    let mut actions_row = row![toggle, horizontal_space().width(Length::Fill), save_btn]
        .align_items(alignment::Alignment::Center);

    if !state.is_new {
        actions_row = actions_row.push(Space::new(8.0, Length::Shrink));
        if show_delete_dialog {
            let cancel_btn = button(text("Cancel").size(13))
                .padding(iced::Padding { top: 8.0, bottom: 8.0, left: 16.0, right: 16.0 })
                .style(theme::Button::custom(crate::ui::macro_list::ClearBtnStyle))
                .on_press(Message::CancelDelete);

            let confirm_btn = button(text("Confirm Delete").size(13))
                .padding(iced::Padding { top: 8.0, bottom: 8.0, left: 16.0, right: 16.0 })
                .style(theme::Button::custom(DangerButtonStyle))
                .on_press(Message::ConfirmDelete);

            actions_row = actions_row.push(
                row![
                    text("Are you sure?").size(13).style(theme::Text::Color(TEXT_SECONDARY)),
                    Space::new(8.0, 0.0),
                    cancel_btn,
                    Space::new(4.0, 0.0),
                    confirm_btn,
                ].align_items(alignment::Alignment::Center)
            );
        } else {
            let delete_btn = button(
                row![
                    text("\u{F5DD}").font(iced_aw::BOOTSTRAP_FONT).size(13),
                    Space::new(4.0, 0.0),
                    text("Delete").size(13),
                ].align_items(alignment::Alignment::Center)
            )
            .padding(iced::Padding { top: 8.0, bottom: 8.0, left: 16.0, right: 16.0 })
            .style(theme::Button::custom(DangerButtonStyle))
            .on_press(Message::DeleteMacroClick);

            actions_row = actions_row.push(delete_btn);
        }
    }

    // ─── Separator line ──────────────────
    let separator = container(Space::new(Length::Fill, 1.0))
        .width(Length::Fill)
        .style(|_theme: &Theme| container::Appearance {
            background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.05))),
            ..Default::default()
        });

    // ─── Assembly ──────────────────
    let editor_form = column![
        header_row,
        Space::new(0.0, 20.0),
        trigger_col,
        desc_col,
        content_col,
        separator,
        Space::new(0.0, 4.0),
        actions_row,
    ]
    .spacing(16)
    .padding(iced::Padding { top: 32.0, bottom: 24.0, left: 40.0, right: 40.0 })
    .height(Length::Fill);

    container(editor_form)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
