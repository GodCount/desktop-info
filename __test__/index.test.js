
const { getDesktopWindowInfo } = require("../index");

describe("桌面窗口信息获取", () => {
  test("获取桌面显示的窗口信息", () => {
    const desktopWindowInfo = getDesktopWindowInfo(process.ppid);
    expect(desktopWindowInfo).not.toBeNull();
  });
});
