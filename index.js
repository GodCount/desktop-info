switch (platform) {
  case "win32":
    try {
      nativeBinding = require('./desktop-info.win32-x64-msvc.node')
    } catch (e) {
      loadError = e
    }
    break;
  case "darwin":
    try {
      nativeBinding = require('./desktop-info.darwin-x64.node')
    } catch (e) {
      loadError = e
    }
    break
}


if (!nativeBinding) {
  if (loadError) {
    throw loadError
  }
  throw new Error(`Failed to load native binding`)
}

const { DesktopWindowInfo, getDesktopWindowInfo } = nativeBinding

module.exports.DesktopWindowInfo = DesktopWindowInfo
module.exports.getDesktopWindowInfo = getDesktopWindowInfo
