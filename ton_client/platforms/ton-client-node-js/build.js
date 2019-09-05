const fs = require('fs');
const path = require('path');
const os = require('os');
const root = __dirname;

function dylibext() {
    switch (require('os').platform()) {
    case "win32":
        return "dll";
    case "darwin":
        return "dylib";
    default:
        return "so";
    }
}

const dev = {
    lib: 'libtonsdk.a',
    dylib: `libtonsdk.${dylibext()}`,
    addon: 'tonclient.node',
};
// const release = JSON.parse(JSON.stringify(dev));

const config = dev;
const sdkDir = root;
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
    await spawnProcess('cargo', ['clean']);
    await spawnProcess('cargo', ['update']);
    await spawnProcess('cargo', ['build', '--release']);
    // build addon
    if (os.platform() !== "win32") {
        await spawnProcess('node-gyp', ['rebuild']);
    } else {
        await spawnProcess('cmd', ['/c', 'node-gyp', 'rebuild']);
    }
    // collect files
    let dir = path.join(root, 'bin'); 
    fs.mkdirSync(dir);
    let src = path.join(root, 'build', 'Release', config.addon);
    let dst = path.join(root, 'bin', config.addon);
    fs.copyFileSync(src, dst);
    if(os.platform() === 'darwin') {
        src = path.join(root, 'target', 'release', config.dylib);
        dst = path.join(root, 'bin', config.dylib);
        fs.copyFileSync(src, dst);
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
