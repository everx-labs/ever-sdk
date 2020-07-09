// This file is just a template that used to generate index.js at npm installation stage

import {TONClient} from 'ton-client-js';

const workerScript = '';

//---

const wasmOptions = {
    debugLog: null,
    binaryURL: '/tonclient.wasm',
};

function debugLog(message) {
    if (wasmOptions.debugLog) {
        wasmOptions.debugLog(message);
    }
}

const createLibrary = async () => {
    const workerBlob = new Blob(
        [workerScript],
        { type: 'application/javascript' }
    );
    const workerUrl = URL.createObjectURL(workerBlob);
    const worker = new Worker(workerUrl);

    const activeRequests = new Map();

    // Deferred requests are accumulated before WASM module have been loaded
    let deferredRequests = [];

    let nextActiveRequestId = 1;

    worker.onerror = (evt) => {
        console.log(`Error from Web Worker: ${evt.message}`);
    };

    const coreRequest = (context, method, params, callback) => {
        const id = nextActiveRequestId;
        nextActiveRequestId += 1;
        const request = {
            id,
            context,
            method,
            params,
        };
        const isDeferredSetup = (method === 'setup') && (deferredRequests !== null);
        activeRequests.set(id, {
            callback: isDeferredSetup ? () => {
            } : callback
        });
        if (deferredRequests !== null) {
            deferredRequests.push(request);
        } else {
            worker.postMessage({ request });
        }
        if (isDeferredSetup) {
            callback('', '');
        }
    }

    const library = {
        coreCreateContext: (callback) => {
            coreRequest(0, 'context.create', '', (resultJson) => {
                if (callback) {
                    const context = JSON.parse(resultJson);
                    callback(context);
                }
            });
        },
        coreDestroyContext: (context, callback) => {
            coreRequest(context, 'context.destroy', '', () => {
                if (callback) {
                    callback();
                }
            });
        },
        coreRequest,
        request: (method, params, callback) => {
            coreRequest(1, method, params, callback);
        },
    };

    worker.onmessage = (evt) => {
        const setup = evt.data.setup;
        if (setup) {
            for (const request of deferredRequests) {
                worker.postMessage({ request });
            }
            deferredRequests = null;
            return;
        }

        const response = evt.data.response;
        if (response) {
            const activeRequest = activeRequests.get(response.id);
            if (!activeRequest) {
                return;
            }
            activeRequests.delete(response.id);
            if (activeRequest.callback) {
                let { result } = response;
                // Remove BOM from result
                result = result.charCodeAt(0) === 0xFEFF ? result.substr(1) : result;
                const { result_json, error_json } = JSON.parse(result);
                activeRequest.callback(result_json, error_json);
            }
        }
    };

    (async () => {
        const e = Date.now();
        let wasmModule;
        const fetched = fetch(wasmOptions.binaryURL);
        if (WebAssembly.compileStreaming) {
            debugLog('compileStreaming binary');
            wasmModule = await WebAssembly.compileStreaming(fetched);
        } else {
            debugLog('compile binary');
            wasmModule = await WebAssembly.compile(await (await fetched).arrayBuffer());
        }
        worker.postMessage({
            setup: {
                wasmModule,
            }
        });
        debugLog(`compile time ${Date.now() - e}`);
    })();

    return Promise.resolve(library);
};

function setWasmOptions(options) {
    Object.assign(wasmOptions, options);
}

const clientPlatform = {
    fetch,
    WebSocket,
    createLibrary,
};

function initTONClient(tonClientClass) {
    tonClientClass.setLibrary(clientPlatform);
}

initTONClient(TONClient);

export {
    createLibrary,
    setWasmOptions,
    clientPlatform,
    initTONClient,
    TONClient
};
