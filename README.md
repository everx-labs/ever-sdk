# TON Client Library for TON DApp development

**Community links:**

[![Chat on Telegram](https://img.shields.io/badge/chat-on%20telegram-9cf.svg)](https://t.me/ton_sdk)  [![Gitter](https://badges.gitter.im/ton-sdk/community.svg)](https://gitter.im/ton-sdk/community?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge)

**Documentation**  

[GraphQL API and SDK documentation](https://docs.ton.dev/86757ecb2/p/92b041-overview)

# What is TON Client Library

TON Client Library is a library written in Rust that can be dynamically linked. It provides all 
heavy-computation components and functions, such as TON Virtual Machine, TON Transaction 
Executor, ABI-related functions, boc-related functions, crypto functions. 

The decision to create the Rust library was made after a period of time using pure 
JavaScript to implement these use cases. 

We ended up with very slow work of pure JavaScript and decided to move all this to Rust 
library and link it to Javascript as a compiled binary including a wasm module for browser 
applications. 

Also this approach provides an opportunity to easily create bindings for any programming 
language and platform, thus, to make it possible to develop distributed applications (DApps) 
for any possible use-cases, such as: mobile DApps, web DApps, server-side DApps, enterprise 
DApp etc.

Client Library exposes all the functionality through a few of exported functions. All 
interaction with library is performed using JSON-RPC like protocol.

Library works over GraphQL API of [TON OS DApp Server](https://github.com/tonlabs/TON-OS-DApp-Server). 
So, it can be used to interact directly with TON OS Clouds: 
- [Freeton](https://main.ton.dev/graphql)
- [Devnet](https://net.ton.dev/graphql)
- [Testnet](https://testnet.ton.dev/graphql)

# How to use library

The simplest way is to use library in then Rust applications because of the native Rust library 
interface. The Rust interface is clear and well documented.

But what if you are required to use library in languages others than Rust?

You have some options:
- use library module `json_interface` which provides access to library functions through 
  JSON-RPC interface. This interface exports several extern "C" functions. So you can build
  a dynamic or static link library and link it to your application as any other external 
  libraries. The JSON Interface is fully "C" compliant. You can find description 
  in section [JSON Interface](docs/json_interface.md).
- use bindings already written by TON Labs and community. Below you can find a list of known 
  bindings.
- write your own binding to chosen language and share it with community.

If you choose using JSON Interface please read this document [JSON Interface](docs/json_interface.md).   
Here you can find directions how to use `json_interface` and write your own binding.
 
# Bindings

Binding is a thin client library written on the specific language that acts like a bridge between 
a client library and an application code written on that language.

List of known bindings:
- [Web binding](https://github.com/tonlabs/ton-client-web-js)  
- [Node.js binding](https://github.com/tonlabs/ton-client-node-js)  
- [React-native binding](https://github.com/tonlabs/ton-client-react-native-js)  
- [Rust binding](https://github.com/tonlabs/ton-client-rs)  

# Build client library

The best way to build client libraries is to use build scripts from this repo. 

**Note**: The scripts are written in JavaScript so you have to install Node.js (v.10 or newer) 
to run them. Also make sure you have the latest version of Rust installed.

To build a binary for a specific target (or binding), navigate to the relevant folder and 
run `node build.js`.

The resulting binaries are placed to `bin` folder in the gz-compressed format.

The list defines all build targets (paths are relative and determined to the location where 
you clone this repo):

- `ton_client/platforms/ton-client-node-js` – Node.js add-on (and an optional dylib for Mac OS)  used in Node.js-based JavaScript binding.

    Note that the build script generates binaries compatible with the platform used to run the script. For example, if you run it on Mac OS, you get binaries targeted at Darwin (macOS) platform.

- `ton_client/platforms/ton-client-react-native` –  iOS and Android native libraries for react-native mobile applications.
- `ton_client/platforms/ton-client-web` – WASM and JavaScript wrapper for browser-based applications.
- `ton_client/client` – general purpose dynamic link library. Currently, it is used in rust binding. It is a good starting place for creating a new bindings.


# Download precompiled binaries

Instead of building library yourself, you can download the __latest__ precompiled binaries from 
TON Labs SDK Binaries Store.

Platform | Major | Download links
-------- | ----- | --------------
Win32    | 0     | [`tonclient.lib`](http://sdkbinaries-ws.tonlabs.io/tonclient_0_win32_lib.gz), [`tonclient.dll`](http://sdkbinaries-ws.tonlabs.io/tonclient_0_win32_dll.gz), [`tonclient.node`](http://sdkbinaries-ws.tonlabs.io/tonclient_0_nodejs_addon_win32.gz)
&nbsp;   | 1     | [`tonclient.lib`](http://sdkbinaries-ws.tonlabs.io/tonclient_1_win32_lib.gz), [`tonclient.dll`](http://sdkbinaries-ws.tonlabs.io/tonclient_1_win32_dll.gz), [`tonclient.node`](http://sdkbinaries-ws.tonlabs.io/tonclient_1_nodejs_addon_win32.gz)
macOS    | 0     | [`libtonclient.dylib`](http://sdkbinaries-ws.tonlabs.io/tonclient_0_darwin.gz), [`tonclient.node`](http://sdkbinaries-ws.tonlabs.io/tonclient_0_nodejs_addon_darwin.gz), [`libtonclientnodejs.dylib`](http://sdkbinaries-ws.tonlabs.io/tonclient_0_nodejs_dylib_darwin.gz)
&nbsp;   | 1     | [`libtonclient.dylib`](http://sdkbinaries-ws.tonlabs.io/tonclient_1_darwin.gz), [`tonclient.node`](http://sdkbinaries-ws.tonlabs.io/tonclient_1_nodejs_addon_darwin.gz)
Linux    | 0     | [`libtonclient.so`](http://sdkbinaries-ws.tonlabs.io/tonclient_0_linux.gz), [`tonclient.node`](http://sdkbinaries-ws.tonlabs.io/tonclient_0_nodejs_addon_linux.gz)
&nbsp;   | 1     | [`libtonclient.so`](http://sdkbinaries-ws.tonlabs.io/tonclient_1_linux.gz), [`tonclient.node`](http://sdkbinaries-ws.tonlabs.io/tonclient_1_nodejs_addon_linux.gz)
WASM     | 0     | [`tonclient.wasm`](http://sdkbinaries-ws.tonlabs.io/tonclient_0_wasm.gz), [`tonclient.js`](http://sdkbinaries-ws.tonlabs.io/tonclient_0_wasm_js.gz)
&nbsp;   | 1     | [`tonclient.wasm`](http://sdkbinaries-ws.tonlabs.io/tonclient_1_wasm.gz), [`tonclient.js`](http://sdkbinaries-ws.tonlabs.io/tonclient_1_wasm_js.gz)
iOS      | 0     | [`libtonclient.a`](http://sdkbinaries-ws.tonlabs.io/tonclient_0_react_native_ios.gz)
Android  | 0     | [`armeabi-v7a/libtonclient.so`](http://sdkbinaries-ws.tonlabs.io/tonclient_0_react_native_armv7-linux-androideabi.gz), [`arm64-v8a/libtonclient.so`](http://sdkbinaries-ws.tonlabs.io/tonclient_0_react_native_aarch64-linux-android.gz), [`x86/libtonclient.so`](http://sdkbinaries-ws.tonlabs.io/tonclient_0_react_native_i686-linux-android.gz)

If you want an older version of library (e.g. `0.25.0` for macOS), you need to choose a link to your platform from the list above and replace `0` with a version:
[http://sdkbinaries.tonlabs.io/tonclient_<b>0_25_0</b>_darwin.gz](http://sdkbinaries.tonlabs.io/tonclient_0_25_0_darwin.gz)

_Downloaded archive is gzipped file_

