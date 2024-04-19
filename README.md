# Ever SDK

Client libraries for DApp development in TVM blockchains (Everscale, Gosh, TON, Venom Blockchain, etc).

Core Ever-SDK client library is written in Rust, with bindings in 11 programming languages.

**Get quick help in our telegram channel:**

[![Channel on Telegram](https://img.shields.io/badge/chat-on%20telegram-9cf.svg)](https://t.me/ever\_sdk)

* [Ever SDK](./#ever-sdk)
  * [Supported languages](./#supported-languages)
    * [Rust (core library)](./#rust-core-library)
    * [Official Javascript(Typescript) binding](./#official-javascripttypescript-binding)
    * [Community bindings](./#community-bindings)
    * [If you did not find the language you need](./#if-you-did-not-find-the-language-you-need)
  * [Use-cases](./#use-cases)
  * [Quick Start](./#quick-start)
  * [How to avoid Soft Breaking Problems](./#how-to-avoid-soft-breaking-problems)
  * [Build client library](./#build-client-library)
  * [Build artifacts](./#build-artifacts)
  * [Run tests](./#run-tests)
  * [Download precompiled binaries](./#download-precompiled-binaries)

## Supported languages

### Rust (core library)

Repository: https://github.com/everx-labs/ever-sdk

**What is Core Client Library?**

Core Client Library is written in Rust that can be dynamically linked. It provides all heavy-computation components and functions, such as TON Virtual Machine, TON Transaction Executor, ABI-related functions, boc-related functions, crypto functions.

The decision to create the Rust library was made after a period of time using pure JavaScript to implement these use cases.

We ended up with very slow work of pure JavaScript and decided to move all this to Rust library and link it to Javascript as a compiled binary including a wasm module for browser applications.

Also this approach provided an opportunity to easily create bindings for any programming language and platform, thus, to make it possible to develop distributed applications (DApps) for any possible use-cases, such as: mobile DApps, web DApps, server-side DApps, enterprise DApp etc.

Client Library exposes all the functionality through a few of exported functions. All interaction with library is performed using JSON-RPC like protocol.

### Official Javascript(Typescript) binding

Repository: [JavaScript SDK](https://github.com/everx-labs/ever-sdk-js)

You need to install core package and the package with binary for your platform. [See the documentation.](https://github.com/everx-labs/ever-sdk-js#library-distribution)

| Platform                       | Package                                                                                                           |
| ------------------------------ | ----------------------------------------------------------------------------------------------------------------- |
| core package for all platforms | [@eversdk/core](https://github.com/everx-labs/ever-sdk-js#install-core-package)                                      |
| Node.js                        | [@eversdk/lib-node](https://github.com/everx-labs/ever-sdk-js#nodejs)                                                |
| Web                            | [@eversdk/lib-web](https://github.com/everx-labs/ever-sdk-js#web)                                                    |
| React-Native                   | [@eversdk/lib-react-native](https://github.com/everx-labs/ever-sdk-js#react-native)                                  |
| React-Native with JSI support  | [@eversdk/lib-react-native-jsi](https://github.com/everx-labs/ever-sdk-js/tree/master/packages/lib-react-native-jsi) |

### Community bindings

| Language | Repository                                                                                                                                                                               |
| -------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Clojure  | [serge-medvedev/tonos-client-clojure](https://github.com/serge-medvedev/tonos-client-clojure)                                                                                            |
| Golang   | <p><a href="https://github.com/radianceteam/ton-client-go">radianceteam/ton-client-go</a><br><a href="https://github.com/markgenuine/ever-client-go">markgenuine/ever-client-go</a></p>  |
| Java     | <p><a href="https://github.com/radianceteam/ton-client-java">radianceteam/ton-client-java</a><br><a href="https://github.com/deplant/java4ever-binding">laugan/java4ever-binding</a></p> |
| Kotlin   | [mdorofeev/ton-client-kotlin](https://github.com/mdorofeev/ton-client-kotlin)                                                                                                            |
| Lua      | [serge-medvedev/tonos-client-lua](https://github.com/serge-medvedev/tonos-client-lua)                                                                                                    |
| .NET     | [everscale-actions/everscale-dotnet](https://github.com/everscale-actions/everscale-dotnet)                                                                                              |
| PHP      | <p><a href="https://github.com/extraton/php-ton-client">extraton/php-ton-client</a><br><a href="https://github.com/radianceteam/ton-client-php">radianceteam/ton-client-php</a></p>      |
| Python   | [move-ton/ton-client-py](https://github.com/move-ton/ton-client-py)                                                                                                                      |
| Ruby     | [nerzh/ton-client-ruby](https://github.com/nerzh/ton-client-ruby)                                                                                                                        |
| Scala    | [radianceteam/ton-client-scala](https://github.com/radianceteam/ton-client-scala)                                                                                                        |
| Swift    | [nerzh/ton-client-swift](https://github.com/nerzh/ton-client-swift)                                                                                                                      |

### If you did not find the language you need

* use library module `json_interface` which provides access to library functions through JSON-RPC interface. This interface exports several extern "C" functions. So you can build a dynamic or static link library and link it to your application as any other external libraries. The JSON Interface is fully "C" compliant. You can find description in section [JSON Interface](docs/for-binding-developers/json\_interface.md).
* write your own binding to chosen language and share it with community.

If you choose using JSON Interface please read this document [JSON Interface](docs/for-binding-developers/json\_interface.md).\
Here you can find directions how to use `json_interface` and write your own binding.

## Use-cases

With Ever-SDK you can implement logic of any complexity on TVM compatible blockchains (Everscale, Gosh, TON, Venom, etc).

* Create and send messages to blockchain
* Process messages reliably (supports retries and message expiration mechanics)
* Supports TON Solidity and ABI compatible contracts
* Emulate transactions locally
* Run get methods
* Get account state
* Query blockchain data (blocks, transactions, messages)
* Subscripe to events and any other blockchain updates (literally)
* Sign data/check signature, calculate hashes (sha256, sha512), encrypt/decrypt data
* Validate addresses
* Work with blockchain native types (bag of cells or BOCs): encode, decode, calculate hash, etc
* Works on top of GraphQL API and compatible with Evernode-SE/DS, Evercloud.

## Quick Start

Get your endpoing at [dashboard.evercloud.dev](https://dashboard.evercloud.dev/) or run Evernode-SE (local node) or Dapp Server (self-hosted blockchain node).

See the list of available TVM networks: [https://docs.everplatform.dev/products/evercloud/networks-endpoints](https://docs.everplatform.dev/products/evercloud/networks-endpoints)

[Quick Start (Javascript binding)](docs/quick\_start.md)

[Error descriptions](docs/reference/error\_codes.md)

[JavaScript SDK Types and Methods (API Reference)](https://everx-labs.github.io/ever-sdk-js/)

[Core Types and Methods (API Reference)](docs/reference/types-and-methods/modules.md)

[Guides](docs/guides/installation/add\_sdk\_to\_your\_app.md)

## How to avoid Soft Breaking Problems

Soft Breaking is API changes that include only new optional fields in the existing structures. This changes are fully backward compatible for JSON Interface.

But in Rust such changes can produce some problems with an old client code.

Look at the example below:

1. There is an API v1.0 function `foo` and the corresponding params structure:

```rust
#[derive(Default)]
struct ParamsOfFoo {
    pub foo: String,
}

pub fn foo(params: ParamsOfFoo)
```

1. Application uses this function in this way:

```rust
foo(ParamsOfFoo {
    foo: "foo".into(),
});
```

1. API v.1.1 introduces new field in `ParamsOfFoo`:

```rust
#[derive(Default)]
struct ParamsOfFoo {
    pub foo: String,
    pub bar: Option<String>,
}
```

From the perspective of JSON-interface it isn't breaking change because the new parameter is optional. But code snippet (2) will produce Rust compilation error.

1. To avoid such problems we recommend to use default implementation inside structure initialisation:

```rust
foo(ParamsOfFoo {
    foo: "foo".into(),
    ..Default::default(),
});
```

For all Ton Client API structures `Default` trait is implemented.

## Build client library

The best way to build client libraries is to use build scripts from this repo.

**Note**: The scripts are written in JavaScript so you have to install Node.js (v.10 or newer) to run them. Also make sure you have the latest version of Rust installed.

To build a binary for a specific target (or binding), navigate to the relevant folder and run `node build.js`.

The resulting binaries are placed to `bin` folder in the gz-compressed format.

Note that the build script generates binaries compatible with the platform used to run the script. For example, if you run it on Mac OS, you get binaries targeted at Darwin (macOS) platform.

**Note**: You need latest version of rust. Upgrade it with `rustup update` command. Check version with `rustc --version`, it should be above or equal to `1.47.0`.

## Build artifacts

Rebuild `api.json`:

```shell
cd toncli
cargo run api -o ../tools
```

Rebuild `docs`:

```shell
cd tools
npm i
tsc
node index docs -o ../docs
```

Rebuild `modules.ts`:

```shell
cd tools
npm i
tsc
node index binding -l ts -o ../../ever-sdk-js/packages/core/src
```

## Run tests

To run test suite use standard Rust test command

```
cargo test
```

SDK tests need [EVER OS API](https://docs.everplatform.dev/reference/graphql-api/networks) endpoint to run on. Such an API is exposed by a [DApp Server](https://github.com/everx-labs/evernode-ds) which runs in real networks and by local blockchain [Evernode SE](https://github.com/everx-labs/evernode-se).

Evernode SE is used by default with address `http://localhost` and port 80. If you launch it on another port you need to specify it explicitly like this: `http://localhost:port`. If you have Evernode SE running on another address or you need to run tests on a real Everscale network use the following environment variables to override the default parameters

```
TON_USE_SE: true/false - flag defining if tests run against Evernode SE or a real network (DApp Server)
TON_NETWORK_ADDRESS - Dapp server or Evernode SE addresses separated by comma.
TON_GIVER_SECRET - Giver secret key. If not defined, default Evernode SE giver keys are used
TON_GIVER_ADDRESS - Address of the giver to use for prepaying accounts before deploying test contracts. If not defined, the address is calculated using `GiverV2.tvc` and configured public key
EVERCLOUD_AUTH_PROJECT – Evercloud project id used to authorise tests that requires main net interaction 
```

## Download precompiled binaries

Instead of building library yourself, you can download the **latest** precompiled binaries from EverX SDK Binaries Store.

| Platform | Major | Download links                                                                                                                                           |
| -------- | ----- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Win32    | 0     | [`ton_client.lib`](https://binaries.everx-labs.io/tonclient\_0\_win32\_lib.gz), [`ton_client.dll`](https://binaries.everx-labs.io/tonclient\_0\_win32\_dll.gz) |
|          | 1     | [`ton_client.lib`](https://binaries.everx-labs.io/tonclient\_1\_win32\_lib.gz), [`ton_client.dll`](https://binaries.everx-labs.io/tonclient\_1\_win32\_dll.gz) |
| macOS    | 0     | [`libton_client.dylib`](https://binaries.everx-labs.io/tonclient\_0\_darwin.gz)                                                                             |
|          | 1     | (x86\_64)[`libton_client.dylib`](https://binaries.everx-labs.io/tonclient\_1\_darwin.gz)                                                                    |
|          | 1     | (aarch64)[`libton_client.dylib`](https://binaries.everx-labs.io/tonclient\_1\_darwin\_arm64.gz)                                                             |
| Linux    | 0     | [`libton_client.so`](https://binaries.everx-labs.io/tonclient\_0\_linux.gz)                                                                                 |
|          | 1     | [`libton_client.so`](https://binaries.everx-labs.io/tonclient\_1\_linux.gz)                                                                                 |

If you want an older version of library (e.g. `0.25.0` for macOS), you need to choose a link to your platform from the list above and replace `0` with a version: [https://binaries.everx-labs.io/tonclient\_**0\_25\_0**\_darwin.gz](https://binaries.everx-labs.io/tonclient\_0\_25\_0\_darwin.gz)

_Downloaded archive is gzipped file_
