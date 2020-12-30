# How to work with Application Objects in binding generators

Binding generator must detect functions that accept application objects.

Application object interaction protocol and functions that accept app objects have the following signatures in core:

```rust
/// Methods of the Foo interface with parameters
enum ParamsOfAppFoo {
    CallWithParamsAndResult { a: String, b: String },
    CallWithParams { a: String, b: String },
    CallWithResult,
    CallWithoutParamsAndResult,
    NotifyWithParams { a: String, b: String },
    NotifyWithoutParams
}

/// Method results of the Foo interface
enum ResultOfAppFoo {
    CallWithParamsAndResult { c: String },
    CallWithParams,
    CallWithResult { c: String },
    CallWithoutParamsAndResult,
}

/// API function that accepts an application object as a parameter
async fn foo(
		context: Arc<ClientContext>,
		params: ParamsOfFoo,
		obj: AppObject<ParamsOfAppFoo, ResultOfAppFoo>,
) -> ClientResult<()> {}
```

It means when a function accepts an application-implemented object, from that point library will have access to the methods of this object.

### How to detect

Function contains parameter with generic type `AppObject<foo_module.ParamsOfAppFoo, foo_module.ResultOfAppFoo>`. 

### Generated code

Generator must produce:

- interface declaration;
- interface dispatcher;
- function that accept application object.

### Interface declaration

```tsx
export type ParamsOfAppFooCallWithParamsAndResult {
    a: string,
    b: string,
}

export type ResultOfAppFooCallWithParamsAndResult {
    c: string,
}

export type ParamsOfAppFooCallWithParams {
    a: string,
    b: string,
}

export type ResultOfAppFooCallWithResult {
    c: string,
}

export type ParamsOfAppFooNotifyWithParams {
    a: string,
    b: string,
}

export interface AppFoo {
    call_with_params_and_result(params: ParamsOfFooWithParamsAndResult): Promise<ResultOfFooWithParamsAndResult>,
    call_with_params(params: ParamsOfFooWithParams): Promise<void>,
    call_with_result(): Promise<ResultOfFooWithResult>,
    notify_with_params(params: ParamsoOfFooNotifyWithParams),
    notify_without_params(),
}
```

- Interface  `Foo` is extracted from the name of the first generic arg of the `AppObject<foo_module.ParamsOfAppFoo, foo_module.ResultOfAppFoo>`. Note that a generic arg name is a fully qualified name so you must remove the module name first. In the example above the first arg name is `foo_module.ParamsOfAppFoo.` After removing module name we have `ParamsOfAppFoo`. Then we must remove the prefix `ParamsOf`. The rest of the name contains the interface name `Foo`.
- To collect a list of interface methods we must collect variant names from enum `foo_module.ParamsOfAppFoo`. Each variant of `ParamsOfAppFoo` represents the interface method. Respectively each variant of `ResultOfAppFoo` represents the result of the interface method. If interface method has no `ResultOfAppFoo` then such method is a notify method –  no waiting for the response is needed.
- The name of the interface method  is constructed from the name of the variant by using a simple rule `PascalStyleName` → `snake_style_name`. So the variant `CallWithParamsAndResult` will be converted to `call_with_params_and_result`.
- If a function has a result then this function must return `Promise` and perform asynchronous execution.

### Interface dispatcher

The implementation of the wrapper method is more difficult than regular. It must pass dispatching `responseHandler` to the library. Library will call this handler every time when it requires to call the application object. Two response types are used for calling application objects: `3` for calling methods which return result and `4` for notifiyng without awaiting any result. When response type `4` is passed, data contains enum `ParamsOfAppFoo`. When  `3` is passed, data contains struct `ParamsOfAppRequest`  where `request_data` field contains `ParamsOfAppFoo`

```tsx
type ParamsOfAppRequest {
		app_request_id: number,
		request_data: any,
}
```

Generator must define special dispatch helper for application object invocation:

```tsx
async function dispatchFoo(obj: Foo, params: ParamsOfAppFoo, app_request_id: number | null, client: TonClient) {
    try {
        let result = undefined;
		    switch (params.type) {
		    case 'CallWithParamsAndResult':
		        result = await obj.call_with_params_and_result(params);
		        break;
		    case 'CallWithParams':
		        await obj.call_with_params(params);
		        break;
		    case 'CallWithResult':
		        result = await obj.call_with_result();
		        break;
		    case 'CallWithoutParamsAndResult':
		        await obj.call_with_result();
		        break;
		    case 'NotifyWithParams':
		        obj.notify_with_params(params);
		        break;
		    case 'NotifyWithoutParams':
		        obj.notify_without_params();
		        break;
				}
				if (app_request_id) {
            client.resolve_app_request({ app_request_id, result: { type: 'Ok', result: { type: params.type, ...result }}});
        }
    }
    catch (error) {
        if (app_request_id) {
            client.resolve_app_request({ app_request_id, result: { type: 'Error', text: error.message }});
        }
    }
}

```

### Functions with application object

The `obj` parameter must be declared instead of the source `obj: AppObject<ParamsOfAppFoo, ResultOfAppFoo>`.

Wrapper implementation with dispatcher must be generated as:

```tsx
type ParamsOfFoo {
		...
}

export class AppFoo {
    foo(params: ParamsOfFoo, obj: Foo): Promise<ResultOfFoo> {
        return this.client.request('foo', params, (params, responseType) => {
            if (responseType === 3) {
                 dispatchFoo(obj, params.request_data, params.app_request_id, this);
            } else if (responseType === 4) {
                 dispatchFoo(obj, params, null, this);
            }
        }
    }
}
```
