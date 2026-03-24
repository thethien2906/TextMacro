use iced::widget::pick_list;
use iced::{theme, Background, Border, Color, Theme};
use crate::ui::app::{PANEL, TEXT_PRIMARY, TEXT_SECONDARY, SURFACE_HIGHEST};

pub struct ScratchPickListStyle;

impl pick_list::StyleSheet for ScratchPickListStyle {
    type Style = Theme;

    fn active(&self, _style: &<Self as pick_list::StyleSheet>::Style) -> pick_list::Appearance {
        pick_list::Appearance {
            text_color: TEXT_PRIMARY,
            placeholder_color: TEXT_SECONDARY,
            handle_color: TEXT_PRIMARY,
            background: Background::Color(PANEL),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
        }
    }

    fn hovered(&self, _style: &<Self as pick_list::StyleSheet>::Style) -> pick_list::Appearance {
        let mut app = self.active(_style);
        app.background = Background::Color(SURFACE_HIGHEST);
        app
    }
}
