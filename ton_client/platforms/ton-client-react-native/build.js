const fs = require('fs');
const path = require('path');
const {deleteFolderRecursive, version, root_path} = require('../build-lib');
const url = require('url');
const http=require('http');
const zlib = require('zlib');
const outDir = root_path('output');
const ndkURLstr = 'http://dl.google.com/android/repository/android-ndk-r17c-darwin-x86_64.zip';
const ndkZipFile = root_path(((parts = ndkURLstr.split('/')).length < 1 ? null : parts[parts.length-1]));
const ndkDirName = root_path('android-ndk-r17c');

// parse arguments
let pArgs = process.argv.slice(2);
let build_Android=false;
let build_iOS=false;
while(pArgs.length > 0) {
	build_Android = build_Android ? true : pArgs[0].trim().toLowerCase() === '--android';
	build_iOS = build_iOS ? true : pArgs[0].trim().toLowerCase() === '--ios';
	pArgs = pArgs.slice(1);
}
if(!build_Android && !build_iOS) {
	build_Android = build_iOS = true;
}

const dev = {
	ios: {
		archs: ['x86_64-apple-ios'],
		lib: 'libton_client_react_native.a',
		header: 'ton_client.h'
	},
	android: {
		archs: ['i686-linux-android'],
		jniArchs: ['x86'],
		lib: 'libton_client_react_native.so',
	},
};
const release = JSON.parse(JSON.stringify(dev));
release.ios.archs.push('i386-apple-ios', 'armv7-apple-ios', 'armv7s-apple-ios', 'aarch64-apple-ios');
release.android.archs.push('aarch64-linux-android', 'armv7-linux-androideabi');
release.android.jniArchs.push('arm64-v8a', 'armeabi-v7a');
const cargoTargetsIOS = [
	"x86_64-apple-darwin",
	"aarch64-apple-ios",
	"armv7-apple-ios",
	"armv7s-apple-ios",
	"i386-apple-ios",
	"x86_64-apple-ios"
];
const cargoTargetsAndroid = [
	"x86_64-apple-darwin",
	"aarch64-linux-android",
	"armv7-linux-androideabi",
	"i686-linux-android"
];

const config = release;
const sdkDir = root_path('');
const iosDir = root_path('bin', 'ios');
const androidDir = root_path('bin', 'android');
const androidNDKs = ['arm64', 'arm', 'x86'];
const spawnEnv = {
	...process.env,
	PATH: [
		(process.env.PATH || ''),
		...androidNDKs.map(x => root_path('NDK', x, 'bin'))
	].join(':'),
};

async function gz(src, dst){
	return(new Promise((resolve, reject) => {
		fs.createReadStream(src)
		.pipe(zlib.createGzip({ level: 9}))
		.pipe(fs.createWriteStream(dst))
		.on('error', (err) => {
			reject(err);
		})
		.on('finish', () => {
			console.log(`GZipped ${src} -> ${dst}`);
			resolve();
		})
	}));
}

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

async function downloadNDK() {
	return(new Promise((resolve, reject) => {
		console.log('Downloading android NDK...');
		const ndkURL = url.parse(ndkURLstr);
		const fd = fs.createWriteStream(ndkZipFile, { encoding: 'binary' });
		const req = http.get(ndkURL, (res) => {
			res.pipe(fd);
			res.on('end', () => {
				resolve();
			});
		});
		req.on('error', (err) => {
			reject(err);
		});
	}));
}


async function getNDK() {
	let ndkHomeDir = process.env.NDK_HOME || '';
	if(ndkHomeDir === '' || !fs.existsSync(ndkHomeDir)) {
		try {
			if(!fs.existsSync(ndkZipFile)) await downloadNDK();
			console.log('Unzipping android NDK...');
			await spawnProcess('unzip', ['-q', '-d', root_path(''), ndkZipFile]);
			ndkHomeDir = ndkDirName;
			process.env.NDK_HOME = ndkHomeDir;
		} catch (err) {
			throw err;
		}
	}
	return(ndkHomeDir);
}


