#![deny(clippy::all)]

use winapi::shared::windef::{HWND, RECT};
use winapi::um::winnt::PWSTR;
use winapi::um::winuser::{
    GetForegroundWindow, GetWindow, GetWindowInfo, GetWindowTextW, IsIconic, IsWindowVisible,
    WINDOWINFO,
};

#[macro_use]
extern crate napi_derive;

#[napi(object)]
#[derive(Debug, PartialEq)]
pub struct WINRECT {
    pub title: String,
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl WINRECT {
    fn new(rect: RECT, title: String) -> Self {
        WINRECT {
            title,
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        }
    }
    fn and(&self) -> i32 {
        return self.left + self.top + self.right + self.bottom;
    }
}

fn enum_window_hierarchy(window: HWND, prev_next: bool, list: &mut Vec<WINRECT>) {
    unsafe {
        if window.is_null() {
            return;
        }
        let mut window_info = WINDOWINFO {
            cbSize: std::mem::size_of::<WINDOWINFO>() as u32,
            rcWindow: RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            rcClient: RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            dwStyle: 0,
            dwExStyle: 0,
            dwWindowStatus: 0,
            cxWindowBorders: std::mem::size_of::<WINDOWINFO>() as u32,
            cyWindowBorders: std::mem::size_of::<WINDOWINFO>() as u32,
            atomWindowType: 0,
            wCreatorVersion: 0,
        };
        GetWindowInfo(window, &mut window_info);

        let mut text: [u16; 512] = [0; 512];
        let len = GetWindowTextW(window, PWSTR::from(text.as_mut_ptr()), text.len() as i32);
        let text = String::from_utf16_lossy(&text[..len as usize]);

        let win_rect = WINRECT::new(window_info.rcWindow, text);
        if win_rect.and() > 0 && IsWindowVisible(window) != 0 && IsIconic(window) == 0 {
            if prev_next == true {
                list.splice(0..0, [win_rect]);
            } else {
                list.push(win_rect);
            }
        }

        if prev_next == true {
            let prev = GetWindow(window, 3);
            if !prev.is_null() && prev != window {
                enum_window_hierarchy(prev, true, list);
            }
        } else {
            let next = GetWindow(window, 2);
            if !next.is_null() && next != window {
                enum_window_hierarchy(next, false, list);
            }
        }
    }
}

#[napi]
pub fn sum() -> Vec<WINRECT> {
    unsafe {
        let mut list: Vec<WINRECT> = vec![];
        enum_window_hierarchy(GetForegroundWindow(), false, &mut list);
        enum_window_hierarchy(GetWindow(GetForegroundWindow(), 3), true, &mut list);
        list
    }
}
