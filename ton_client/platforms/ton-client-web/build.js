const fs = require('fs');
const path = require('path');
const zlib = require('zlib');
const root = __dirname;

const spawnEnv = {
    ...process.env,
};


function spawnProcess(name, args) {
    return new Promise((resolve, reject) => {
        const { spawn } = require('child_process');
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


async function buildWasm() {
    //await spawnProcess('cargo', ['clean']);
    //await spawnProcess('cargo', ['update']);
    await spawnProcess('wasm-pack', ['build', '--release']);
}

function makeBinDir() {
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

    fs.renameSync(path.join(root, 'pkg'), path.join(root, 'bin'));
}

(async () => {
    try {
        await buildWasm();
        makeBinDir();
    } catch (error) {
        console.error(error);
        process.exit(1);
    }
})();
