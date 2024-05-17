# Add SDK to your App

[Node.js](add\_sdk\_to\_your\_app.md#nodejs) | [Web](add\_sdk\_to\_your\_app.md#web)

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
npm i --save @eversdk/core
npm i --save @eversdk/lib-node
```

If you want to use high-level [AppKit](https://github.com/everx-labs/ever-appkit-js) package then install this package as well:

```
npm i --save @eversdk/appkit
```

You must initialize the library before the first use. The best place to do it is an initialization code of your application.

You need to attach the chosen binary module to the `TonClient` class. Create `index.js` file and add this code:

```
const {TonClient} = require("@eversdk/core");
const {libNode} = require("@eversdk/lib-node");

// Application initialization
TonClient.useBinaryLibrary(libNode)
```

That's it! Now you are ready to create and[ configure TONClient object!](../configuration/endpoint-configuration.md)

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
npm i --save @eversdk/core
npm i --save @eversdk/lib-web
```

**Important!** Each time you run `npm install` the new version of the `eversdk.wasm` and `index.js` is downloaded. So you have to always update the `eversdk.wasm` inside your web package before publishing (starting local web server, creating web bundle etc.). If you use Webpack the best way is to use CopyPlugin.

If you want to use high-level [AppKit](https://github.com/everx-labs/ever-appkit-js) package then install this package as well:

```
npm i --save @eversdk/appkit
```

You must initialize the library before the first use. The best place to do it is in initialization code of your application.

You need to attach the chosen binary module to the `TonClient` class:

```
import { TonClient } from '@eversdk/core';
import { libWeb } from '@eversdk/lib-web';

TonClient.useBinaryLibrary(libWeb);
```
