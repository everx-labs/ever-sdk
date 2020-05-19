const wasmWrapper = {
    setup(instance) {
    }
};
//---
self.onmessage = (e) => {
    const message = e.data;
    const setup = message.setup;
    if (setup) {
        (async () => {
            const instance = (await WebAssembly.instantiate(setup.wasmModule, {
                wbg: wasmWrapper.wbg
            })).exports;
            wasmWrapper.setup(instance);
            postMessage({
                setup: {}
            })
        })();
        return;
    }
    const request = message.request;
    if (request) {
        const result = wasmWrapper.request(request.method, request.params);
        postMessage({
            response: {
                id: request.id,
                result,
            }
        });
    }
};
