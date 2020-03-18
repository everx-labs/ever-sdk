const path = require('path');
const fs = require('fs');
const os = require('os');

if(process.argv.length < 5) {
    console.log(`Usage: node ${path.relative(process.cwd(), __filename)} <file> <from string> <to string>`);
    process.exit(1);
}

const file = path.join(process.cwd(), process.argv[2]);
const fromStr = process.argv[3];
const toStr = (os.platform() !== 'win32' ? process.argv[4] : process.argv[4].replace(/[A-Za-z]:\\/g, '/')).replace(/\\/g,'/');

if(!fs.existsSync(file)) {
    console.log(`File ${file} not found`);
    process.exit(1);
}

const cont_b = fs.readFileSync(file);
const re = new RegExp(fromStr,'g');
const cont_a = cont_b.toString().replace(re,toStr);
if(cont_b.toString() !== cont_a.toString()) {
    console.log(cont_a.toString());
}
fs.writeFileSync(file, cont_a);


