This repository contains source code for all client libraries included in TON Labs SDK.

## Client core binaries

The best way to build client libraries is to use build scripts from this repo. 

**Note**: The scripts are written in JavaScript so you have to install Node.js (v.10 or newer) to run them.

To build a binary for a specific target (or binding), navigate to the relevant folder and run `node build.js`.

The resulting binaries are placed to `bin` folder in the gz-compressed fomat .

The list defines all build targets (paths are relative and determined to the location where you clone this repo):

- `ton_client/platforms/ton-client-node-js` – Node.js add-on (and an optional dylib for Mac OS)  used in Node.js-based JavaScript binding.

    Note that the build script generates binaries compatible with the platform used to run the script. For example, if you run it on Mac OS, you get binaries targeted at Darwin (macOS) platform.

- `ton_client/platforms/ton-client-react-native` –  iOS and Android native libraries for react-native mobile applications.
- `ton_client/platforms/ton-client-web` – WASM and JavaScript wrapper for browser-based applications.
- `ton_client/client` – general purpose dynamic link library. Currently, it is used in rust binding. It is a good starting place for creating a new bindings.

## Native Rust code

Apart from the code needed to generate a client library binary core, this repo offers a considerable TON-related code base in Rust and formed into cargo crates. Use these crates by integrating those directly into your Rust projects with a classic cargo dependencies.

Now we provide no further documentation on this complimentary code, but we will provide it later. The source code is well organised and easy to explore. Unit tests included in crates will help you with usage rules. Good luck guys!

For additional information and guidelines visit docs.ton.dev. Also check our [YouTube channel](https://www.youtube.com/channel/UC9kJ6DKaxSxk6T3lEGdq-Gg) for tutorials.
