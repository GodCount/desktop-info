
const { sum } = require("../index");

describe("桌面窗口信息获取", () => {
    test("获取桌面显示的窗口信息", () => {
        const winRects = sum();
        console.log(winRects);
        expect(winRects).not.toBeNull();
    });
});