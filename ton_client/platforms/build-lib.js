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

const ton_client_toml = fs.readFileSync(path.join(__dirname, '..', 'client', 'Cargo.toml'))
    .toString();
const toml_version = /^\s*version\s*=\s*"([0-9.]+)"\s*$/gm.exec(ton_client_toml)[1] || '';
const version = toml_version.split('.').join('_');


function spawnProcess(name, args) {
    return new Promise((resolve, reject) => {
        const spawned = spawn(name, args, { env: spawnEnv });

        spawned.stdout.on('data', function (data) {
            process.stdout.write(data);
        });

        spawned.stderr.on('data', (data) => {
            process.stderr.write(data);
        });

        spawned.on('error', (err) => {
            reject(err);
        });

        spawned.on('close', (code) => {
            if (code === 0) {
                resolve();
            } else {
                reject();
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

function gz(src, dst, devPath) {
    return new Promise((resolve, reject) => {
        const srcPath = root_path(src);
        const dstPath = root_path('bin', dst);

        if (devOut) {
            let dstDevPath = appendFileNameIfMissing(
                devPath ? path.resolve(devOut, ...devPath) : devOut,
                src[src.length - 1],
            );
            mkdir(path.dirname(dstDevPath))
            fs.copyFileSync(srcPath, dstDevPath);
        }

        fs.createReadStream(srcPath)
            .pipe(zlib.createGzip({ level: 9 }))
            .pipe(fs.createWriteStream(dstPath + '.gz'))
            .on('finish', () => {
                fs.chmodSync(dstPath + '.gz', 0o666);
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
    gz,
    toml_version,
    version,
    root_path,
    devOut,
    devMode,
    mkdir,
};
