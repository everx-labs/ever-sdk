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
        let result;
        if (request.method === 'context.create') {
            const context = wasmWrapper.core_create_context();
            result = JSON.stringify({result_json: JSON.stringify(context), error_json: ''});
        } else if (request.method === 'context.destroy') {
            wasmWrapper.core_destroy_context(request.context);
            result = JSON.stringify({result_json: '', error_json: ''});
        } else {
            result = wasmWrapper.core_json_request(request.context, request.method, request.params);
        }
        postMessage({
            response: {
                id: request.id,
                result,
            }
        });
    }
};
