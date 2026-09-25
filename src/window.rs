use crate::core::{Allocator, Result};
use crate::ffi;
use crate::ffi::emwin_window_config__bindgen_ty_1;

use std::ffi::{CStr, CString};

use bitflags::bitflags;

pub struct Desktop {
    sys: *mut ffi::emwin_desktop,
}

pub struct WindowId(u64);

pub struct JoystickId(u64);

pub enum DesktopEvent {
    WindowClose(WindowId),
    WindowResize(WindowId, [u32; 2]),
    WindowFocusGained(WindowId),
    WindowFocusLost(WindowId),
    InputKeyAction(WindowId, KeyCode, bool),
    InputMouseMotion(WindowId, [f64; 2]),
    InputMouseButtonAction(WindowId, MouseCode, bool),
    InputMouseWheel(WindowId, [f64; 2]),
    JoystickConnect(JoystickId),
    JoystickDisconnect(JoystickId),
}

impl Desktop {
    pub fn poll_events(&mut self) -> Option<DesktopEvent> {
        todo!()
    }

    pub fn wait_events(&mut self) -> Option<DesktopEvent> {
        todo!()
    }
}

pub enum WindowPosition {
    Absolute([u32; 2]),
    Centered,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum WindowMode {
    Windowed,
    Maximized,
    Fullscreen,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum CursorMode {
    Normal,
    Hidden,
    Disabled,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct WindowFlags: u32 {
        const Visible      = 0b00000001;
        const NotDecorated = 0b00000010;
        const LockedSize   = 0b00000100;
        const VSync        = 0b00001000;
    }
}

pub struct WindowConfig<'a> {
    pub window_mode: WindowMode,
    pub cursor_mode: CursorMode,
    pub flags: WindowFlags,
    pub title: String,
    pub position: WindowPosition,
    pub min_size: [u32; 2],
    pub max_size: [u32; 2],
    pub size: [u32; 2],
    pub desktop: Option<&'a Desktop>,
}

pub struct Window {
    sys: ffi::emwin_window,
}

impl<'a> Default for WindowConfig<'a> {
    fn default() -> Self {
        WindowConfig {
            window_mode: WindowMode::Windowed,
            cursor_mode: CursorMode::Normal,
            flags: WindowFlags::empty(),
            title: String::new(),
            position: WindowPosition::Centered,
            min_size: [ 0, 0 ],
            max_size: [ 0, 0 ],
            size: [ 100, 100 ],
            desktop: None,
        }
    }
}

impl Window {
    pub fn open(allocator: &mut Allocator, config: &WindowConfig) -> Result<(Window, Desktop)> {
        let title = CString::new(config.title.clone())
            .expect("Window title contained a NULL byte");

        let c_config = ffi::emwin_window_config {
            window_mode: config.window_mode as u32,
            cursor_mode: config.cursor_mode as u32,
            flags: config.flags.bits(),
            title: title.as_ptr(),
            min_size: ffi::uvec2 { x: config.min_size[0], y: config.min_size[1] },
            max_size: ffi::uvec2 { x: config.max_size[0], y: config.max_size[1] },
            size: ffi::uvec2 { x: config.size[0], y: config.size[1] },
            __bindgen_anon_1: match config.position {
                WindowPosition::Absolute(pos) => {
                    emwin_window_config__bindgen_ty_1 {
                        absolute_pos: ffi::uvec2 { x: pos[0], y: pos[1] },
                    }
                }

                WindowPosition::Centered => {
                    emwin_window_config__bindgen_ty_1 {
                        centered_pos: true,
                    }
                }
            },
        };
        let mut window = Window { 
            sys: unsafe { std::mem::zeroed() }
        };
        let mut desktop = Desktop { 
            sys: unsafe { std::mem::zeroed() } 
        };
        let result = unsafe {
            ffi::emwin_window_open(
                allocator.sys.get() as *mut ffi::em_allocator, 
                &c_config as *const ffi::emwin_window_config, 
                0, 
                &mut window.sys as *mut ffi::emwin_window, 
                &mut desktop.sys as *mut *mut ffi::emwin_desktop)
        };

        if result != ffi::em_result_EMBER_RESULT_OK {
            return Err(result.into());
        }
        Ok((window, desktop))
    }

    pub fn close(&mut self, allocator: &mut Allocator) {
        unsafe { 
            ffi::emwin_window_close(
                allocator.sys.get() as *mut ffi::em_allocator, 
                &mut self.sys as *mut ffi::emwin_window);
        }
    }

    pub fn request_close(&mut self) {
        unsafe { ffi::emwin_window_request_close(&mut self.sys as *mut ffi::emwin_window) }
    }

    pub fn set_visible(&mut self, visible: bool) {
        unsafe {
            ffi::emwin_window_set_visible(&mut self.sys as *mut ffi::emwin_window, visible);
        }
    }

    pub fn visible(&self) -> bool {
        unsafe { ffi::emwin_window_visible(&self.sys as *const ffi::emwin_window) }
    }

    #[inline]
    pub fn size(&self) -> [u32; 2] {
        [self.sys.size.x, self.sys.size.y]
    }

    #[inline]
    pub fn id(&self) -> WindowId {
        WindowId(self.sys.id)
    }

    #[inline]
    pub fn title(&self) -> &str {
        unsafe { CStr::from_ptr(self.sys.title).to_str().unwrap() }
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KeyCode {
    Space,
    Apostrophe,
    Comma,
    Minus,
    Period,
    Slash,
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Semicolon,
    Equal,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    LeftBracket,
    Backslash,
    RightBracket,
    GraveAccent,
    World1,
    World2,
    Escape,
    Enter,
    Tab,
    Backspace,
    Insert,
    Delete,
    Right,
    Left,
    Down,
    Up,
    PageUp,
    PageDown,
    Home,
    End,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    Kp0,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    KpDecimal,
    KpDivide,
    KpMultiply,
    KpSubtract,
    KpAdd,
    KpEnter,
    KpEqual,
    LeftShift,
    LeftControl,
    LeftAlt,
    LeftSuper,
    RightShift,
    RightControl,
    RightAlt,
    RightSuper,
    Menu,
}

pub enum MouseCode {
    Left,
    Right,
    Middle,
}
