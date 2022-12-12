const nugget = require("nugget");
const rc = require("rc");
const pkg = require('./package.json');
const platform = process.platform;

const nodeFile = platform == "win32" ? "desktop-info.win32-x64-msvc.node" : "desktop-info.darwin-x64.node";

install();
function install() {
    console.log('Downloading prebuild for platform:', nodeFile);
    let downloadUrl =
        'https://github.com/GodCount/desktop-info/releases/download/v' +
        pkg.version + "/" + nodeFile;

    let nuggetOpts = {
        dir: "./",
        target: nodeFile,
        strictSSL: true,
    };

    let npmrc = {};

    try {
        rc('npm', npmrc);
    } catch (error) {
        console.warn('Error reading npm configuration: ' + error.message);
    }

    if (npmrc && npmrc.proxy) {
        nuggetOpts.proxy = npmrc.proxy;
    }

    if (npmrc && npmrc['https-proxy']) {
        nuggetOpts.proxy = npmrc['https-proxy'];
    }

    if (npmrc && npmrc['strict-ssl'] === false) {
        nuggetOpts.strictSSL = false;
    }

    nugget(downloadUrl, nuggetOpts, function (errors) {
        if (errors) {
            throw errors[0];
        }else{
            console.log("download done!!!");
        }
    });
}