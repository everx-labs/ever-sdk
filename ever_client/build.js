const fs = require('fs');
const os = require('os');
const {
    gz,
    spawnProcess,
    deleteFolderRecursive,
    main,
    version,
    root_path,
    postBuild,
    devMode,
} = require('./build-lib');
const platform = os.platform();
const cArch = os.arch();
const getArch = () => {
  for (let i = 1; i < process.argv.length - 1; i++) {
    if(process.argv[i] === '--target') {
      return process.argv[i + 1];
    }
  }
  return cArch;
}
const getRustTarget = () => {
  if(cArch === arch) {
    return '';

  }
  const platformTargets = {
    linux: {
      arm: 'armv7-unknown-linux-gnueabihf',
      arm64: 'aarch64-unknown-linux-gnu',
      x64: 'x86_64-unknown-linux-gnu'
    },
    win32: {
      x64: 'x86_64-pc-windows-msvc'
    },
    darwin: {
      x64: 'x86_64-apple-darwin',
      arm64: 'aarch64-apple-darwin'
    }
  };
  return platformTargets[platform]?.[arch] ?? '';
}
const arch = getArch();
const rustTarget = getRustTarget();

main(async () => {
    if(rustTarget) {
      await spawnProcess('rustup', ['target', 'add', rustTarget]);
    }
    console.log('Executing: cargo',
      ['build', '--release'].concat(rustTarget ? ['--target', rustTarget] : []).join(' ')
    );
    await spawnProcess('cargo', ['build', '--release'].concat(rustTarget ? ['--target', rustTarget] : []));
    deleteFolderRecursive(root_path('bin'));
    fs.mkdirSync(root_path('bin'), { recursive: true });
    const platformNames = {
        linux: {
          x64: [['lib{}.so', '']]
        },
        win32: {
          x64: [['{}.dll.lib', '_lib'], ['{}.dll', '_dll']]
        },
        darwin: {
          x64: [['lib{}.dylib', '']],
          arm64: [['lib{}.dylib', '']]
        },
    };
    for (const [src, dstSuffix] of platformNames[platform][arch] || []) {
        const target = ['..', 'target', rustTarget, 'release', src.replace('{}', 'ever_client')];
        await postBuild(target, platform);
        await gz(
          target,
          `tonclient_${version}_${platform}${cArch==arch ? '' : `_${arch}`}${dstSuffix || ''}`,
          [__dirname, 'build']
        );
    }
});