async function spawnAll(items, getArgs) {
	for(const item of items) {
		const args = getArgs(item);
		console.log(`Build: ${args.join(' ')}`);
		await spawnProcess(args[0], args.slice(1));
	}
}


async function checkNDK() {
	const ndkDir = root_path('NDK');
	if (fs.existsSync(ndkDir)) {
		console.log('Standalone NDK already exists...');
		return;
	}
	let ndkHomeDir = await getNDK();
	if (ndkHomeDir === '' || !fs.existsSync(ndkHomeDir)) {
		ndkHomeDir = path.join(process.env.ANDROID_HOME || '', 'ndk-bundle');
	}
	const maker = path.join(ndkHomeDir, 'build', 'tools', 'make_standalone_toolchain.py');
	if (!fs.existsSync(maker)) {
		console.error('Please install android-ndk: $ brew install android-ndk');
		process.exit(1);
	}
	mkdir(ndkDir);
	process.chdir(ndkDir);
	await spawnAll(androidNDKs, (arch) => {
		return ['python', maker, '--arch', arch, '--install-dir', arch];
	});
}


async function cargoBuild(targets) {
	return spawnAll(targets, x => ['cargo', 'build', '--target', x, '--release']);
}


async function buildReactNativeIosLibrary() {
	process.chdir(sdkDir);

	await cargoBuild(config.ios.archs);
	mkdir(iosDir);
	const dest = path.join(iosDir, config.ios.lib);
	const getIosOutput = x => path.join('target', x, 'release', config.ios.lib);
	await spawnProcess('lipo', [
		'-create',
		'-output', dest,
		...config.ios.archs.map(getIosOutput),
	]);

	if(fs.existsSync(dest)) {
		const header_src = path.join(sdkDir, config.ios.header);
		const header_dst = path.join(iosDir, config.ios.header);
		fs.copyFileSync(header_src, header_dst);

		const outGZip = path.join(outDir, `tonclient_${version}_react_native_ios.gz`);
		await gz(dest, outGZip);
	}
}


function mkdir(path) {
	if (!fs.existsSync(path)) {
		fs.mkdirSync(path, {recursive: true});
	}
}


async function buildReactNativeAndroidLibrary() {
	process.chdir(sdkDir);

	await cargoBuild(config.android.archs);
	const jniLibsDir = androidDir;
	mkdir(jniLibsDir);

	config.android.archs.forEach(async (arch, index) => {
		const jniArch = config.android.jniArchs[index];
		const jniArchDir = path.join(jniLibsDir, jniArch);
		mkdir(jniArchDir);
		const src = path.join(sdkDir, 'target', arch, 'release', config.android.lib);
		if (fs.existsSync(src)) {
			const dst = path.join(jniArchDir, config.android.lib);
			fs.copyFileSync(src, dst);
			process.stdout.write(`Android library for [${arch}] copied to "${dst}".\n`);
			const outGZip = path.join(outDir, `tonclient_${version}_react_native_${arch}.gz`);
			await gz(dst, outGZip);
		} else {
			process.stderr.write(`Android library for [${arch}] does not exists. Skipped.\n`);
		}
	});
}


(async () => {
	if(fs.existsSync(outDir)) {
		deleteFolderRecursive(outDir);
	}
	fs.mkdirSync(outDir);
	try {
		await checkNDK();
		let cargoTargets = [];
		cargoTargets = build_iOS ? cargoTargets.concat(cargoTargetsIOS) : cargoTargets;
		cargoTargets = build_Android ? cargoTargets.concat(cargoTargetsAndroid) : cargoTargets;
		await spawnProcess('rustup', ['target', 'add'].concat(cargoTargets));
		await spawnProcess('cargo', ['update']);
		if(build_iOS) {
			await buildReactNativeIosLibrary();
		}
		if(build_Android) {
			await buildReactNativeAndroidLibrary();
		}
	} catch (error) {
		console.error(error);
		process.exit(1);
	}
})();
