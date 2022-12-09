use winapi::ctypes::c_void;
use winapi::shared::ntdef::PWSTR;
use winapi::shared::windef::{HWND, RECT};
use winapi::um::dwmapi::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use winapi::um::winuser::{
  GetForegroundWindow, GetWindow, GetWindowTextW, IsIconic, IsWindowVisible,
};

#[napi(object)]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct WindowBounds {
  pub title: String,
  pub x: i32,
  pub y: i32,
  pub width: i32,
  pub height: i32,
}

impl WindowBounds {
  fn new(rect: RECT, title: String) -> Self {
    let (width, height) = (rect.right - rect.left, rect.bottom - rect.top);
    WindowBounds {
      title,
      x: rect.left,
      y: rect.top,
      width,
      height,
    }
  }
}

#[napi(js_name = "DesktopWindowInfo")]
pub struct JsDesktopWindowInfo {
  pub win_rects: Vec<WindowBounds>,
}

#[napi]
impl JsDesktopWindowInfo {
  #[napi(constructor)]
  pub fn new(win_rects: Vec<WindowBounds>) -> Self {
    JsDesktopWindowInfo { win_rects }
  }
  #[napi]
  pub fn is_overlaps(&self, x: i32, y: i32) -> Option<WindowBounds> {
    for i in 0..self.win_rects.len() {
      let win: &WindowBounds = &self.win_rects[i];
      if win.x <= x && win.y <= y && (win.x + win.width) >= x && (win.y + win.height) >= y {
        return Some(win.clone());
      }
    }
    None
  }
}

fn enum_window_hierarchy(window: HWND, prev_next: bool, list: &mut Vec<WindowBounds>) {
  unsafe {
    if window.is_null() {
      return;
    }

    let mut rect: RECT = RECT {
      left: 0,
      top: 0,
      right: 0,
      bottom: 0,
    };
    let rect_ptr: *mut c_void = &mut rect as *mut _ as *mut c_void;
    DwmGetWindowAttribute(window, DWMWA_EXTENDED_FRAME_BOUNDS, rect_ptr, 16);

    let mut text: [u16; 512] = [0; 512];
    let len = GetWindowTextW(window, PWSTR::from(text.as_mut_ptr()), text.len() as i32);
    let text = String::from_utf16_lossy(&text[..len as usize]);

    let win_rect = WindowBounds::new(rect, text);
    if win_rect.width > 5
      && win_rect.height > 5
      && IsWindowVisible(window) != 0
      && IsIconic(window) == 0
    {
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

fn filter_impurities(list: &mut Vec<WindowBounds>) {
  for i in (list.len() - 1)..0 {
    let win = &list[i];
    for j in (i - 1)..0 {
      let up_win = &list[j];
      if (win.x - up_win.x) >= 0
        && (win.y - up_win.y) >= 0
        && (win.x + win.width) <= (up_win.x + up_win.width)
        && (win.y + win.height) <= (up_win.y + up_win.height)
      {
        list.remove(i);
        break;
      }
    }
  }
}

#[napi]
pub fn get_desktop_window_info(_ppid: i32) -> JsDesktopWindowInfo {
  unsafe {
    let mut list: Vec<WindowBounds> = vec![];
    enum_window_hierarchy(GetWindow(GetForegroundWindow(), 2), false, &mut list);
    // enum_window_hierarchy(GetWindow(GetForegroundWindow(), 3), true, &mut list);
    filter_impurities(&mut list);
    JsDesktopWindowInfo::new(list)
  }
}
