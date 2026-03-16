use rdev::{Event, EventType, Key};
use std::sync::mpsc::Sender;

#[derive(Debug, Clone, PartialEq)]
pub enum InputAction {
    Char(char),
    Backspace,
    Reset,
    // Add more variants if needed later
}

pub struct KeyboardListener {
    sender: Sender<InputAction>,
}

impl KeyboardListener {
    pub fn new(sender: Sender<InputAction>) -> Self {
        Self { sender }
    }

    pub fn start(self) {
        // rdev::listen blocks the current thread. 
        // We spawn a new thread to run it.
        std::thread::spawn(move || {
            let mut ctrl_pressed = false;
            let mut alt_pressed = false;
            let mut win_pressed = false;

            let callback = move |event: Event| {
                match event.event_type {
                    EventType::KeyPress(key) => {
                        match key {
                            Key::ControlLeft | Key::ControlRight => ctrl_pressed = true,
                            Key::Alt | Key::AltGr => alt_pressed = true,
                            Key::MetaLeft | Key::MetaRight => win_pressed = true,
                            Key::Backspace => {
                                let _ = self.sender.send(InputAction::Backspace);
                            }
                            Key::Return | Key::Tab | Key::Escape => {
                                let _ = self.sender.send(InputAction::Reset);
                            }
                            _ => {
                                if !ctrl_pressed && !alt_pressed && !win_pressed {
                                    if let Some(name) = event.name {
                                        if name.chars().count() == 1 {
                                            if let Some(c) = name.chars().next() {
                                                // Ignore control characters
                                                if !c.is_control() {
                                                    let _ = self.sender.send(InputAction::Char(c));
                                                }
                                            }
                                        }
                                    } else if key == Key::Space {
                                        let _ = self.sender.send(InputAction::Char(' '));
                                    }
                                }
                            }
                        }
                    }
                    EventType::KeyRelease(key) => {
                        match key {
                            Key::ControlLeft | Key::ControlRight => ctrl_pressed = false,
                            Key::Alt | Key::AltGr => alt_pressed = false,
                            Key::MetaLeft | Key::MetaRight => win_pressed = false,
                            _ => {}
                        }
                    }
                    _ => {}
                }
            };

            if let Err(e) = rdev::listen(callback) {
                log::error!("Error listening to global keyboard hook: {:?}", e);
            }
        });
    }
}
