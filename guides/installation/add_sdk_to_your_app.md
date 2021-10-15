# Add SDK to your App

[Node.js](add_sdk_to_your_app.md#nodejs) | [Web](add_sdk_to_your_app.md#web)

## Node.js

> Our library is fully-annotated with `.d.ts` files so we recommend to write your applications in Typescript.

Let's start with a clean npm project.

```
mkdir hello
cd hello
npm init -y
```

Now lets install core package and bridge package for Node.js

```
npm i --save @tonclient/core
npm i --save @tonclient/lib-node
```

If you want to use high-level AppKit package then install this package as well:

```
npm i --save @tonclient/appkit
```

You must initialize the library before the first use. The best place to do it is an initialization code of your application.

You need to attach the chosen binary module to the `TonClient` class. Create `index.js` file and add this code:

```
const {TonClient} = require("@tonclient/core");
const {libNode} = require("@tonclient/lib-node");

// Application initialization
TonClient.useBinaryLibrary(libNode)
```

That's it! Now you are ready to create and [configure TONClient object](configure_sdk.md)!

## Web

> Our library is fully-annotated with `.d.ts` files so we recommend to write your applications in Typescript.

Let's start with a clean project.

```
mkdir hello
cd hello
npm init -y
```

**Installation**

Now lets install core package and bridge package for Web

```
npm i --save @tonclient/core
npm i --save @tonclient/lib-web
```

**Important!** Each time you run `npm install` the new version of the `tonclient.wasm` and `index.js` is downloaded. So you have to always update the `tonclient.wasm` inside your web package before publishing (starting local web server, creating web bundle etc.). If you use Webpack the best way is to use CopyPlugin.

If you want to use high-level AppKit package then install this package as well:

```
npm i --save @tonclient/appkit
```

You must initialize the library before the first use. The best place to do it is in initialization code of your application.

You need to attach the chosen binary module to the `TonClient` class:

```
import { TonClient } from '@tonclient/core';
import { libWeb } from '@tonclient/lib-web';

TonClient.useBinaryLibrary(libWeb);
```
