const fs = require('fs');
const path = require('path');
const os = require('os');
const {
    deleteFolderRecursive,
    spawnProcess,
    root_path,
    main,
    gz,
    version
} = require('../build-lib');

const platform = require('os').platform();

function dylibext() {
    switch (platform) {
    case "win32":
        return "dll";
    case "darwin":
        return "dylib";
    default:
        return "so";
    }
}

const dev = {
    lib: 'libtonclientnodejs.a',
    dylib: `libtonclientnodejs.${dylibext()}`,
    addon: 'tonclient.node',
};

// const release = JSON.parse(JSON.stringify(dev));

const config = dev;

function getOption(option) {
    const prefixes = [];
    ['--', '-'].forEach(pfx => [':', '='].forEach(sfx => prefixes.push(`${pfx}${option}${sfx}`)));
    for (const arg of process.argv) {
        for (const pfx of prefixes) {
            if (arg.startsWith(pfx)) {
                return arg.slice(pfx.length);
            }
        }
    }
    return '';
}

async function buildNodeJsAddon() {
    deleteFolderRecursive(root_path('bin'));
    // build sdk release
    // await spawnProcess('cargo', ['clean']);
    await spawnProcess('cargo', ['update']);
    await spawnProcess('cargo', ['build', '--release']);
    // build addon
    if (os.platform() !== "win32") {
        await spawnProcess('npm', ['run', 'build']);
    } else {
        await spawnProcess('cmd', ['/c', 'node-gyp', 'rebuild']);
    }
    // collect files
    let dir = root_path('bin');
    fs.mkdirSync(dir);

    const devOut = getOption('dev-out');
    if (devOut) {
        fs.copyFileSync(
            root_path('build', 'Release', config.addon),
            path.resolve(devOut, config.addon),
        );
    }

    await gz(['build', 'Release', config.addon], `tonclient_${version}_nodejs_addon_${platform}`);
    if (platform === 'darwin') {
        await gz(['target', 'release', config.dylib], `tonclient_${version}_nodejs_dylib_${platform}`);
        if (devOut) {
            fs.copyFileSync(
                root_path('target', 'release', config.dylib),
                path.resolve(devOut, config.dylib),
            );
        }
    }
}


main(async () => {
    await buildNodeJsAddon();
});
