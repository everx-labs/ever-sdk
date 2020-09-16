const fs = require('fs');
const os = require('os');
const {
    deleteFolderRecursive,
    spawnProcess,
    root_path,
    main,
    gz,
    devMode,
    version,
} = require('../build-lib');

const platform = require('os').platform();

function dylibext() {
    return { win32: 'dll', darwin: 'dylib' }[platform] || 'so';
}

const dev = {
    lib: 'libtonclient.a',
    dylib: `libtonclient.${dylibext()}`,
    addon: 'tonclient.node',
};

// const release = JSON.parse(JSON.stringify(dev));

const config = dev;

async function buildNodeJsAddon() {
    deleteFolderRecursive(root_path('bin'));
    // build sdk release
    // await spawnProcess('cargo', ['clean']);
    // if (!devMode) {
    //     await spawnProcess('cargo', ['update']);
    // }
    await spawnProcess('cargo', ['build', '--release']);
    // build addon
    if (os.platform() !== 'win32') {
        await spawnProcess('npm', ['run', 'build']);
    } else {
        await spawnProcess('cmd', ['/c', 'node-gyp', 'rebuild']);
    }
    // collect files
    let dir = root_path('bin');
    fs.mkdirSync(dir);

    await gz(['build', 'Release', config.addon], `tonclient_${version}_nodejs_addon_${platform}`);
    if (platform === 'darwin') {
        await gz(
            ['target', 'release', config.dylib],
            `tonclient_${version}_nodejs_dylib_${platform}`,
            ['libtonclientnodejs.dylib'], // TODO: for backward compatibility. Remove this on 1.0.0
        );
    }
}


main(async () => {
    await buildNodeJsAddon();
});
