use crate::core::{Allocator, Result};
use crate::ffi;
use crate::ffi::emwin_window_config__bindgen_ty_1;

use std::ffi::{CStr, CString};

use bitflags::bitflags;

pub struct Desktop {
    sys: *mut ffi::emwin_desktop,
}

pub struct Monitor {
    id: MontiorId,
}

impl From<ffi::emwin_monitor> for Monitor {
    fn from(value: ffi::emwin_monitor) -> Self {
        Monitor { id: value.id.into() }
    }
}

pub struct WindowId(u64);

pub struct MontiorId(u64);

pub struct JoystickId(u64);

pub enum DesktopEvent {
    MonitorConnect(Monitor),
    MonitorDisconnect(Monitor),
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

impl From<u64> for WindowId {
    fn from(value: u64) -> Self {
        WindowId(value)
    }
}

impl From<u64> for MontiorId {
    fn from(value: u64) -> Self {
        MontiorId(value)
    }
}

impl From<u64> for JoystickId {
    fn from(value: u64) -> Self {
        JoystickId(value)
    }
}

impl Desktop {
    pub fn poll_events(&mut self) -> Result<DesktopEvent> {
        let mut event = unsafe { ffi::emwin_desktop_event { ..std::mem::zeroed() } };

        let result = unsafe {
            ffi::emwin_poll_events(
                self.sys,
                &mut event as *mut ffi::emwin_desktop_event,
            )
        };

        if result != ffi::em_result_EMBER_RESULT_OK {
            return Err(result.into());
        }

        Ok(unsafe {
            match event.type_ {
                ffi::emwin_event_type_EMWIN_EVENT_MONITOR_CONNECT => {
                    DesktopEvent::MonitorConnect(
                        event.__bindgen_anon_1.monitor_connect.monitor.into()
                    )
                }

                ffi::emwin_event_type_EMWIN_EVENT_MONITOR_DISCONNECT => {
                    DesktopEvent::MonitorDisconnect(
                        event.__bindgen_anon_1.monitor_disconnect.monitor.into()
                    )
                }

                ffi::emwin_event_type_EMWIN_EVENT_WINDOW_CLOSE => {
                    DesktopEvent::WindowClose(
                        event.__bindgen_anon_1.window_close.id.into()
                    )
                }

                ffi::emwin_event_type_EMWIN_EVENT_WINDOW_RESIZE => {
                    let size = event.__bindgen_anon_1.window_resize.size;

                    DesktopEvent::WindowResize(
                        event.__bindgen_anon_1.window_resize.id.into(),
                        [size.x, size.y],
                    )
                }

                ffi::emwin_event_type_EMWIN_EVENT_WINDOW_FOCUS_GAINED => {
                    DesktopEvent::WindowFocusGained(
                        event.__bindgen_anon_1.window_focus_gained.id.into()
                    )
                }

                ffi::emwin_event_type_EMWIN_EVENT_WINDOW_FOCUS_LOST => {
                    DesktopEvent::WindowFocusLost(
                        event.__bindgen_anon_1.window_focus_lost.id.into()
                    )
                }

                ffi::emwin_event_type_EMWIN_EVENT_INPUT_KEY_ACTION => {
                    let data = event.__bindgen_anon_1.input_key_action;

                    DesktopEvent::InputKeyAction(
                        data.id.into(),
                        data.key.into(),
                        data.pressed,
                    )
                }

                ffi::emwin_event_type_EMWIN_EVENT_INPUT_MOUSE_MOTION => {
                    let data = event.__bindgen_anon_1.mouse_motion;

                    DesktopEvent::InputMouseMotion(
                        data.id.into(),
                        [data.delta_pos.x as f64, data.delta_pos.y as f64],
                    )
                }

                ffi::emwin_event_type_EMWIN_EVENT_INPUT_MOUSE_BUTTON_ACTION => {
                    let data = event.__bindgen_anon_1.button_action;

                    DesktopEvent::InputMouseButtonAction(
                        data.id.into(),
                        data.button.into(),
                        data.pressed,
                    )
                }

                ffi::emwin_event_type_EMWIN_EVENT_INPUT_MOUSE_WHEEL => {
                    let data = event.__bindgen_anon_1.mouse_wheel;

                    DesktopEvent::InputMouseWheel(
                        data.id.into(),
                        [data.delta_scroll.x as f64, data.delta_scroll.y as f64],
                    )
                }

                ffi::emwin_event_type_EMWIN_EVENT_JOYSTICK_CONNECT => {
                    DesktopEvent::JoystickConnect(
                        event.__bindgen_anon_1.joystick_connect.id.into()
                    )
                }

                ffi::emwin_event_type_EMWIN_EVENT_JOYSTICK_DISCONNECT => {
                    DesktopEvent::JoystickDisconnect(
                        event.__bindgen_anon_1.joystick_disconnect.id.into()
                    )
                }

                _ => unreachable!("invalid Ember window event type: {}", event.type_),
            }
        })
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
    pub fn open(allocator: &Allocator, config: &WindowConfig) -> Result<(Window, Desktop)> {
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

    pub fn close(&mut self, allocator: &Allocator) {
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

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MouseCode {
    Left,
    Right,
    Middle,
}

impl From<u32> for KeyCode {
    fn from(value: u32) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl From<u32> for MouseCode {
    fn from(value: u32) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}
