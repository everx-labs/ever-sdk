{
  'variables': {
    'base_cflags': [
      '-Wall',
      '-Wextra',
      '-Wno-unused-parameter',
      '-std=c++11',
    ],
    'debug_cflags': ['-g', '-O0'],
    'release_cflags': ['-O3'],
  },
  'targets': [
    {
      'target_name': 'tonclient',
      'sources': ['binding.cc'],
      'conditions': [
        ['OS == "win"', {
          'libraries': [
            '../target/release/ton_client_node_js.lib',
            'advapi32.lib',
            'ws2_32.lib',
            'userenv.lib',
            'shell32.lib',
            'Secur32.lib',
            'Crypt32.lib',
          ],
        }, {
          'libraries': [
            '../target/release/libton_client_node_js.a',
            '-Wl,-rpath,./addon/'
          ],
        }],
      ],
      'configurations': {
        'Debug': {
          'cflags': ['<@(debug_cflags)'],
          'xcode_settings': {
            'OTHER_CFLAGS': ['<@(debug_cflags)'],
          },
        },
        'Release': {
          'cflags': ['<@(release_cflags)'],
          'xcode_settings': {
            'OTHER_CFLAGS': ['<@(release_cflags)'],
          },
        },
      },
      'cflags': ['<@(base_cflags)'],
      'xcode_settings': {
        'OTHER_CFLAGS': ['<@(base_cflags)'],
      },
    },
  ],
}
