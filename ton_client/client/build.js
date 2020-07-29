const fs = require('fs');
const os = require('os');
const path = require('os');
const { gz, spawnProcess, deleteFolderRecursive, main, version, root_path } = require('../platforms/build-lib');
const platform = os.platform();


main(async () => {
    // await spawnProcess('cargo', ['clean']);
    await spawnProcess('cargo', ['update']);
    await spawnProcess('cargo', ['build', '--release']);
    deleteFolderRecursive(root_path('bin'));
    fs.mkdirSync(root_path('bin'), { recursive: true });
    switch (platform) {
    case 'linux':
        await gz(
            ['..', '..', 'target', 'release', 'libtonclient.so'],
            `tonclient_${version}_${platform}`,
            [__dirname, 'build'],
        );
        break;
    case 'win32':
        await gz(
            ['..', '..', 'target', 'release', 'tonclient.dll.lib'],
            `tonclient_${version}_${platform}_lib`,
            [__dirname, 'build'],
        );
        await gz(
            ['..', '..', 'target', 'release', 'tonclient.dll'],
            `tonclient_${version}_${platform}_dll`,
            [__dirname, 'build'],
        );
        break;
    case 'darwin':
        await gz(
            ['..', '..', 'target', 'release', 'libtonclient.dylib'],
            `tonclient_${version}_${platform}`,
            [__dirname, 'build'],
        );
        break;
    default:
    }
});
