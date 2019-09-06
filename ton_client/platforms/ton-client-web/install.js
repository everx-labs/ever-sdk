const os = require('os');
const path = require('path');
const exec = require('util').promisify(require('child_process').exec);
const env = {...process.env};
let cargoWarn = false;

async function checkCargo() {
    console.log('Installing cargo...');
    const cmd = 'cargo --version || ' + (os.platform() !== 'win32' ? 
        '(curl -f -L https://static.rust-lang.org/rustup.sh -O && sh rustup.sh -y)' : 
        'powershell Invoke-WebRequest -OutFile rustup-init.exe ' + 
        'https://static.rust-lang.org/rustup/dist/i686-pc-windows-gnu/rustup-init.exe && rustup-init.exe');
    try {
        const { stdout, stderr } = await exec(cmd, { env: env });
        if(stdout) {
            console.log(stdout);
            if(env['PATH'].indexOf(path.join(env['HOME'],'/.cargo/bin')) < 0) {
                env['PATH'] =`${path.join(env['HOME'],'/.cargo/bin')}:${env['PATH']}`;
                cargoWarn = true;
            }
        } else {
            console.log(stderr);
            process.exit(1);
        }
    } catch(err) {
        console.log(err);
        process.exit(1);
    }
}

async function checkWasmPack() {
    console.log('Installing wasm-pack...');
    const cmd = 'wasm-pack --version || cargo install wasm-pack';
    try {
        const { stdout, stderr } = await exec(cmd, { env: env });
        if(stdout) {
            console.log(stdout);
        } else {
            let pos = stderr.indexOf('Installed package `wasm-pack');
            if(pos < 0){
                console.log(stderr);
                process.exit(1);
            }
            console.log(stderr.substring(pos));
        }
    } catch(err) {
        console.log(err);
        process.exit(1);
    }
}

(async () => {
    await checkCargo();
    await checkWasmPack();
    if(cargoWarn && os.platform()!=='win32') {
        console.log('Do not forget to run\nsource $HOME/.cargo/env\nin your shell to configure your current shell.')
    }
})();