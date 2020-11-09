const fs = require('fs');
const os = require('os');
const {
    gz,
    spawnProcess,
    deleteFolderRecursive,
    main,
    version,
    root_path,
    postBuild,
    writeBuildInfo,
    buildNumber,
    gitCommit,
    devMode,
} = require('../platforms/build-lib');
const platform = os.platform();

main(async () => {
    if (!devMode) {
        // await spawnProcess('cargo', ['clean']);
        await spawnProcess('cargo', ['update']);
    }
    await writeBuildInfo(root_path('src', 'build_info.json'), buildNumber, gitCommit);
    await spawnProcess('cargo', ['build', '--release']);
    deleteFolderRecursive(root_path('bin'));
    fs.mkdirSync(root_path('bin'), { recursive: true });
    const platformNames = {
        linux: [['lib{}.so', '']],
        win32: [['{}.dll.lib', '_lib'], ['{}.dll', '_dll']],
        darwin: [['lib{}.dylib', '']],
    };
    for (const [src, dstSuffix] of platformNames[platform] || []) {
        const target = ['..', '..', 'target', 'release', src.replace('{}', 'ton_client')];
        await postBuild(target, platform);
        await gz(
            target,
            `tonclient_${version}_${platform}${dstSuffix || ''}`, [__dirname, 'build']);
    }
});
