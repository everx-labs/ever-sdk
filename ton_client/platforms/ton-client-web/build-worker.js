const wasmWrapper = {
    setup(instance) {
    }
};
//---

const coreContexts = new Map();

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
        let coreContext = coreContexts.get(request.context);
        if (!coreContext) {
            coreContext = wasmWrapper.core_create_context();
            coreContexts.set(request.context, coreContext);
        }
        let result;
        if (request.method === 'context.destroy') {
            wasmWrapper.core_destroy_context(coreContext);
            coreContexts.delete(request.context);
            result = {
                result_json: '',
                error_json: '',
            }
        } else {
            result = wasmWrapper.core_json_request(coreContext, request.method, request.params);
        }
        postMessage({
            response: {
                id: request.id,
                result,
            }
        });
    }
};
