use iced::widget::{
    button, column, container, pick_list, row, text, text_input, toggler, Space,
};
use iced::{alignment, theme, Element, Length, Theme};

use crate::models::config::Config;
use crate::ui::app::{Message, TEXT_PRIMARY, BORDER};

fn group_title<'a>(title: &str) -> Element<'a, Message> {
    column![
        text(title)
            .size(16)
            .style(theme::Text::Color(TEXT_PRIMARY)),
        // Separator line
        container(Space::new(Length::Fill, 1.0))
            .width(Length::Fill)
            .style(|_theme: &Theme| container::Appearance {
                background: Some(iced::Background::Color(BORDER)),
                ..Default::default()
            })
    ]
    .spacing(8)
    .into()
}

pub fn view<'a>(
    config: &'a Config,
    validation_errors: &'a std::collections::HashMap<String, String>,
    is_recording_shortcut: bool,
) -> Element<'a, Message> {
    
    // 1. General Group
    let run_on_startup = toggler(
        Some("Run at system startup".to_string()),
        config.run_on_startup,
        Message::ToggleRunOnStartup,
    )
    .width(Length::Fill)
    .text_size(14);

    let enable_bg_service = toggler(
        Some("Enable background service".to_string()),
        config.enable_background_service,
        Message::ToggleBackgroundService,
    )
    .width(Length::Fill)
    .text_size(14);

    let mut trigger_prefix_col = column![
        row![
            text("Trigger prefix").size(14).style(theme::Text::Color(TEXT_PRIMARY)),
            Space::new(Length::Fill, 0.0),
            text_input("e.g. /", &config.trigger_prefix)
                .on_input(Message::TriggerPrefixChanged)
                .on_submit(Message::TriggerPrefixSubmit)
                .width(Length::Fixed(100.0))
                .padding(8),
        ]
        .align_items(alignment::Alignment::Center)
    ].spacing(4);

    if let Some(err) = validation_errors.get("trigger_prefix") {
        trigger_prefix_col = trigger_prefix_col.push(
            text(err).size(12).style(theme::Text::Color(crate::ui::app::ERROR))
        );
    }

    let general_group = column![
        group_title("General"),
        run_on_startup,
        enable_bg_service,
        trigger_prefix_col,
    ]
    .spacing(16);

    // 2. Editor Group
    let monospace = toggler(
        Some("Use monospace font".to_string()),
        config.editor_font_monospace,
        Message::ToggleEditorFontMonospace,
    )
    .width(Length::Fill)
    .text_size(14);

    let preserve_fmt = toggler(
        Some("Preserve formatting".to_string()),
        config.preserve_formatting,
        Message::TogglePreserveFormatting,
    )
    .width(Length::Fill)
    .text_size(14);

    let markdown = toggler(
        Some("Markdown support".to_string()),
        config.markdown_support,
        Message::ToggleMarkdownSupport,
    )
    .width(Length::Fill)
    .text_size(14);

    let editor_group = column![
        group_title("Editor"),
        monospace,
        preserve_fmt,
        markdown,
    ]
    .spacing(16);

    // 3. Shortcuts Group
    let shortcut_btn_text = if is_recording_shortcut {
        "Recording... (Press Escape to cancel)"
    } else {
        &config.command_palette_shortcut
    };

    let mut shortcut_col = column![
        row![
            text("Command palette").size(14).style(theme::Text::Color(TEXT_PRIMARY)),
            Space::new(Length::Fill, 0.0),
            button(text(shortcut_btn_text).size(14))
                .padding([8, 12])
                .on_press(Message::StartShortcutRecording)
        ]
        .align_items(alignment::Alignment::Center)
    ].spacing(4);

    if let Some(err) = validation_errors.get("command_palette_shortcut") {
        shortcut_col = shortcut_col.push(
            text(err).size(12).style(theme::Text::Color(crate::ui::app::ERROR))
        );
    }

    let shortcuts_group = column![
        group_title("Shortcuts"),
        shortcut_col,
    ]
    .spacing(16);

    // 4. Appearance Group
    let theme_options = vec!["dark".to_string(), "light".to_string()];
    let theme_picker = pick_list(
        theme_options,
        Some(config.theme.clone()),
        Message::ThemeSelected,
    )
    .width(Length::Fixed(150.0));

    let density_options = vec!["compact".to_string(), "comfortable".to_string()];
    let density_picker = pick_list(
        density_options,
        Some(config.ui_density.clone()),
        Message::UIDensitySelected,
    )
    .width(Length::Fixed(150.0));

    let appearance_group = column![
        group_title("Appearance"),
        row![
            text("Theme").size(14).style(theme::Text::Color(TEXT_PRIMARY)),
            Space::new(Length::Fill, 0.0),
            theme_picker,
        ]
        .align_items(alignment::Alignment::Center),
        row![
            text("UI density").size(14).style(theme::Text::Color(TEXT_PRIMARY)),
            Space::new(Length::Fill, 0.0),
            density_picker,
        ]
        .align_items(alignment::Alignment::Center),
    ]
    .spacing(16);

    let data_group = column![
        group_title("Data Management"),
        row![
            button("Import Macros").on_press(Message::ImportMacrosClicked).padding(8),
            Space::new(16.0, 0.0),
            button("Export Macros").on_press(Message::ExportMacrosClicked).padding(8),
        ]
    ]
    .spacing(16);

    // Assembly
    let content = column![
        text("Settings").size(24).style(theme::Text::Color(TEXT_PRIMARY)),
        Space::new(Length::Fill, 8.0),
        general_group,
        editor_group,
        shortcuts_group,
        appearance_group,
        data_group,
    ]
    .spacing(32)
    .padding(32)
    .max_width(800);

    container(iced::widget::scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .into()
}
