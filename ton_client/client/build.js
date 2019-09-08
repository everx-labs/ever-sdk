const fs = require('fs');
const os = require('os');
const {gz, spawnProcess, deleteFolderRecursive, main, version, root_path} = require('../platforms/build-lib');
const platform = os.platform();

main(async() => {
    // await spawnProcess('cargo', ['clean']);
    await spawnProcess('cargo', ['update']);
    await spawnProcess('cargo', ['build', '--release']);
    deleteFolderRecursive(root_path('bin'));
    fs.mkdirSync(root_path('bin'), { recursive: true });
    switch(platform) {
        case 'linux':
            await gz(['..', '..', 'target', 'release', 'libton_client.so'], `tonclient_${version}_${platform}`);
            break;
        case 'win32':
            await gz(['..', '..', 'target', 'release', 'ton_client.dll.lib'], `tonclient_${version}_${platform}.lib`);
            await gz(['..', '..', 'target', 'release', 'ton_client.dll'], `tonclient_${version}_${platform}.dll`);
            break;
        case 'darwin':
            await gz(['..', '..', 'target', 'release', 'libton_client.dylib'], `tonclient_${version}_${platform}`);
            break;
        default:
    }
});
