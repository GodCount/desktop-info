use serde::{Deserialize, Serialize};
use napi::{Error, Result, Status};
use osascript;

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
pub fn get_desktop_window_info(ppid: i32) -> Result<JsDesktopWindowInfo> {
  match osascript::JavaScript::new(JAVASCRIPT_CODE).execute_with_params(ppid) {
    Ok(window_infos) => Ok(JsDesktopWindowInfo::new(window_infos)),
    Err(err) => Err(Error::new(Status::CallbackScopeMismatch, err.to_string())),
  }
}
