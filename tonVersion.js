const fs = require('fs');
const os = require('os');
const path = require('path');
const https = require('https');

function canonical(versionString) {
    if(!versionString) {
        return [];
    }
    verArray = versionString.split('.');
    for(let i=0; i < verArray.length; i++) {
        const part = Number.parseInt(verArray[i]);
        if(!Number.isInteger(part)) {
            throw new Error(`Invalid version part "${verArray[i]}" in version string "${versionString}"`);
        } else {
            verArray[i] = part;
        }
    }
    if(verArray.length < 2) {
        throw new Error(`Invalid version format "${versionString}". Expected [X]*.[Y]*.[N]*`);
    }
    if(verArray.length === 2) {
        verArray.push(1);
    }
    return verArray;
};
function increment(canonicalVersion) {
    canonicalVersion[2]++;
    return canonicalVersion
};
function setVersion(dest, version) {
    const packagePath = path.join(dest, 'package.json');
    const cargoPath = path.join(dest, 'Cargo.toml');
    if(fs.existsSync(packagePath)) {
        const conf = JSON.parse(fs.readFileSync(packagePath).toString());
        conf.version = version;
        fs.writeFileSync(
            packagePath,
            JSON.stringify(conf, null, ' ')
        );
    }
    if(fs.existsSync(cargoPath)) {
        const conf = fs.readFileSync(cargoPath).toString().split(os.EOL);
        let pos = conf.indexOf('[package]') + 1;
        while(conf[pos] !== '') {
            if(conf[pos].startsWith('version = ')) {
                conf[pos] = `version = "${version}"`
                break;
            }
            pos++;
        }
        fs.writeFileSync(cargoPath, conf.join(os.EOL));
    }
};

const cwd = process.cwd();
let args = process.argv.slice(2);
const folders = []
const lastVersion = JSON.parse(fs.readFileSync(path.join(cwd,'version.json')));
let argVersion;

while(args.length > 0) {
    switch(args[0].trim()) {
    case '--set':
        if(args.length < 2) {
            throw new Error('Version value wasn\'t been presented');
        }
        argVersion = args[1].trim();
        args = args.slice(1);
        break;
    case '--release':
        if(lastVersion.candidate) {
            lastVersion.release = lastVersion.candidate;
            lastVersion.candidate = '';
            fs.writeFileSync(path.join(cwd, 'version.json'), JSON.stringify(lastVersion));
            process.exit(0);
        } else {
            throw new Error('Unable to set candidate as release');
        }
    case '--decline':
            lastVersion.candidate='';
            fs.writeFileSync(path.join(cwd, 'version.json'), JSON.stringify(lastVersion));
            process.exit(0);
    default :
        item = path.join(cwd, ...args[0].split(/[\\/]/g));
        if(fs.existsSync(item)) {
            folders.push(item);
        }
        break;
    }
    args = args.slice(1);
}

function allowed(ver) {
    const n = canonical(ver);
    const c = canonical(lastVersion.candidate ? lastVersion.candidate : lastVersion.release);
    return (
        c[0] <= n[0] &&
        (c[1] <= n[1] || c[0] < n[0]) &&
        (c[2] < n[2] || c[1] < n[1])
    );
}

const newVersion = argVersion && allowed(argVersion) ? canonical(argVersion) : (
    lastVersion.candidate ? canonical(lastVersion.candidate) : increment(canonical(lastVersion.release))
);
console.log(newVersion.join('.'));
folders.forEach(item => setVersion(item, newVersion.join('.')));

if(!lastVersion.candidate) {
    lastVersion.candidate = newVersion.join('.');
    fs.writeFileSync(path.join(cwd, 'version.json'), JSON.stringify(lastVersion));
}