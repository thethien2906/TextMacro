// ui/settings_panel.rs — Editorial Macro Suite Design
use iced::widget::{
    button, column, container, pick_list, row, text, text_input, toggler, Space, horizontal_space,
};
use iced::{alignment, theme, Background, Border, Color, Element, Length, Theme};
use iced_aw::BOOTSTRAP_FONT;

use crate::models::config::Config;
use crate::ui::app::{
    Message, ACCENT, PANEL, SURFACE_BRIGHT, SURFACE_HIGHEST,
    TEXT_PRIMARY, TEXT_SECONDARY, SECONDARY, ERROR,
};

// ────────────────────────────────
//  Section header with left accent border
// ────────────────────────────────
fn section_header<'a>(title: &'a str, color: Color) -> Element<'a, Message> {
    row![
        container(Space::new(0.0, 24.0))
            .width(Length::Fixed(4.0))
            .style(move |_theme: &Theme| container::Appearance {
                background: Some(Background::Color(color)),
                ..Default::default()
            }),
        Space::new(12.0, 0.0),
        text(title).size(20).style(theme::Text::Color(TEXT_PRIMARY))
    ].align_items(alignment::Alignment::Center).into()
}

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
    .padding(iced::Padding { top: 24.0, bottom: 24.0, left: 24.0, right: 24.0 })
    .style(settings_card_style)
    .into()
}

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
            .size(36)
            .style(theme::Text::Color(TEXT_PRIMARY)),
        text("Configure your professional macro environment.")
            .size(18)
            .style(theme::Text::Color(TEXT_SECONDARY)),
    ].spacing(8);

    // ══════════════════════════════════
    //  Section 1: General
    // ══════════════════════════════════
    let general_header = section_header("General", ACCENT);

    let general_cards = row![
        container(toggle_card("Run at system startup", "Automatically launch when logging in.", config.run_on_startup, Message::ToggleRunOnStartup)).width(Length::FillPortion(1)),
        Space::new(16.0, 0.0),
        container(toggle_card("Enable background service", "Keep macros active when window is closed.", config.enable_background_service, Message::ToggleBackgroundService)).width(Length::FillPortion(1)),
    ];

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
        .padding(iced::Padding { top: 16.0, bottom: 16.0, left: 24.0, right: 24.0 })
        .style(settings_card_style)
    ].spacing(4);

    if let Some(err) = validation_errors.get("trigger_prefix") {
        trigger_prefix_col = trigger_prefix_col.push(
            text(err).size(12).style(theme::Text::Color(ERROR))
        );
    }

    let general_section = column![
        general_header,
        general_cards,
        trigger_prefix_col,
    ].spacing(24);

    // ══════════════════════════════════
    //  Section 2: Editor
    // ══════════════════════════════════
    let editor_header = section_header("Editor", SECONDARY);

    let editor_card = container(
        column![
            // Monospace
            container(
                row![
                    text("\u{F5F7}").font(BOOTSTRAP_FONT).size(20).style(theme::Text::Color(SECONDARY)),
                    Space::new(16.0, 0.0),
                    column![
                        text("Use monospace font").size(14).style(theme::Text::Color(TEXT_PRIMARY)),
                        text("Optimize for script readability.").size(12).style(theme::Text::Color(TEXT_SECONDARY)),
                    ].spacing(4),
                    horizontal_space().width(Length::Fill),
                    toggler(None, config.editor_font_monospace, Message::ToggleEditorFontMonospace).width(Length::Shrink)
                ].align_items(alignment::Alignment::Center)
            ).padding(iced::Padding { top: 24.0, bottom: 24.0, left: 24.0, right: 24.0 }),
            
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
                    text("\u{F5C7}").font(BOOTSTRAP_FONT).size(20).style(theme::Text::Color(SECONDARY)),
                    Space::new(16.0, 0.0),
                    column![
                        text("Preserve formatting").size(14).style(theme::Text::Color(TEXT_PRIMARY)),
                        text("Maintain indentation on copy/paste.").size(12).style(theme::Text::Color(TEXT_SECONDARY)),
                    ].spacing(4),
                    horizontal_space().width(Length::Fill),
                    toggler(None, config.preserve_formatting, Message::TogglePreserveFormatting).width(Length::Shrink)
                ].align_items(alignment::Alignment::Center)
            ).padding(iced::Padding { top: 24.0, bottom: 24.0, left: 24.0, right: 24.0 }),
            
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
                    text("\u{F481}").font(BOOTSTRAP_FONT).size(20).style(theme::Text::Color(SECONDARY)),
                    Space::new(16.0, 0.0),
                    column![
                        text("Markdown support").size(14).style(theme::Text::Color(TEXT_PRIMARY)),
                        text("Enable rich text preview for comments.").size(12).style(theme::Text::Color(TEXT_SECONDARY)),
                    ].spacing(4),
                    horizontal_space().width(Length::Fill),
                    toggler(None, config.markdown_support, Message::ToggleMarkdownSupport).width(Length::Shrink)
                ].align_items(alignment::Alignment::Center)
            ).padding(iced::Padding { top: 24.0, bottom: 24.0, left: 24.0, right: 24.0 }),
        ]
    )
    .style(settings_card_style);

    let editor_section = column![
        editor_header,
        editor_card,
    ].spacing(24);

    // ══════════════════════════════════
    //  Section 3: Shortcuts
    // ══════════════════════════════════
    let tertiary_color = Color::from_rgb(1.0, 0.647, 0.851);
    let shortcuts_header = section_header("Shortcuts", tertiary_color);

    let shortcut_btn_text = if is_recording_shortcut {
        "Recording... (Esc to cancel)"
    } else {
        &config.command_palette_shortcut
    };

    let command_palette_card = container(
        column![
            row![
                text("\u{F451}").font(BOOTSTRAP_FONT).size(20).style(theme::Text::Color(tertiary_color)),
                Space::new(8.0, 0.0),
                text("Command Palette").size(16).style(theme::Text::Color(TEXT_PRIMARY)),
            ].align_items(alignment::Alignment::Center),
            Space::new(0.0, 24.0),
            
            row![
                text("Open Palette").size(14).style(theme::Text::Color(TEXT_SECONDARY)),
                horizontal_space().width(Length::Fill),
                button(
                    text(shortcut_btn_text).size(12).style(theme::Text::Color(ACCENT))
                )
                .padding(iced::Padding { top: 6.0, bottom: 6.0, left: 12.0, right: 12.0 })
                .style(theme::Button::custom(KeybindButtonStyle))
                .on_press(Message::StartShortcutRecording)
            ].align_items(alignment::Alignment::Center),
            
            Space::new(0.0, 16.0),
            container(Space::new(Length::Fill, 1.0))
                .width(Length::Fill)
                .style(|_theme: &Theme| container::Appearance {
                    background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.05))),
                    ..Default::default()
                }),
            Space::new(0.0, 16.0),
            
            row![
                text("Global Toggle").size(14).style(theme::Text::Color(TEXT_SECONDARY)),
                horizontal_space().width(Length::Fill),
                container(text("CMD + SHIFT + E").size(12).style(theme::Text::Color(ACCENT)))
                    .padding(iced::Padding { top: 6.0, bottom: 6.0, left: 12.0, right: 12.0 })
                    .style(|_theme: &Theme| container::Appearance {
                        background: Some(Background::Color(SURFACE_BRIGHT)),
                        border: Border {
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                            width: 0.0,
                            radius: 8.0.into(),
                        },
                        ..Default::default()
                    })
            ].align_items(alignment::Alignment::Center),
        ]
    )
    .padding(32.0)
    .style(settings_card_style)
    .width(Length::FillPortion(2));

    let custom_mapping_card = container(
        column![
            text("\u{F46C}").font(BOOTSTRAP_FONT).size(36).style(theme::Text::Color(ACCENT)),
            Space::new(0.0, 16.0),
            text("Custom Mapping").size(16).style(theme::Text::Color(TEXT_PRIMARY)),
            Space::new(0.0, 4.0),
            text("Create your own productivity shortcuts.")
                .size(12)
                .style(theme::Text::Color(TEXT_SECONDARY))
                .horizontal_alignment(alignment::Horizontal::Center),
        ].align_items(alignment::Alignment::Center)
    )
    .padding(32.0)
    .style(|_theme: &Theme| container::Appearance {
        background: Some(Background::Color(Color::from_rgb(0.125, 0.125, 0.122))),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 12.0.into(),
        },
        ..Default::default()
    })
    .width(Length::FillPortion(1));

    let shortcuts_row = row![
        command_palette_card,
        Space::new(24.0, 0.0),
        custom_mapping_card
    ];

    let mut shortcut_col = column![shortcuts_row].spacing(4);

    if let Some(err) = validation_errors.get("command_palette_shortcut") {
        shortcut_col = shortcut_col.push(
            text(err).size(12).style(theme::Text::Color(ERROR))
        );
    }

    let shortcuts_section = column![
        shortcuts_header,
        shortcut_col,
    ].spacing(24);

    // ══════════════════════════════════
    //  Section 4: Appearance
    // ══════════════════════════════════
    let appearance_header = section_header("Appearance", TEXT_PRIMARY);

    let theme_options = vec!["Midnight Obsidian (Default)".to_string(), "Deep Cobalt".to_string(), "Editorial Light".to_string()];
    let current_theme = if theme_options.contains(&config.theme) { config.theme.clone() } else if config.theme == "light" { "Editorial Light".to_string() } else { "Midnight Obsidian (Default)".to_string() };
    let theme_picker = pick_list(
        theme_options,
        Some(current_theme),
        Message::ThemeSelected,
    )
    .width(Length::Fill)
    .padding(16)
    .style(theme::PickList::Custom(
        std::rc::Rc::new(PickListStyle),
        std::rc::Rc::new(PickListMenuStyle)
    ));

    let density_options = vec!["Comfortable".to_string(), "Compact".to_string(), "Zen Mode".to_string()];
    let current_density = if density_options.contains(&config.ui_density) { config.ui_density.clone() } else if config.ui_density == "compact" { "Compact".to_string() } else { "Comfortable".to_string() };
    let density_picker = pick_list(
        density_options,
        Some(current_density),
        Message::UIDensitySelected,
    )
    .width(Length::Fill)
    .padding(16)
    .style(theme::PickList::Custom(
        std::rc::Rc::new(PickListStyle),
        std::rc::Rc::new(PickListMenuStyle)
    ));

    let appearance_dropdowns = row![
        column![
            text("THEME ENGINE").size(12).style(theme::Text::Color(TEXT_SECONDARY)),
            Space::new(0.0, 8.0),
            theme_picker
        ].width(Length::FillPortion(1)),
        Space::new(32.0, 0.0),
        column![
            text("UI DENSITY").size(12).style(theme::Text::Color(TEXT_SECONDARY)),
            Space::new(0.0, 8.0),
            density_picker
        ].width(Length::FillPortion(1)),
    ].width(Length::Fill);

    let visual_preview = container(
        row![
            container(
                row![
                    container(Space::new(0.0, 0.0)).width(Length::Fixed(8.0)).height(Length::Fixed(8.0)).style(|_t: &Theme| container::Appearance {
                        background: Some(Background::Color(Color::from_rgba(0.639, 0.651, 1.0, 0.4))),
                        border: Border { radius: 4.0.into(), ..Default::default() },
                        ..Default::default()
                    }),
                    Space::new(4.0, 0.0),
                    container(Space::new(0.0, 0.0)).width(Length::Fixed(32.0)).height(Length::Fixed(8.0)).style(|_t: &Theme| container::Appearance {
                        background: Some(Background::Color(Color::from_rgba(0.639, 0.651, 1.0, 0.6))),
                        border: Border { radius: 4.0.into(), ..Default::default() },
                        ..Default::default()
                    }),
                ].align_items(alignment::Alignment::Center)
            )
            .width(Length::Fixed(96.0))
            .height(Length::Fixed(56.0))
            .center_x().center_y()
            .style(|_theme: &Theme| container::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.125, 0.125, 0.122))),
                border: Border {
                    color: Color::from_rgba(0.639, 0.651, 1.0, 0.2),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }),
            Space::new(24.0, 0.0),
            column![
                text("Visual Preview").size(14).style(theme::Text::Color(TEXT_PRIMARY)),
                text("A snapshot of how your workspace will look with current settings.").size(12).style(theme::Text::Color(TEXT_SECONDARY)),
            ],
            horizontal_space().width(Length::Fill),
            text("RESET TO FACTORY").size(12).style(theme::Text::Color(ACCENT))
        ].align_items(alignment::Alignment::Center)
    )
    .padding(24.0)
    .style(settings_card_style)
    .width(Length::Fill);

    let appearance_section = column![
        appearance_header,
        appearance_dropdowns,
        Space::new(0.0, 8.0),
        visual_preview
    ].spacing(24);

    // ══════════════════════════════════
    //  Action Bar
    // ══════════════════════════════════

    let action_bar = container(
        column![
            container(Space::new(Length::Fill, 1.0))
                .width(Length::Fill)
                .style(|_theme: &Theme| container::Appearance {
                    background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.05))),
                    ..Default::default()
                }),
            Space::new(0.0, 24.0),
            row![
                text("Last synced: 2 minutes ago").size(12).style(theme::Text::Color(TEXT_SECONDARY)),
                horizontal_space().width(Length::Fill),
            ].align_items(alignment::Alignment::Center)
        ]
    )
    .width(Length::Fill);

    // ═══════════════════════════════════════
    //  Full page assembly
    // ═══════════════════════════════════════
    let content = column![
        page_header,
        Space::new(Length::Fill, 32.0),
        general_section,
        editor_section,
        shortcuts_section,
        appearance_section,
        Space::new(Length::Fill, 32.0),
        action_bar
    ]
    .spacing(32)
    .padding(iced::Padding { top: 48.0, bottom: 64.0, left: 48.0, right: 48.0 })
    .max_width(900);

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
                width: 1.0, // Used to be 0
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

