const fs = require('fs');
const path = require('path');
const os = require('os');
const zlib = require('zlib');
const root = __dirname;

const platform = require('os').platform();
const version = JSON.parse(
    fs.readFileSync(
        path.join(root,'package.json')
    )
).version.split('.').join('_');

function dylibext() {
    switch (platform) {
    case "win32":
        return "dll";
    case "darwin":
        return "dylib";
    default:
        return "so";
    }
}

const dev = {
    lib: 'libtonclientnodejs.a',
    dylib: `libtonclientnodejs.${dylibext()}`,
    addon: 'tonclient.node',
};
// const release = JSON.parse(JSON.stringify(dev));

const config = dev;
const spawnEnv = {
    ...process.env,
};


function spawnProcess(name, args) {
    return new Promise((resolve, reject) => {
        const {spawn} = require('child_process');
        const spawned = spawn(name, args, {env: spawnEnv});

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


function gz(src, dst) {
    const src_path = path.join(root, ...src);
    const dst_path = path.join(root, 'bin', dst);
    fs.createReadStream(src_path)
    .pipe(zlib.createGzip({ level: 9 }))
    .pipe(fs.createWriteStream(dst_path + '.gz'));
    fs.chmodSync(dst_path + '.gz', 0o666);
}

async function buildNodeJsAddon() {
    // clean up and restore environment
    const deleteFolderRecursive = (dir) => {
        if( fs.existsSync(dir) ) {
            const files = fs.readdirSync(dir);
            files.forEach(file => {
                const curPath = path.join(dir, file);
                if(fs.lstatSync(curPath).isDirectory()) {
                    deleteFolderRecursive(curPath);
                } else {
                    fs.unlinkSync(curPath);
                }
            });
            fs.rmdirSync(dir);
        }
    };
    deleteFolderRecursive(path.join(root, 'bin'));
    // build sdk release
    // await spawnProcess('cargo', ['clean']);
    await spawnProcess('cargo', ['update']);
    await spawnProcess('cargo', ['build', '--release']);
    // build addon
    if (os.platform() !== "win32") {
        await spawnProcess('npm', ['run', 'build']);
    } else {
        await spawnProcess('cmd', ['/c', 'node-gyp', 'rebuild']);
    }
    // collect files
    let dir = path.join(root, 'bin');
    fs.mkdirSync(dir);
    gz(['build', 'Release', config.addon], `tonclient_${version}_nodejs_addon_${platform}`);
    if(platform === 'darwin') {
        gz(['target', 'release', config.dylib], `tonclient_${version}_nodejs_dylib_${platform}`);
    }
}


(async () => {
    try {
        await buildNodeJsAddon();
    } catch (error) {
        console.error(error);
        process.exit(1);
    }
})();
