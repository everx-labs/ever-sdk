/*
 * Copyright 2018-2020 TON DEV SOLUTIONS LTD.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific TON DEV software governing permissions and
 * limitations under the License.
 *
 */

const fs = require('fs');
const path = require('path');
const zlib = require('zlib');
const { spawn } = require('child_process');

const root = process.cwd();
const spawnEnv = {
    ...process.env,
};

function root_path(...items) {
    return path.resolve(root, ...(items.reduce((a, x) => a.concat(x), [])));
}

const ton_client_toml = fs.readFileSync(path.join(__dirname, '..', 'ton_client', 'Cargo.toml'))
    .toString();
const toml_version = /^\s*version\s*=\s*['"]([0-9.]+)['"]\s*$/gm.exec(ton_client_toml)[1] || '';
const version = toml_version.split('.').join('_');


function spawnProcess(name, args, options) {
    return new Promise((resolve, reject) => {
        const spawned = spawn(name, args, { env: spawnEnv });
        let res = '';
        let err = '';

        spawned.stdout.on('data', function (data) {
            res += data;
            if (options && options.quiet === true) {
                return;
            }
            process.stdout.write(data);
        });

        spawned.stderr.on('data', (data) => {
            err += data;
            if (options && options.quiet === true) {
                return;
            }
            process.stderr.write(data);
        });

        spawned.on('error', (err) => {
            reject(err);
        });

        spawned.on('close', (code) => {
            if (code === 0) {
                resolve(res);
            } else {
                reject(options && options.quiet ? `return code: ${code}\ncmdline: ${name} ${args.join(' ')}\n` + err : '');
            }
        });
    });
}

async function spawnAll(items, getArgs) {
    const list = [];
    for (const item of items) {
        const args = getArgs(item);
        console.log(`Build: ${args.join(' ')}`);
        list.push(spawnProcess(args[0], args.slice(1)));
    }
    return Promise.all(list);
}


const options = process.argv.slice(2).reduce((acc, key, ind, argv) => {
    if (key.startsWith('-')) {
        const name=key.split(/^--/, 2)[1];
        const [nameVal, val] = name.split(/=(.*)/, 2);
        if (nameVal && val) {
            acc[nameVal] = val;
        } else if (name && argv[ind+1] && argv[ind+1].startsWith('-')) {
            acc[name] = true;
        } else if (name && argv[ind+1]) {
            acc[name] = argv[ind + 1];
        } else if (name && !argv[ind+1]) {
            acc[name] = true;
        }
    }
    return acc;
}, {});
const getOption = opt => options[opt] || '';

const noBuildInfo = getOption('no-build-info');
if (noBuildInfo) {
    spawnEnv['NO_BUILD_INFO'] = true
}
const verboseMode = getOption('verbose');
const devOut = getOption('dev-out');
const devMode = !!devOut;


function mkdir(path) {
    if (!fs.existsSync(path)) {
        fs.mkdirSync(path, { recursive: true });
    }
}

function appendFileNameIfMissing(fileOrDirPath, defaultFileName) {
    return path.extname(fileOrDirPath) !== ''
        ? fileOrDirPath
        : path.resolve(fileOrDirPath, defaultFileName);

}

async function postBuild(target, platform) {
    switch (platform) {
        case 'darwin':
            const libPath = root_path(target);
            const libFileName = path.basename(libPath);
            const libFixedPath = `@loader_path/${libFileName}`;
            if (verboseMode) {
                console.log(`Fix lib:${libPath} using id:${libFixedPath}`);
            }
            await spawnProcess('install_name_tool', ['-id', libFixedPath, libPath]);
    }

    return Promise.resolve();
}

function gz(src, dst, devPath) {
    return new Promise((resolve, reject) => {
        const srcPath = root_path(src);
        const dstPath = root_path('bin', dst) + '.gz';

        if (devOut || devPath) {
            let dstDevPath = appendFileNameIfMissing(
                devPath ? path.resolve(devOut, ...devPath) : devOut,
                src[src.length - 1],
            );
            mkdir(path.dirname(dstDevPath))
            fs.copyFileSync(srcPath, dstDevPath);
        }
        if (verboseMode) {
            console.log(`Gzip src:${srcPath} to dst:${dstPath}`);
        }
        fs.createReadStream(srcPath)
            .pipe(zlib.createGzip({ level: 9 }))
            .pipe(fs.createWriteStream(dstPath))
            .on('finish', () => {
                fs.chmodSync(dstPath, 0o666);
                resolve();
            })
            .on('error', (error) => {
                reject(error);
            });
    });
}


function deleteFolderRecursive(dir) {
    if (fs.existsSync(dir)) {
        const files = fs.readdirSync(dir);
        files.forEach(file => {
            const curPath = path.join(dir, file);
            if (fs.lstatSync(curPath).isDirectory()) {
                deleteFolderRecursive(curPath);
            } else {
                fs.unlinkSync(curPath);
            }
        });
        fs.rmdirSync(dir);
    }
}

function main(f) {
    (async () => {
        try {
            await f();
        } catch (error) {
            console.error(error);
            process.exit(1);
        }
    })();
}

module.exports = {
    spawnEnv,
    spawnProcess,
    spawnAll,
    deleteFolderRecursive,
    main,
    postBuild,
    gz,
    toml_version,
    version,
    root_path,
    devOut,
    devMode,
    mkdir,
};
