//! TextMacro – A lightweight automation tool for creating macros
//! triggered by typed text, keyboard shortcuts, or system events.

pub mod core;
pub mod injector;
pub mod input;
pub mod models;
pub mod storage;
pub mod ui;

fn main() -> iced::Result {
    ui::app::run()
}
