const fs = require("fs");
const path = require("path");

const nodeFilename = process.platform == "win32" ? "desktop-info.win32-x64-msvc.node" : "desktop-info.darwin-x64.node";

main();
function main() {
    const mark = process.argv[2];
    const target = path.join("./", nodeFilename);
    let output;
    switch (mark) {
        case "build":
            output = path.join("./build/release", nodeFilename);
            if (fs.existsSync(output)) fs.unlinkSync(output);
            if (fs.existsSync(target)) fs.renameSync(target, output);
            break;
        case "build:debug":
            output = path.join("./build/dev", nodeFilename);
            if (fs.existsSync(output)) fs.unlinkSync(output);
            if (fs.existsSync(target)) fs.renameSync(target, output);
            break;
    }
}