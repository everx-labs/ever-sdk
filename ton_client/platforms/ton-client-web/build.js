const fs = require('fs');
const path = require('path');
const {
    gz,
    spawnProcess,
    deleteFolderRecursive,
    main,
    toml_version,
    version,
    root_path
} = require('../build-lib');

function scriptToStringLiteral(s) {
    return `\`${s.split('`').join('\\``')}\``;
}

function getTemplate(name) {
    const template = fs.readFileSync(path.resolve(__dirname, name), 'utf-8').split('//---');
    if (template.length > 1) {
        template.shift();
    }
    return template.join('');
}

function getWasmWrapperScript() {
    let script = fs.readFileSync(path.resolve(__dirname, 'pkg', 'ton_client_web.js'), 'utf-8');
    script = script.replace(
        /^let wasm;$/gm,
        `
const wasmWrapper = (function() {
let wasm = null;
const result = {
    setup: (newWasm) => {
        wasm = newWasm;
    },
};
`,
    );
    script = script.replace(/^export const /gm, 'result.');
    script = script.replace(/^export function (\w+)/gm, 'result.$1 = function');
    script = script.replace(/^async function load\([^]*?^}$/gm, '');
    script = script.replace(/^async function init\([^]*?^\s*const imports = {};$/gm, '');
    script = script.replace(/^\s*if \(typeof input === [^]*/gm, '');
    script = script.replace(/^\s*imports\.wbg/gm, '    result.wbg');
    script +=
        `   return result;
})()`;
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
        `import { TONClient } from 'ton-client-js';`,
        `const workerScript = ${scriptToStringLiteral(workerScript)};`,
        getTemplate('build-index.js').replace('__VERSION__', toml_version),
    ];
    return script.join('\n');
}


main(async () => {
    // await spawnProcess('cargo', ['clean']);
    // await spawnProcess('cargo', ['update']);
    await spawnProcess('wasm-pack', ['build', '--release', '--target', 'web']);
    fs.writeFileSync(path.resolve(__dirname, 'pkg', 'index.js'), getIndexScript(), { encoding: 'utf8' });
    deleteFolderRecursive(root_path('bin'));
    fs.mkdirSync(root_path('bin'), { recursive: true });
    await gz(['pkg', 'ton_client_web_bg.wasm'], `tonclient_${version}_wasm`);
    await gz(['pkg', 'index.js'], `tonclient_${version}_wasm_js`);
});
