const desktopInfo = require("./build/release/index");
const desktopInfoDev = require("./build/dev/index");



module.exports = {
    ...desktopInfo,
    dev: {
        ...desktopInfoDev
    }
}