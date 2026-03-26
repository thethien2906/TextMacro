#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyboardState, MapVirtualKeyW, ToUnicodeEx, GetKeyboardLayout,
    VIRTUAL_KEY, VK_CONTROL, VK_LCONTROL, VK_RCONTROL, VK_MENU, VK_LMENU, VK_RMENU,
    VK_LWIN, VK_RWIN, VK_BACK, VK_RETURN, VK_TAB, VK_ESCAPE, VK_SPACE, MAPVK_VK_TO_CHAR,
};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
    KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP, MSG,
};
#[cfg(target_os = "windows")]
use std::sync::OnceLock;

#[cfg(not(target_os = "windows"))]
use rdev::{Event, EventType, Key};

use std::sync::mpsc::Sender;

#[derive(Debug, Clone, PartialEq)]
pub enum InputAction {
    Char(char),
    Backspace,
    Reset,
}

pub struct KeyboardListener {
    sender: Sender<InputAction>,
}

#[cfg(target_os = "windows")]
static INPUT_SENDER: OnceLock<Sender<InputAction>> = OnceLock::new();

impl KeyboardListener {
    pub fn new(sender: Sender<InputAction>) -> Self {
        Self { sender }
    }

    pub fn start(self) {
        #[cfg(target_os = "windows")]
        {
            let _ = INPUT_SENDER.set(self.sender);
            std::thread::spawn(move || {
                unsafe {
                    let hook = SetWindowsHookExW(
                        WH_KEYBOARD_LL,
                        Some(low_level_keyboard_proc),
                        None,
                        0,
                    ).expect("Failed to set keyboard hook");

                    let mut msg = MSG::default();
                    while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                        // Keep message loop running for the hook
                    }

                    let _ = UnhookWindowsHookEx(hook);
                }
            });
        }

        #[cfg(not(target_os = "windows"))]
        {
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
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn low_level_keyboard_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code >= 0 {
        let kbd_struct = *(lparam.0 as *const KBDLLHOOKSTRUCT);
        
        // Ignore injected events (sent by IMEs like UniKey or other software)
        // LLKHF_INJECTED is 0x10.
        let is_injected = (kbd_struct.flags.0 & 0x10) != 0;
        
        let vk_code = VIRTUAL_KEY(kbd_struct.vkCode as u16);
        let msg = wparam.0 as u32;

        static mut CTRL: bool = false;
        static mut ALT: bool = false;
        static mut WIN: bool = false;

        match msg {
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                match vk_code {
                    VK_LCONTROL | VK_RCONTROL | VK_CONTROL => CTRL = true,
                    VK_LMENU | VK_RMENU | VK_MENU => ALT = true,
                    VK_LWIN | VK_RWIN => WIN = true,
                    VK_BACK => {
                        if !is_injected {
                            if let Some(sender) = INPUT_SENDER.get() {
                                let _ = sender.send(InputAction::Backspace);
                            }
                        }
                    }
                    VK_RETURN | VK_TAB | VK_ESCAPE => {
                        if !is_injected {
                            if let Some(sender) = INPUT_SENDER.get() {
                                let _ = sender.send(InputAction::Reset);
                            }
                        }
                    }
                    _ => {
                        if !CTRL && !ALT && !WIN && !is_injected {
                            if let Some(c) = translate_char(vk_code, kbd_struct.scanCode) {
                                if let Some(sender) = INPUT_SENDER.get() {
                                    let _ = sender.send(InputAction::Char(c));
                                }
                            }
                        }
                    }
                }
            }
            WM_KEYUP | WM_SYSKEYUP => {
                match vk_code {
                    VK_LCONTROL | VK_RCONTROL | VK_CONTROL => CTRL = false,
                    VK_LMENU | VK_RMENU | VK_MENU => ALT = false,
                    VK_LWIN | VK_RWIN => WIN = false,
                    _ => {}
                }
            }
            _ => {}
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}

#[cfg(target_os = "windows")]
unsafe fn translate_char(vk_code: VIRTUAL_KEY, scan_code: u32) -> Option<char> {
    if vk_code == VK_SPACE {
        return Some(' ');
    }

    let mut keyboard_state = [0u8; 256];
    let _ = GetKeyboardState(&mut keyboard_state);

    let mut buffer = [0u16; 1];
    let layout = GetKeyboardLayout(0);

    // Using ToUnicodeEx to translate. 
    // Flag 0x4 (TO_UNICODE_EX_QUERY_STATE) prevents the function from 
    // modifying the menu/dead-key state, which is critical for UniKey/EVKey.
    let res = ToUnicodeEx(
        vk_code.0 as u32,
        scan_code,
        &keyboard_state,
        &mut buffer,
        4, // TO_UNICODE_EX_QUERY_STATE
        Some(layout),
    );

    if res == 1 {
        let c = char::from_u32(buffer[0] as u32)?;
        if !c.is_control() {
            return Some(c);
        }
    } 

    None
}

