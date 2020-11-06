# JSON Interface to Ton Client

In addition to the native rust interface the core library has an alternative 
JSON RPC like interface.

The interaction with library is performed using an asynchronous request/response calls.

The library provides the `request` function to receive requests. And the application 
provides `response_handler` to receive responses from library related to requests.

This interface is offered for _bindings_ – a small wrapping libraries which purpose is 
to directly use the ton client library in languages others than _Rust_.
 
Counterparts:
- _Application_ – uses native language interface, provided by _binding_ library.
- _Binding_ – provides native language interface to ton client library. Uses JSON 
  Interface to directly call ton client library.  
- _Library (or Core)_ – provides JSON Interface to all ton client functionality.

## Strings

Many library functions operates with strings. So there are a responsibility for 
string ownership and lifetimes.

There are two types related to strings:
```c
typedef struct {
    const char* content;
    uint32_t len;
} tc_string_data_t;

typedef struct tc_string_handle_t tc_string_handle_t;

```
- `tc_string_handle_t` – internal Rust string representation.
  Application or binding can't use this memory directly. There is the 
  `tc_read_string` function for this purpose. Application responsible for 
  the releasing of string with `rc_destroy_string` function.
- `tc_string_data_t` – temporarily access string internal data. `content` field 
  points to the `utf8` encoded content of the string and the `len` field 
  contains the content size in bytes. Note that content **IS NOT NULL TERMINATED**.       

String manipulation functions:
```c
tc_string_data_t tc_read_string(const tc_string_handle_t* string);
void tc_destroy_string(const tc_string_handle_t* string);
```
- `tc_read_string` – read string content provided by the `string` pointer. 
Returned value is the internal string data. Note that this data will be 
invalid after the string will be destroyed.
- `tc_destroy_string` – destroys rust string provided by `string` pointer.

   
## Contexts

All library functions requires _context_ – the main library object that encapsulates 
configuration and state data.

Application can create several different contexts and use them all together. 
For example – creates two contexts that configured to work with different blockchain networks.

Context related functions:
```c
tc_string_handle_t* tc_create_context(tc_string_data_t config);
void tc_destroy_context(uint32_t context);
```
- `tc_create_context` – create context using provided `config` with configuration json.
  Returned string is a JSON with the result or the error. Result is returned in 
  form of `{ "result": context }` where `context` is a number with context handle.
  Error is returned in form `{ "error": { error fields } }`.
  **Note**: `tc_create_context` doesn't store pointer passed in `config` parameter. So
  it is safe to free this memory after the function returns.  
  **Important**: application is responsible for freeing of the receiving string. Example:
  ```c
  tc_string_data config = {"{}", 2};
  tc_string_handle_t* json_ptr = tc_create_context(config);
  tc_string_data json = tc_read_string(json_ptr);
  uint32_t context = parse_create_context_json(json.content, json.len);
  tc_free_string(json_ptr);
  ```
- `tc_destroy_context` – closes and releases all recourses that was allocated and opened 
  by library during serving functions related to provided context.
   
## Request

When application requires to invoke some ton client function it sends a function 
request to the library.

```c
void tc_request(
    uint32_t context,
    tc_string_data_t function_name,
    tc_string_data_t function_params_json,
    uint32_t request_id,
    tc_response_handler_t response_handler);

void tc_request_ptr(
    uint32_t context,
    tc_string_data_t function_name,
    tc_string_data_t function_params_json,
    void* request_ptr,
    tc_response_handler_ptr_t response_handler);
```
Where:
- `function_name` – function name requested.
- `function_params_json` – function parameters encoded as a JSON string. If a function 
  hasn't parameters then en empty string must be passed.
- `request_id` or `request_ptr` – application (or binding) defined request identifier or pointer.
Usually binding allocates and stores some additional data with every request.
This data will help in the future to properly route responses to the application. 
- `response_handler` – function that will receive responses related to this request.

This function returns nothing. The function execution result will be sent to 
the `response_handler`.

**Note**: `response_handler` can be called before the function returns.

**Note**: `tc_request` doesn't store pointers passed in `function_name` and 
`function_params_json` parameters. So it is safe to free this memory after the 
function returns.  

### Request Id versus Request Pointer

