const fs = require('fs');
const os = require('os');
const {
    deleteFolderRecursive,
    spawnProcess,
    root_path,
    main,
    gz,
    writeBuildInfo,
    buildNumber,
    gitCommit,
    devMode,
    version,
} = require('../build-lib');

const platform = require('os').platform();

async function buildNodeJsAddon() {
    deleteFolderRecursive(root_path('bin'));
    // build sdk release
    if (!devMode) {
        // await spawnProcess('cargo', ['clean']);
        await spawnProcess('cargo', ['update']);
    }
    await writeBuildInfo(root_path('..', '..', 'client', 'src', 'build_info.json'), buildNumber, gitCommit);
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

    await gz(['build', 'Release', 'tonclient.node'], `tonclient_${version}_nodejs_addon_${platform}`);
}


main(async () => {
    await buildNodeJsAddon();
});
