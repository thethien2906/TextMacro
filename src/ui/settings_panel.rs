// ui/settings_panel.rs — Editorial Macro Suite Design
use iced::widget::{
    button, column, container, pick_list, row, text, text_input, toggler, Space,
};
use iced::{alignment, theme, Background, Border, Color, Element, Length, Theme};

use crate::models::config::Config;
use crate::ui::app::{
    Message, ACCENT, PANEL, SURFACE_BRIGHT, SURFACE_HIGHEST,
    TEXT_PRIMARY, TEXT_SECONDARY,
};

// ────────────────────────────────
//  Section header with left accent border
// ────────────────────────────────


// ────────────────────────────────
//  Settings card container
// ────────────────────────────────
fn settings_card_style(_theme: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(Background::Color(PANEL)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 12.0.into(),
        },
        ..Default::default()
    }
}

// ────────────────────────────────
//  Toggle row component
// ────────────────────────────────
fn toggle_card<'a>(
    title: &'a str,
    subtitle: &'a str,
    is_checked: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> Element<'a, Message> {
    container(
        row![
            column![
                text(title).size(14).style(theme::Text::Color(TEXT_PRIMARY)),
                text(subtitle).size(12).style(theme::Text::Color(TEXT_SECONDARY)),
            ].spacing(4),
            horizontal_space().width(Length::Fill),
            toggler(None, is_checked, on_toggle).width(Length::Shrink)
        ]
        .align_items(alignment::Alignment::Center)
    )
    .padding(iced::Padding { top: 16.0, bottom: 16.0, left: 20.0, right: 20.0 })
    .style(settings_card_style)
    .into()
}

use iced::widget::horizontal_space;

