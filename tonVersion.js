const fs = require('fs');
const os = require('os');
const path = require('path');
const https = require('https');

const canonical = (versionString) => {
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
const get = (url) => { 
    return new Promise((resolve, reject) => {
        const req = https.get(url, (res) => {
            let data = '';
            res.on('data', (part) => { 
                data = data.concat(data, part);
            });
            res.on('end', () => {
                resolve(JSON.parse(data.toString()));
            });
        });
        req.on('error', (err) => {
            reject(err);
        });
    });
};
const getVersion = async () => {
    try {
        return(await get('https://s3.eu-central-1.amazonaws.com/sdkbinaries.tonlabs.io/version.json'));
    } catch (e) {
        throw e;
    }
}
const increment = (canonicalVersion) => {
    canonicalVersion[2]++;
    return canonicalVersion
};
const setVersion = (dest, version) => {
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
while(args.length > 0) {
    item = path.join(cwd, ...args[0].split(/[\\/]/g));
    if(fs.existsSync(item)) {
        folders.push(item);
    }
    args = args.slice(1);
}

(async () => {
    try {
        const lastVersion = await getVersion();
        const newVersion = lastVersion.candidate ? canonical(lastVersion.candidate) : increment(canonical(lastVersion.release));
        console.log(newVersion.join('.'));
        folders.forEach(item => setVersion(item, newVersion.join('.')));
    } catch (err) {
        throw err;
    }
        
})();