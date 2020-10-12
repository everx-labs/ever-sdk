# Core Client Library for TON DApp development

**Community links:**

[![Chat on Telegram](https://img.shields.io/badge/chat-on%20telegram-9cf.svg)](https://t.me/ton_sdk)  [![Gitter](https://badges.gitter.im/ton-sdk/community.svg)](https://gitter.im/ton-sdk/community?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge)

**Documentation**  

[GraphQL API and SDK documentation](https://docs.ton.dev/86757ecb2/p/92b041-overview)

# What is Core Library

Core Library is a library written in Rust that can be dynamically linked. It provides all heavy-computation components and functions, such as TON Virtual Machine, TON Transaction Executor, ABI-related functions, boc-related functions, crypto functions. 
The decision to create the Core Rust Library was made after a period of time using pure JavaScript to implement these use cases. 
We ended up with very slow work of pure JavaScript and decided to move all this to Rust library and link it to Javascript as a compiled binary including a wasm module for browser applications. 

Also this approach provides an apportunity to easily create bindings for any programming language and platform, thus, to make it possible to develop distributed applications (DApps) for any possible use-cases, such as: mobile DApps, web DApps, server-side DApps, enterprise DApp etc.
Core Client Library exposes all the functionality through a few of exported functions. All interaction with core library is performed using OPEN-RPC like protocol.

Core library works over GraphQL API of [TON OS Dapp Server](https://github.com/tonlabs/TON-OS-DApp-Server). 
So, it can be used to interact directly with TON OS Clouds: 
- [Freeton](https://main.ton.dev/graphql)
- [Devnet](https://net.ton.dev/graphql)
- [Testnet](https://testnet.ton.dev/graphql)

# What is Binding

Binding is a thin client library written on the specific language that acts like a bridge between a core library and an application code written on that language.

- [Web binding](https://github.com/tonlabs/ton-client-web-js)  
- [Node.js binding](https://github.com/tonlabs/ton-client-node-js)  
- [React-native binding](https://github.com/tonlabs/ton-client-react-native-js)  
- [Rust binding](https://github.com/tonlabs/ton-client-rs)  

# Build Core Library

The best way to build client libraries is to use build scripts from this repo. 

**Note**: The scripts are written in JavaScript so you have to install Node.js (v.10 or newer) to run them. Also make sure you have the latest version of Rust installed.

To build a binary for a specific target (or binding), navigate to the relevant folder and run `node build.js`.

The resulting binaries are placed to `bin` folder in the gz-compressed fomat .

The list defines all build targets (paths are relative and determined to the location where you clone this repo):

- `ton_client/platforms/ton-client-node-js` – Node.js add-on (and an optional dylib for Mac OS)  used in Node.js-based JavaScript binding.

    Note that the build script generates binaries compatible with the platform used to run the script. For example, if you run it on Mac OS, you get binaries targeted at Darwin (macOS) platform.

- `ton_client/platforms/ton-client-react-native` –  iOS and Android native libraries for react-native mobile applications.
- `ton_client/platforms/ton-client-web` – WASM and JavaScript wrapper for browser-based applications.
- `ton_client/client` – general purpose dynamic link library. Currently, it is used in rust binding. It is a good starting place for creating a new bindings.


# Download Core Library

Instead of building core library yourself, you can download the __latest__ precompiled binaries from TON Labs SDK Binaries Store.
Platform | Major | Download links
-------- | -------------------- | --------------
Win32 | 0 | [`tonclient.lib`](http://sdkbinaries-ws.tonlabs.io/tonclient_0_win32_lib.gz), [`tonclient.dll`](http://sdkbinaries-ws.tonlabs.io/tonclient_0_win32_dll.gz)
&nbsp;| 1-rc | [`tonclient.lib`](http://sdkbinaries-ws.tonlabs.io/tonclient_1_0_0-rc_win32_lib.gz), [`tonclient.dll`](http://sdkbinaries-ws.tonlabs.io/tonclient_1_0_0-rc_win32_dll.gz)
macOS | 0 | [`tonclient.dylib`](http://sdkbinaries-ws.tonlabs.io/tonclient_0_darwin.gz)
&nbsp;| 1-rc | [`tonclient.dylib`](http://sdkbinaries-ws.tonlabs.io/tonclient_1_0_0-rc_darwin.gz)
Linux | 0 | [`tonclient.so`](http://sdkbinaries-ws.tonlabs.io/tonclient_0_linux.gz)
&nbsp;| 1-rc | [`tonclient.so`](http://sdkbinaries-ws.tonlabs.io/tonclient_1_0_0-rc_linux.gz)

If you want an older version of library (e.g. `0.25.0` for macOS), you need to choose a link to your platform from the list above and replace `0` with a version:
[http://sdkbinaries.tonlabs.io/tonclient_<b>0_25_0</b>_darwin.gz](http://sdkbinaries.tonlabs.io/tonclient_0_25_0_darwin.gz)

_Downloaded archive is gzipped file_

# Use Core Library

You can link core library to you application like any other dynamic link library. In this documentation we will write examples using C++ compiler on the macOS.

The first step is to create the `tonclient.h` file with declarations for types and functions used in core library interactions:

```cpp
#pragma once

#include <stdint.h>

typedef struct {
    const char* content;
    uint32_t len;
} tc_string_t;

typedef struct  {
    tc_string_t result_json;
    tc_string_t error_json;
} tc_response_t;

typedef struct  {
} tc_response_handle_t;

enum tc_response_flags_t {
    tc_response_finished = 1,
};

typedef void (*tc_on_response_t)(
    int32_t request_id,
    tc_string_t result_json,
    tc_string_t error_json,
    int32_t flags);

#ifdef __cplusplus
extern "C" {
#endif

uint32_t tc_create_context();
void tc_destroy_context(uint32_t context);
void tc_json_request_async(
    uint32_t context,
    tc_string_t method,
    tc_string_t params_json,
    int32_t request_id,
    tc_on_response_t on_result);

tc_response_handle_t* tc_json_request(
    uint32_t context,
    tc_string_t method,
    tc_string_t params_json);

tc_response_t tc_read_json_response(const tc_response_handle_t* handle);
void tc_destroy_json_response(const tc_response_handle_t* handle);

#ifdef __cplusplus
}
#endif
```

## Contexts

Core library uses contexts to operate with specific blockchain networks and specific set of options. So inside a single app you can work with several networks simultaneously. Just create your own context for every network you need.

Context is an integer handle to the internal core structure. Before using any client function you must create a context:

```cpp
auto context = tc_create_context();
```

After you have finished with client you have to close (destroy) context:

```cpp
tc_destroy_context(context);
```

## Passing Strings

All parameters and results are passed as strings with JSON-structures encoded with UTF-8.

String is represented as a structure with `content` pointer which points to UTF-8 string bytes and a `len` field that specifies number of bytes in content. The content bytes `is not null terminated`, so you must use provided `len`.

## JSON interface

Access to all client context functions can be performed through single core invocation function: 

```cpp
tc_response_handle_t* tc_json_request(
    uint32_t context,
    tc_string_t method,
    tc_string_t params_json);
```

Where:

- `context` is a context handle;
- `method` is a method (function) name;
- `params_json` is a string with JSON representation of method parameters or an empty string if method has no parameters.

You can find all the possible methods [in the reference section](https://github.com/tonlabs/TON-SDK/wiki/Core-Library-JSON-API)

The returned value is a handle to an internal structure that holds a response. You must extract responded data and then destroy a response handle.

To extract response data you must use:

```cpp
tc_response_t tc_read_json_response(const tc_response_handle_t* handle);
```

The data returned is valid until a call to `tc_destroy_response`. So if you need to hold a response data longer than a response handle you must copy this data before destroying response handle.

Core response contains two fields: `result_json` and `error_json`. If the `error_json` is empty then invocation is succeeded and `result_json` contains a JSON representation of function result.

When the `error_json` is not empty it contains a JSON representation of the error object and the `result_json` is empty.

In our example we create a thin wrapper class around TON Client context:

```cpp
#include <iostream>
#include <string>
#include <vector>

#include "tonclient.h"

using namespace std;

class TonClient {
public:
    struct Error {
        std::string error_json;
    };

    TonClient()
    {
        _context = tc_create_context();
    }

    ~TonClient()
    {
        if (_context > 0) {
            tc_destroy_context(_context);
            _context = 0;
        }
    }

    std::string request(const char* method, const char* params_json)
    {
        tc_string_t tc_method = { method, static_cast<uint32_t>(strlen(method)) };
        tc_string_t tc_params_json = { params_json, static_cast<uint32_t>(strlen(params_json)) };
        auto tc_response_handle = tc_json_request(_context, tc_method, tc_params_json);
        auto tc_response = tc_read_json_response(tc_response_handle);
        auto result_json = std::string(tc_response.result_json.content, tc_response.result_json.len);
        auto error_json = std::string(tc_response.error_json.content, tc_response.error_json.len);
        tc_destroy_json_response(tc_response_handle);
        if (error_json.length() > 0) {
            throw Error { error_json };
        }
        return result_json;
    }

private:
    uint32_t _context;
}; 
```

Now we can write some example app:

```cpp
int main()
{
    auto client = TonClient();
    try {
				// Print out a core version
        cout << "version: " << client.request("version", "") << endl;
        client.request("setup", R"(
					{ "servers": ["net.ton.dev"] }
				)");

				// Request and print 50 accounts
        cout << "accounts: " << 
				client.request("net.query", R"(
					{ 
						"table": "accounts", 
						"filter": "{}", 
						"result": "id balance(format:DEC) " 
					})") 
				<< endl;

 				// Request and print 50 messages
        cout << "messages: " << 
				client.request("net.query", R"(
					{
						"table": "messages",
						"filter": "{}",
						"result": "id src dst" 
					})") 
				<< endl;
    } catch (TonClient::Error error) {
        cerr << error.error_json << endl;
        return 1;
    }
    return 0;
}
```

All the functions of core library are divided into groups (or modules). For example, crypto functions, contracts functions etc.

## Setup Client Context

Every context must be initialised before use:

```cpp
client.request("setup", R"(
	{ "servers": ["net.ton.dev"] }
)");
```

The structure of setup parameters will be discussed below.

# Writing Bindings

If you want to use core library in some language please read following recommendations.

- Create a wrapper (or binding) library in chosen language around core library.
- Use an object oriented design – create a TonClient class that incapsulates core context.
- Add a nested objects of TonClient for each core module.
- Map core errors to error or exception system of your language.