struct TransparentButtonStyle;
impl button::StyleSheet for TransparentButtonStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: TEXT_SECONDARY,
            border: Border { color: Color::TRANSPARENT, width: 0.0, radius: 8.0.into() },
            ..Default::default()
        }
    }
    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        let mut app = self.active(_style);
        app.text_color = TEXT_PRIMARY;
        app
    }
}

struct PrimaryActionStyle;
impl button::StyleSheet for PrimaryActionStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(ACCENT)),
            text_color: Color::from_rgb(0.05, 0.0, 0.6), // Dark primary
            border: Border { color: Color::TRANSPARENT, width: 0.0, radius: 12.0.into() },
            ..Default::default()
        }
    }
    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        let mut app = self.active(_style);
        app.text_color = Color::BLACK;
        app
    }
}

struct PickListStyle;
impl pick_list::StyleSheet for PickListStyle {
    type Style = Theme;

    fn active(&self, _style: &<Self as pick_list::StyleSheet>::Style) -> pick_list::Appearance {
        pick_list::Appearance {
            text_color: TEXT_PRIMARY,
            placeholder_color: TEXT_SECONDARY,
            handle_color: TEXT_SECONDARY,
            background: Background::Color(PANEL),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 12.0.into(),
            },
        }
    }

    fn hovered(&self, _style: &<Self as pick_list::StyleSheet>::Style) -> pick_list::Appearance {
        let mut app = self.active(_style);
        app.background = Background::Color(SURFACE_HIGHEST);
        app.handle_color = TEXT_PRIMARY;
        app
    }
}

struct PickListMenuStyle;
impl iced::overlay::menu::StyleSheet for PickListMenuStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::overlay::menu::Appearance {
        iced::overlay::menu::Appearance {
            text_color: TEXT_PRIMARY,
            background: Background::Color(PANEL),
            border: Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.1),
                width: 1.0,
                radius: 8.0.into(),
            },
            selected_text_color: TEXT_PRIMARY,
            selected_background: Background::Color(SURFACE_HIGHEST),
        }
    }
}
