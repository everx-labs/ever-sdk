const fs = require('fs');
const path = require('path');
const {
    gz,
    spawnProcess,
    deleteFolderRecursive,
    main,
    toml_version,
    version,
    root_path,
    writeBuildInfo,
    buildNumber,
    gitCommit,
    devMode,
    mkdir,
} = require('../build-lib');

function replaceAll(s, replaces) {
    return replaces.reduce((s, r) => s.split(r[0]).join(r[1]), s);
}

function scriptToStringLiteral(s) {
    return `\`${replaceAll(s, [['\\', '\\\\'], ['`', '\\`'], ['$', '\\$']])}\``;
}

function getTemplate(name) {
    const template = fs.readFileSync(path.resolve(__dirname, name), 'utf-8').split('//---');
    if (template.length > 1) {
        template.shift();
    }
    return template.join('');
}

function getWasmWrapperScript() {
    let script = fs.readFileSync(path.resolve(__dirname, 'pkg', 'tonclient.js'), 'utf-8');
    script = script.replace(/^export function /gm, 'function ');
    script = script.replace(/^export default init;$/gm, '');
    script = script.replace(/^\s*input = import\.meta.*$/gm, '');
    script = script.replace(/getObject\(arg0\) instanceof Window/gm, 'true');
    return script;
}

function getWorkerScript() {
    return [
        getWasmWrapperScript(),
        getTemplate('build-worker.js'),
    ].join('\n');
}

function getIndexScript() {
    const workerScript = getWorkerScript();
    const script = [
        `const workerScript = ${scriptToStringLiteral(workerScript)};`,
        getTemplate('build-index.js').replace('__VERSION__', toml_version),
    ];
    return script.join('\n');
}


main(async () => {
    if (!devMode) {
        // await spawnProcess('cargo', ['clean']);
        await spawnProcess('cargo', ['update']);
    }
    await writeBuildInfo(root_path('..', '..', 'client', 'src', 'build_info.json'), buildNumber, gitCommit);
    await spawnProcess('cargo', ['install', 'wasm-pack', '--version', '0.9.1']);
    await spawnProcess('wasm-pack', ['build', '--release', '--target', 'web']);

    mkdir(root_path('build'));
    fs.copyFileSync(root_path('pkg', 'tonclient_bg.wasm'), root_path('build', 'tonclient.wasm'));
    fs.writeFileSync(root_path('build', 'index.js'), getIndexScript(), { encoding: 'utf8' });

    deleteFolderRecursive(root_path('bin'));
    fs.mkdirSync(root_path('bin'), { recursive: true });
    await gz(['build', 'tonclient.wasm'], `tonclient_${version}_wasm`);
    await gz(['build', 'index.js'], `tonclient_${version}_wasm_js`);
});
