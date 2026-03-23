//! TextMacro – A lightweight automation tool for creating macros
//! triggered by typed text, keyboard shortcuts, or system events.

pub mod core;
pub mod injector;
pub mod input;
pub mod models;
pub mod storage;
pub mod ui;

use simplelog::{WriteLogger, TermLogger, CombinedLogger, LevelFilter, ColorChoice, TerminalMode};
use std::fs::OpenOptions;

use crate::storage::macro_repository::StorageManager;
use crate::core::engine::Engine;

fn setup_logging() {
    let data_dir = match crate::storage::paths::resolve_data_dir() {
        Ok(d) => d,
        Err(_) => return,
    };
    
    let logs_dir = crate::storage::paths::logs_dir(&data_dir);
    let _ = std::fs::create_dir_all(&logs_dir);
    
    let log_file_path = logs_dir.join("app.log");
    
    // Quick rotation: if file > 10MB, rename it
    if let Ok(meta) = std::fs::metadata(&log_file_path) {
        if meta.len() > 10 * 1024 * 1024 {
            let backup_path = logs_dir.join("app.old.log");
            let _ = std::fs::rename(&log_file_path, &backup_path);
        }
    }

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path);

    if let Ok(f) = file {
        let _ = CombinedLogger::init(
            vec![
                TermLogger::new(LevelFilter::Info, simplelog::Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
                WriteLogger::new(LevelFilter::Info, simplelog::Config::default(), f),
            ]
        );
    }
}

fn main() -> iced::Result {
    setup_logging();
    log::info!("Starting TextMacro engine...");

    let tray_menu = tray_icon::menu::Menu::new();
    let _ = tray_menu.append_items(&[
        &tray_icon::menu::MenuItem::with_id("show", "Show", true, None),
        &tray_icon::menu::PredefinedMenuItem::separator(),
        &tray_icon::menu::MenuItem::with_id("quit", "Exit", true, None),
    ]);

    let tray_icon_result = tray_icon::TrayIconBuilder::new()
        .with_tooltip("TextMacro")
        // Just a simple transparent 16x16 icon to avoid panic
        .with_icon(tray_icon::Icon::from_rgba(vec![255; 4 * 16 * 16], 16, 16).unwrap())
        .with_menu(Box::new(tray_menu))
        .build();
        
    let _tray_icon = match tray_icon_result {
        Ok(t) => Some(t),
        Err(e) => {
            log::error!("Failed to create tray icon: {}", e);
            None
        }
    };

    let storage = StorageManager::new().expect("Failed to initialize storage");
    let (tx, rx) = Engine::spawn(storage);
    ui::app::run((tx, rx))
}
