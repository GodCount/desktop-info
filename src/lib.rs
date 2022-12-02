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
#[derive(Debug, Default, Clone, PartialEq)]
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
  fn size(&self) -> (i32, i32) {
    let width = self.right - self.left;
    let height = self.bottom - self.top;
    (width, height)
  }
}

#[napi(js_name = "DesktopWindowInfo")]
pub struct JsDesktopWindowInfo {
  pub win_rects: Vec<WINRECT>,
}

#[napi]
impl JsDesktopWindowInfo {
  #[napi(constructor)]
  pub fn new(win_rects: Vec<WINRECT>) -> Self {
    JsDesktopWindowInfo { win_rects }
  }
  #[napi]
  pub fn is_overlaps(&self, x: i32, y: i32) -> Option<WINRECT> {
    for i in 0..self.win_rects.len() {
      let win: &WINRECT = &self.win_rects[i];
      if win.left <= x && win.top <= y && win.right >= x && win.bottom >= y {
        return Some(win.clone());
      }
    }
    None
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
    let (width, height) = win_rect.size();
    if width > 5 && height > 5 && IsWindowVisible(window) != 0 && IsIconic(window) == 0 {
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

fn filter_impurities(list: &mut Vec<WINRECT>) {
  for i in (list.len() - 1)..0 {
    let win = &list[i];
    for j in (i - 1)..0 {
      let up_win = &list[j];
      if (win.left - up_win.left) >= 0
        && (win.top - up_win.top) >= 0
        && win.right <= up_win.right
        && win.bottom <= up_win.bottom
      {
        list.remove(i);
        break;
      }
    }
  }
}

#[napi]
pub fn get_desktop_window_info() -> JsDesktopWindowInfo {
  unsafe {
    let mut list: Vec<WINRECT> = vec![];
    enum_window_hierarchy(GetForegroundWindow(), false, &mut list);
    enum_window_hierarchy(GetWindow(GetForegroundWindow(), 3), true, &mut list);
    println!("前：{:?}", list);
    filter_impurities(&mut list);
    // println!("后：{:?}", list);
    JsDesktopWindowInfo::new(list)
  }
}
