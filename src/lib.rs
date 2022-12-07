#![deny(clippy::all)]
#![allow(unused_assignments)]

#[macro_use]
extern crate napi_derive;

#[cfg(target_os = "windows")]
pub mod desktop {

  #![allow(non_snake_case, dead_code)]

  use winapi::shared::windef::{HWND, RECT};
  use winapi::um::winnt::PWSTR;
  use winapi::um::winuser::{
    GetForegroundWindow, GetWindow, GetWindowInfo, GetWindowTextW, IsIconic, IsWindowVisible,
    WINDOWINFO,
  };

  #[napi(object)]
  #[derive(Debug, Default, Clone, PartialEq)]
  pub struct WindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
  }

  impl WindowBounds {
    fn new(rect: RECT) -> Self {
      let (width, height) = (self.right - self.left, self.bottom - self.top);
      WindowBounds {
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

      let win_rect = WindowBounds::new(window_info.rcWindow);
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
  pub fn get_desktop_window_info(_: i32) -> JsDesktopWindowInfo {
    unsafe {
      let mut list: Vec<WindowBounds> = vec![];
      enum_window_hierarchy(GetWindow(GetForegroundWindow(), 2), false, &mut list);
      enum_window_hierarchy(GetWindow(GetForegroundWindow(), 3), true, &mut list);
      filter_impurities(&mut list);
      JsDesktopWindowInfo::new(list)
    }
  }
}

#[cfg(target_os = "macos")]
pub mod desktop {
  #![allow(non_upper_case_globals, dead_code)]

  extern crate osascript;
  use serde::{Deserialize, Serialize};

  use napi::{Error, Result, Status};

  static JAVASCRIPT_CODE: &'static str = "
  ObjC.import('CoreGraphics');
  ObjC.import('Quartz');
  nil = $();
  $.unwrap = ObjC.deepUnwrap.bind(ObjC);
  $.bind = ObjC.bindFunction.bind($);
  $.bind('CFMakeCollectable', ['id', ['void *']]);
  Ref.prototype._nsObject = function () { return $.unwrap($.CFMakeCollectable(this)); }

  function dispose(win) {
      return {
          window_layer: win.kCGWindowLayer,
          window_memory_usage: win.kCGWindowMemoryUsage,
          window_is_onscreen: win.kCGWindowIsOnscreen || false,
          window_sharing_state: win.kCGWindowSharingState,
          window_owner_pid: win.kCGWindowOwnerPID,
          window_number: win.kCGWindowNumber,
          window_owner_name: win.kCGWindowOwnerName,
          window_name: win.kCGWindowName || '',
          window_store_type: win.kCGWindowStoreType,
          window_bounds: {
              x: win.kCGWindowBounds.X,
              height: win.kCGWindowBounds.Height,
              y: win.kCGWindowBounds.Y,
              width: win.kCGWindowBounds.Width
          },
          window_alpha: win.kCGWindowAlpha
      }
  }

  function enumDesktopWindow(option, relativeToWindow, pid, displayRect) {
      const CGWindowList = $.CGWindowListCopyWindowInfo(option | $.kCGWindowListExcludeDesktopElements, relativeToWindow)._nsObject();
      const windowList = [];
      const asDesktopWindow = [];
      for (const win of CGWindowList) {
          if (!win.kCGWindowIsOnscreen || win.kCGWindowLayer < 0 || win.kCGWindowOwnerPID == pid) continue;
          const bounds = win.kCGWindowBounds;
          if (bounds.X == 0 && bounds.Y == 0 && bounds.Width == displayRect.width && bounds.Height == displayRect.height) {
              asDesktopWindow.push(dispose(win).window_bounds);
          } else {
              windowList.push(dispose(win).window_bounds);
          }
      }
      windowList.push(...asDesktopWindow);
      return windowList;
  }

  function getDisplayRect() {
      const displayMode = $.CGDisplayCopyDisplayMode($.CGMainDisplayID());
      const rect = {
          width: $.CGDisplayModeGetWidth(displayMode),
          height: $.CGDisplayModeGetHeight(displayMode)
      }
      $.CGDisplayModeRelease(displayMode);
      return rect;
  }

  function main(pid) {
      const cureent = $.CGWindowListCopyWindowInfo($.kCGWindowListOptionAll, 0)._nsObject();
      let relativeToWindow = 0;
      let maxWindowLayer = 0;
      const windowList = [];
      for (const win of cureent) {
          if (win.kCGWindowOwnerPID == pid && win.kCGWindowLayer > maxWindowLayer) {
              maxWindowLayer = win.kCGWindowLayer;
              relativeToWindow = win.kCGWindowNumber;
          }
      }
      // windowList.unshift(...enumDesktopWindow($.kCGWindowListOptionOnScreenAboveWindow, relativeToWindow, pid));
      windowList.push(...enumDesktopWindow($.kCGWindowListOptionOnScreenBelowWindow, relativeToWindow, pid, getDisplayRect()));
      return windowList;
  }

  return main($params);
";

  #[napi(object)]
  #[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
  pub struct WindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
  }

  impl WindowBounds {
    fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
      WindowBounds {
        x,
        y,
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

  // #[derive(Debug, Serialize, Deserialize)]
  // struct WindowInfo {
  //   window_layer: i32,
  //   window_memory_usage: i32,
  //   window_is_onscreen: bool,
  //   window_sharing_state: i32,
  //   window_owner_pid: i32,
  //   window_number: i32,
  //   window_name: String,
  //   window_owner_name: String,
  //   window_store_type: i32,
  //   window_bounds: WindowBounds,
  //   window_alpha: f32,
  // }

  #[napi]
  pub fn get_desktop_window_info(pid: i32) -> Result<JsDesktopWindowInfo> {
    match osascript::JavaScript::new(JAVASCRIPT_CODE).execute_with_params(pid) {
      Ok(window_infos) => Ok(JsDesktopWindowInfo::new(window_infos)),
      Err(err) => Err(Error::new(Status::CallbackScopeMismatch, err.to_string())),
    }
  }
}
