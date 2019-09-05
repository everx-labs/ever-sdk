const fs = require('fs');
const path = require('path');
const url = require('url');
const http=require('http');
const zlib = require('zlib');
const root = __dirname;
const outDir = path.join(root, 'output');
const ndkURLstr = 'http://dl.google.com/android/repository/android-ndk-r17c-darwin-x86_64.zip';
const ndkZipFile = (parts = ndkURLstr.split('/')).length < 1 ? null : parts[parts.length-1];
const ndkDirName = 'android-ndk-r17c';

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
const cargoTargets = [
	"aarch64-apple-ios",
	"aarch64-linux-android",
	"armv7-apple-ios",
	"armv7-linux-androideabi",
	"armv7s-apple-ios",
	"i386-apple-ios",
	"i686-linux-android",
	"x86_64-apple-darwin",
	"x86_64-apple-ios"
];

const config = release;
const sdkDir = root;
const iosDir = path.join(root, 'bin', 'ios');
const androidDir = path.join(root, 'bin', 'android');
const androidNDKs = ['arm64', 'arm', 'x86'];
const spawnEnv = {
	...process.env,
	PATH: [
		(process.env.PATH || ''),
		...androidNDKs.map(x => path.join(root, 'NDK', x, 'bin'))
	].join(':'),
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
			ndkHomeDir = path.join(root,ndkDirName);
			console.log('Unzipping android NDK...');
			await spawnProcess('unzip', ['-q', '-d', root, ndkZipFile]);
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
	const ndkDir = path.join(root, 'NDK');
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

		const outGZip = path.join(outDir, `${path.parse(dest).name}-ios${path.parse(dest).ext}.gz`);
		fs.createReadStream(dest).pipe(zlib.createGzip()).pipe(fs.createWriteStream(outGZip));
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

	config.android.archs.forEach((arch, index) => {
		const jniArch = config.android.jniArchs[index];
		const jniArchDir = path.join(jniLibsDir, jniArch);
		mkdir(jniArchDir);
		const src = path.join(sdkDir, 'target', arch, 'release', config.android.lib);
		if (fs.existsSync(src)) {
			const dst = path.join(jniArchDir, config.android.lib);
			fs.copyFileSync(src, dst);
			process.stdout.write(`Android library for [${arch}] copied to "${dst}".\n`);
			const outGZip = path.join(outDir, `${path.parse(dst).name}-${arch}${path.parse(dst).ext}.gz`);
			fs.createReadStream(dst).pipe(zlib.createGzip()).pipe(fs.createWriteStream(outGZip));
		} else {
			process.stderr.write(`Android library for [${arch}] does not exists. Skipped.\n`);
		}
	});
}


(async () => {
	if(!fs.existsSync(outDir)) {
		fs.mkdirSync(outDir);
	} else {
		fs.readdirSync(outDir).forEach(item => {
			fs.unlinkSync(path.join(outDir, item));
		});
	}
	try {
		await checkNDK();
		await spawnProcess('rustup', ['target', 'add'].concat(cargoTargets));
		await spawnProcess('cargo', ['clean']);
		await spawnProcess('cargo', ['update']);
		await buildReactNativeIosLibrary();
		await buildReactNativeAndroidLibrary();
	} catch (error) {
		console.error(error);
		process.exit(1);
	}
})();