// ═══════════════════════════════════════
//  Main settings view
// ═══════════════════════════════════════
pub fn view<'a>(
    config: &'a Config,
    validation_errors: &'a std::collections::HashMap<String, String>,
    is_recording_shortcut: bool,
) -> Element<'a, Message> {

    // ──────────── Page Header ────────────
    let page_header = column![
        text("Preferences")
            .size(32)
            .style(theme::Text::Color(TEXT_PRIMARY)),
        text("Configure your professional macro environment.")
            .size(16)
            .style(theme::Text::Color(TEXT_SECONDARY)),
    ].spacing(6);

    // ══════════════════════════════════
    //  Section 1: General
    // ══════════════════════════════════
    let general_header = container(
        text("General").size(18).style(theme::Text::Color(TEXT_PRIMARY))
    )
    .padding(iced::Padding { top: 0.0, bottom: 0.0, left: 12.0, right: 0.0 })
    .style(|_theme: &Theme| container::Appearance {
        border: Border {
            color: Color::from_rgb(0.639, 0.651, 1.0),  // primary accent
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    });

    let general_cards = column![
        toggle_card("Run at system startup", "Automatically launch when logging in.", config.run_on_startup, Message::ToggleRunOnStartup),
        toggle_card("Enable background service", "Keep macros active when window is closed.", config.enable_background_service, Message::ToggleBackgroundService),
    ].spacing(8);

    // Trigger prefix
    let mut trigger_prefix_col = column![
        container(
            row![
                text("Trigger prefix").size(14).style(theme::Text::Color(TEXT_PRIMARY)),
                Space::new(Length::Fill, 0.0),
                text_input("e.g. /", &config.trigger_prefix)
                    .on_input(Message::TriggerPrefixChanged)
                    .on_submit(Message::TriggerPrefixSubmit)
                    .width(Length::Fixed(100.0))
                    .padding(10),
            ]
            .align_items(alignment::Alignment::Center)
        )
        .padding(iced::Padding { top: 16.0, bottom: 16.0, left: 20.0, right: 20.0 })
        .style(settings_card_style)
    ].spacing(4);

    if let Some(err) = validation_errors.get("trigger_prefix") {
        trigger_prefix_col = trigger_prefix_col.push(
            text(err).size(12).style(theme::Text::Color(crate::ui::app::ERROR))
        );
    }

    let general_section = column![
        general_header,
        general_cards,
        trigger_prefix_col,
    ].spacing(12);

    // ══════════════════════════════════
    //  Section 2: Editor
    // ══════════════════════════════════
    let editor_header = container(
        text("Editor").size(18).style(theme::Text::Color(TEXT_PRIMARY))
    )
    .padding(iced::Padding { top: 0.0, bottom: 0.0, left: 12.0, right: 0.0 })
    .style(|_theme: &Theme| container::Appearance {
        border: Border {
            color: Color::from_rgb(0.635, 0.557, 0.988),  // secondary accent
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    });

    // Editor settings in a single stacked card container
    let editor_card = container(
        column![
            // Monospace
            container(
                row![
                    column![
                        text("Use monospace font").size(14).style(theme::Text::Color(TEXT_PRIMARY)),
                        text("Optimize for script readability.").size(12).style(theme::Text::Color(TEXT_SECONDARY)),
                    ].spacing(4),
                    horizontal_space().width(Length::Fill),
                    toggler(None, config.editor_font_monospace, Message::ToggleEditorFontMonospace).width(Length::Shrink)
                ].align_items(alignment::Alignment::Center)
            ).padding(iced::Padding { top: 16.0, bottom: 16.0, left: 20.0, right: 20.0 }),
            // Divider
            container(Space::new(Length::Fill, 1.0))
                .width(Length::Fill)
                .style(|_theme: &Theme| container::Appearance {
                    background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.05))),
                    ..Default::default()
                }),
            // Preserve formatting
            container(
                row![
                    column![
                        text("Preserve formatting").size(14).style(theme::Text::Color(TEXT_PRIMARY)),
                        text("Maintain indentation on copy/paste.").size(12).style(theme::Text::Color(TEXT_SECONDARY)),
                    ].spacing(4),
                    horizontal_space().width(Length::Fill),
                    toggler(None, config.preserve_formatting, Message::TogglePreserveFormatting).width(Length::Shrink)
                ].align_items(alignment::Alignment::Center)
            ).padding(iced::Padding { top: 16.0, bottom: 16.0, left: 20.0, right: 20.0 }),
            // Divider
            container(Space::new(Length::Fill, 1.0))
                .width(Length::Fill)
                .style(|_theme: &Theme| container::Appearance {
                    background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.05))),
                    ..Default::default()
                }),
            // Markdown
            container(
                row![
                    column![
                        text("Markdown support").size(14).style(theme::Text::Color(TEXT_PRIMARY)),
                        text("Enable rich text preview for comments.").size(12).style(theme::Text::Color(TEXT_SECONDARY)),
                    ].spacing(4),
                    horizontal_space().width(Length::Fill),
                    toggler(None, config.markdown_support, Message::ToggleMarkdownSupport).width(Length::Shrink)
                ].align_items(alignment::Alignment::Center)
            ).padding(iced::Padding { top: 16.0, bottom: 16.0, left: 20.0, right: 20.0 }),
        ]
    )
    .style(settings_card_style);

    let editor_section = column![
        editor_header,
        editor_card,
    ].spacing(12);

    // ══════════════════════════════════
    //  Section 3: Shortcuts
    // ══════════════════════════════════
    let shortcuts_header = container(
        text("Shortcuts").size(18).style(theme::Text::Color(TEXT_PRIMARY))
    )
    .padding(iced::Padding { top: 0.0, bottom: 0.0, left: 12.0, right: 0.0 })
    .style(|_theme: &Theme| container::Appearance {
        border: Border {
            color: Color::from_rgb(1.0, 0.647, 0.851),  // tertiary (ffa5d9)
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    });

    let shortcut_btn_text = if is_recording_shortcut {
        "Recording... (Press Escape to cancel)"
    } else {
        &config.command_palette_shortcut
    };

    let mut shortcut_col = column![
        container(
            row![
                text("Command palette").size(14).style(theme::Text::Color(TEXT_SECONDARY)),
                Space::new(Length::Fill, 0.0),
                button(
                    text(shortcut_btn_text).size(12).style(theme::Text::Color(ACCENT))
                )
                .padding(iced::Padding { top: 6.0, bottom: 6.0, left: 12.0, right: 12.0 })
                .style(theme::Button::custom(KeybindButtonStyle))
                .on_press(Message::StartShortcutRecording)
            ]
            .align_items(alignment::Alignment::Center)
        )
        .padding(iced::Padding { top: 12.0, bottom: 12.0, left: 20.0, right: 20.0 })
        .style(settings_card_style)
    ].spacing(4);

    if let Some(err) = validation_errors.get("command_palette_shortcut") {
        shortcut_col = shortcut_col.push(
            text(err).size(12).style(theme::Text::Color(crate::ui::app::ERROR))
        );
    }

    let shortcuts_section = column![
        shortcuts_header,
        shortcut_col,
    ].spacing(12);

    // ══════════════════════════════════
    //  Section 4: Appearance
    // ══════════════════════════════════
    let appearance_header = container(
        text("Appearance").size(18).style(theme::Text::Color(TEXT_PRIMARY))
    )
    .padding(iced::Padding { top: 0.0, bottom: 0.0, left: 12.0, right: 0.0 })
    .style(|_theme: &Theme| container::Appearance {
        border: Border {
            color: TEXT_PRIMARY,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    });

    let theme_options = vec!["dark".to_string(), "light".to_string()];
    let theme_picker = pick_list(
        theme_options,
        Some(config.theme.clone()),
        Message::ThemeSelected,
    )
    .width(Length::Fixed(180.0));

    let density_options = vec!["compact".to_string(), "comfortable".to_string()];
    let density_picker = pick_list(
        density_options,
        Some(config.ui_density.clone()),
        Message::UIDensitySelected,
    )
    .width(Length::Fixed(180.0));

    let appearance_cards = column![
        container(
            row![
                text("Theme").size(14).style(theme::Text::Color(TEXT_PRIMARY)),
                Space::new(Length::Fill, 0.0),
                theme_picker,
            ].align_items(alignment::Alignment::Center)
        )
        .padding(iced::Padding { top: 14.0, bottom: 14.0, left: 20.0, right: 20.0 })
        .style(settings_card_style),
        container(
            row![
                text("UI density").size(14).style(theme::Text::Color(TEXT_PRIMARY)),
                Space::new(Length::Fill, 0.0),
                density_picker,
            ].align_items(alignment::Alignment::Center)
        )
        .padding(iced::Padding { top: 14.0, bottom: 14.0, left: 20.0, right: 20.0 })
        .style(settings_card_style),
    ].spacing(8);

    let appearance_section = column![
        appearance_header,
        appearance_cards,
    ].spacing(12);

    // ══════════════════════════════════
    //  Section 5: Data Management
    // ══════════════════════════════════
    let data_header = container(
        text("Data Management").size(18).style(theme::Text::Color(TEXT_PRIMARY))
    )
    .padding(iced::Padding { top: 0.0, bottom: 0.0, left: 12.0, right: 0.0 });

    let data_cards = container(
        row![
            button(
                text("Import Macros").size(13).style(theme::Text::Color(TEXT_PRIMARY))
            )
            .padding(iced::Padding { top: 10.0, bottom: 10.0, left: 20.0, right: 20.0 })
            .style(theme::Button::custom(SecondaryButtonStyle))
            .on_press(Message::ImportMacrosClicked),
            Space::new(12.0, 0.0),
            button(
                text("Export Macros").size(13).style(theme::Text::Color(TEXT_PRIMARY))
            )
            .padding(iced::Padding { top: 10.0, bottom: 10.0, left: 20.0, right: 20.0 })
            .style(theme::Button::custom(SecondaryButtonStyle))
            .on_press(Message::ExportMacrosClicked),
        ]
    );

    let data_section = column![
        data_header,
        data_cards,
    ].spacing(12);

    // ═══════════════════════════════════════
    //  Full page assembly
    // ═══════════════════════════════════════
    let content = column![
        page_header,
        Space::new(Length::Fill, 12.0),
        general_section,
        editor_section,
        shortcuts_section,
        appearance_section,
        data_section,
    ]
    .spacing(32)
    .padding(iced::Padding { top: 32.0, bottom: 48.0, left: 40.0, right: 40.0 })
    .max_width(800);

    container(iced::widget::scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .into()
}

// ════════════════════════════════
//  Style structs
// ════════════════════════════════

struct KeybindButtonStyle;
impl button::StyleSheet for KeybindButtonStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(SURFACE_BRIGHT)),
            text_color: ACCENT,
            border: Border {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                width: 0.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }
    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        let mut app = self.active(_style);
        app.background = Some(Background::Color(SURFACE_HIGHEST));
        app
    }
}

struct SecondaryButtonStyle;
impl button::StyleSheet for SecondaryButtonStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(SURFACE_HIGHEST)),
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
        app.background = Some(Background::Color(SURFACE_BRIGHT));
        app
    }
}
