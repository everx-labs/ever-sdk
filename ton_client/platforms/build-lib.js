const fs = require('fs');
const path = require('path');
const zlib = require('zlib');
const { spawn } = require('child_process');

const root = process.cwd();
const spawnEnv = {
    ...process.env,
};

function root_path(...items) {
    return path.join(root, ...items);
}

const ton_client_toml = fs.readFileSync(path.join(__dirname, '..', 'client', 'Cargo.toml')).toString();
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

        spawned.on('close', (code) => {
            if (code === 0) {
                resolve();
            } else {
                reject();
            }
        });
    });
}

function gz(src, dst) {
    return new Promise((resolve, reject) => {
        const src_path = root_path(...src);
        const dst_path = root_path('bin', dst);
        fs.createReadStream(src_path)
            .pipe(zlib.createGzip({ level: 9 }))
            .pipe(fs.createWriteStream(dst_path + '.gz'))
            .on('finish', () => {
                fs.chmodSync(dst_path + '.gz', 0o666);
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
    deleteFolderRecursive,
    main,
    gz,
    version,
    root_path,
};