Ton Client Library has two version of request context representation:
- `id` – Each request is identified by `u32` integer value defined by application.
In this case the application or binding usually uses global hash map to associate 
additional response dispatch information.
- `pointer` – Each request is identified by `void*` pointer defined by application.
In this case the application or binding uses pointers to native objects with 
additional response dispatch information. For example a pointer to closure.
Note, that library doesn't use this pointer and memory pointed to. It just store
this pointer and provide it back to application when library calls response handler.

**Note** `pointer` supports is UNSTABLE feature yet and can be refined. 

## Responses

Application (or binding) defines function `response_handler` that will receive the 
request responses.

The response includes the mandatory function result or error and optional additional 
data responses.

```c
enum tc_response_types_t {
    tc_response_success = 0,
    tc_response_error = 1,
    tc_response_nop = 2,
    tc_response_custom = 100,
};

typedef void (*tc_response_handler_t)(
    uint32_t request_id,
    tc_string_data_t params_json,
    uint32_t response_type,
    bool finished);

typedef void (*tc_response_handler_ptr_t)(
    void* request_ptr,
    tc_string_data_t params_json,
    uint32_t response_type,
    bool finished);
```

`response_handler` – handles responses from the library. Note that an application 
can receive an unlimited count of responses related to single request. Parameters:
- `request_id` or `request_ptr` – the request to which this response is addressed.
- `params_json` – response parameters encoded into JSON string.
- `response_type` – type of this response:
    - `RESULT = 0`, function result.
    - `ERROR = 1`, function execution error.
    - `NOP = 2`, no operation. In combination with `finished = true` signals that the request handling was finished.
    - `RESERVED = 3..99` – reserved for protocol internal purposes. Application (or binding) must ignore this response. 
      Nevertheless the binding must check the `finished` flag to release data, associated with request.
    - `STREAM = 100..` - additional function data related to request handling. Depends on the function.
- `finished` – is a signal to release all additional data associated with the request. It is last response for specified 
  request_id.

**Important**:
- Application MUST NOT store pointers passed in `params_json` and use it after `response_handler` 
  has been returned, if an application requires this data after returning then it must creates 
  an own copy.
- Application MUST NOT free memory of pointers passed in `params_json`.
- Response handler can be called before the `tc_request` returns. In this case the response handler 
  will be called on the calling thread.
- Responses can be called on background thread created by library to serve asynchronous tasks. 
  All responses, related to the same request will be called from the same thread in right sequence.
  
# Bindings

Here we are look to the typical binding structure. In this example we will 
use the *Type Script*.

Declare high level function wrapper:

```ts
async function getVersion(context: number): Promise<string>;
```

Define additional data allocated for each request: 

```ts
type Request = {
    resolve: (result: any) => void,
    reject: (error: Error) => void,
    responseHandler?: (params: any) => void,
}

const requests = new Map<number, Request>();
let nextRequestId = 1;

```

Map library responses to high level handlers:

```ts
function libraryResponseHandler(
    requestId: number, 
    paramsJson: string, 
    responseType: number,
    finished: bool
) {
    const request = requests.get(requestId);
    if (!request) {
        return;
    }
    if (finished) {
        requests.delete(requestId);
    }
    const params = paramsJson !== '' ? JSON.parse(paramsJson) : undefined;
    switch (responseType) {
        case 0: // RESULT
            request.resolve(params);
            break;
        case 1: // ERROR
            request.reject(params);
            break;
        default: // DATA
            if (responseType >= 100 && request.responseHandler) {
                request.responseHandler(params);
            }
            break;
    }
}

```

Map high level call to library request / response chain:

```ts
function requestLibrary(
    context: number,
    functionName: string, 
    functionParams: any, 
    responseHandler?: (params: any) => any,
): Promise<any> {
    return new Promise((resolve, reject) => {
        const requestId = nextRequestId;
        nextRequestId += 1;
        requests.set(requestId, { resolve, reject, responseHandler });
        tc_request(context, functionName, functionParams, requestId, libraryResponseHandler);
    });
}

``` 

Implement high level function:

```ts
async function getVersion(context: number): Promise<string> {
    const response = await requestLibrary(context, "client.version", "");
    return response.version;
}
```
