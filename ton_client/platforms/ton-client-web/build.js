const fs = require('fs');
const {gz, spawnProcess, deleteFolderRecursive, main, version, root_path} = require('../build-lib');


main(async() => {
    // await spawnProcess('cargo', ['clean']);
    await spawnProcess('cargo', ['update']);
    await spawnProcess('wasm-pack', ['build', '--release']);
    deleteFolderRecursive(root_path('bin'));
    fs.mkdirSync(root_path('bin'), { recursive: true });
    await gz(['pkg', 'ton_client_web_bg.wasm'], `tonclient_${version}_wasm`);
    await gz(['pkg', 'ton_client_web.js'], `tonclient_${version}_wasm_js`);
});
