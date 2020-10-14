#define NAPI_EXPERIMENTAL
#include <assert.h>
#include <memory.h>
#include <node_api.h>
#include <stdio.h>
#include <uv.h>
#include "tonclient.h"

#define CHECK(status) assert((status) == napi_ok)

// Utility

napi_value js_undefined(napi_env env)
{
    napi_value undefined;
    CHECK(napi_get_undefined(env, &undefined));
    return undefined;
}

napi_value js_global(napi_env env)
{
    napi_value global;
    CHECK(napi_get_global(env, &global));
    return global;
}

napi_value js_string_from_string_data(napi_env env, const tc_string_data_t ts)
{
    napi_value result;
    CHECK(napi_create_string_utf8(env, ts.len > 0 ? ts.content : nullptr, ts.len, &result));
    return result;
}

napi_value js_number(napi_env env, const uint32_t value)
{
    napi_value result;
    CHECK(napi_create_uint32(env, value, &result));
    return result;
}

napi_value js_boolean(napi_env env, bool value)
{
    napi_value result;
    CHECK(napi_create_uint32(env, value, &result));
    return result;
}

napi_value js_string(napi_env env, const char* s)
{
    napi_value value;
    CHECK(napi_create_string_utf8(env, s, NAPI_AUTO_LENGTH, &value));
    return value;
}

uint32_t get_uint32(napi_env env, napi_value value)
{
    uint32_t result = 0;
    CHECK(napi_get_value_uint32(env, value, &result));
    return result;
}

tc_string_data_t string_data_from_js(napi_env env, napi_value ns)
{
    tc_string_data_t result;
    size_t bytesRequired;
    CHECK(napi_get_value_string_utf8(env, ns, nullptr, 0, &bytesRequired));
    char* content = new char[bytesRequired + 1];
    size_t len = 0;
    CHECK(napi_get_value_string_utf8(env, ns, content, bytesRequired + 1, &len));
    result.len = len;
    result.content = content;
    return result;
}

tc_string_data_t string_data_clone(tc_string_data_t source)
{
    tc_string_data_t result;
    result.len = source.len;
    if (source.len > 0) {
        result.content = new char[source.len];
        memcpy(result.content, source.content, source.len);
    } else {
        result.content = nullptr;
    }
    return result;
}

void string_data_free(tc_string_data_t* source)
{
    delete source->content;
    source->content = nullptr;
    source->len = 0;
}

napi_threadsafe_function onResult;

napi_value onResultSync;
napi_env envSync;

struct response_t {
    uint32_t request_id;
    tc_string_data_t params_json;
    uint32_t response_type;
    bool finished;
};

// function(configJson: string): string
napi_value createContext(napi_env env, napi_callback_info info)
{
    size_t argc = 1;
    napi_value args[1];
    CHECK(napi_get_cb_info(env, info, &argc, args, nullptr, nullptr));
    tc_string_data_t config;
    if (argc > 0) {
        config = string_data_from_js(env, args[0]);
    } else {
        config = {nullptr, 0};
    }
    auto response_handle = tc_create_context(config);
    string_data_free(&config);
    auto response = tc_read_string(response_handle);
    auto js_response = js_string_from_string_data(env, response);
    tc_destroy_string(response_handle);

    napi_deferred deferred;
    napi_value promise;
    CHECK(napi_create_promise(env, &deferred, &promise));
    CHECK(napi_resolve_deferred(env, deferred, js_response));
    return promise;
}

// function(context: number): void
napi_value destroyContext(napi_env env, napi_callback_info info)
{
    size_t argc = 1;
    napi_value args[1];
    CHECK(napi_get_cb_info(env, info, &argc, args, nullptr, nullptr));
    if (argc > 0) {
        tc_destroy_context(get_uint32(env, args[0]));
    }
    return js_undefined(env);
}

//--------------------------------------------------------- request

static napi_threadsafe_function response_handler_func = nullptr;

// function(requestId: number, paramsJson: string, responseType: number, finished: boolean): void
void response_handler_func_call(napi_env env, napi_value func, void* context, void* data) {
    auto response = (response_t*)data;
    if (!func) {
        string_data_free(&response->params_json);
        delete response;
        return;
    }
    napi_value args[4];
    args[0] = js_number(env, response->request_id);
    args[1] = js_string_from_string_data(env, response->params_json);
    args[2] = js_number(env, response->response_type);
    CHECK(napi_coerce_to_bool(env, js_number(env, response->finished ? 1 : 0), &args[3]));
    string_data_free(&response->params_json);
    delete response;
    napi_value call_result;
    CHECK(napi_call_function(env, js_global(env), func, 4, args, &call_result));
}

// function(responseHandler?: ResponseHandler): void
napi_value setResponseHandler(napi_env env, napi_callback_info info)
{
    if (response_handler_func) {
        CHECK(napi_release_threadsafe_function(response_handler_func, napi_tsfn_abort));
        response_handler_func = nullptr;
    }

    size_t argc = 1;
    napi_value args[1];
    CHECK(napi_get_cb_info(env, info, &argc, args, nullptr, nullptr));
    if (argc > 0) {
        CHECK(napi_create_threadsafe_function(
                env,
                args[0],// napi_value func,
                nullptr, // napi_value async_resource,
                js_string(env, "TON Client response handler"), // napi_value async_resource_name,
                0, // size_t max_queue_size,
                1, // size_t initial_thread_count,
                nullptr, // void* thread_finalize_data,
                nullptr, // napi_finalize thread_finalize_cb,
                nullptr, // void* context,
                response_handler_func_call, // napi_threadsafe_function_call_js call_js_cb,
                &response_handler_func)); // napi_threadsafe_function* result);
    }
    return js_undefined(env);
}

void core_response_handler(uint32_t request_id, tc_string_data_t params_json, uint32_t response_type, bool finished) {
    if (!response_handler_func) {
        return;
    }
    auto response = new response_t;
    response->request_id = request_id;
    response->params_json = string_data_clone(params_json);
    response->response_type = response_type;
    response->finished = finished;
    CHECK(napi_call_threadsafe_function(
            response_handler_func,
            response,
            napi_tsfn_nonblocking));
}

// function(context: number, requestId: number, functionName: string, functionParamsJson: string): void
napi_value sendRequest(napi_env env, napi_callback_info info)
{
    size_t argc = 4;
    napi_value args[4];
    CHECK(napi_get_cb_info(env, info, &argc, args, nullptr, nullptr));
    if (argc >= 4) {
        auto context = get_uint32(env, args[0]);
        auto request_id = get_uint32(env, args[1]);
        auto function_name = string_data_from_js(env, args[2]);
        auto function_params_json = string_data_from_js(env, args[3]);
        tc_request(context, function_name, function_params_json, request_id, core_response_handler);
        string_data_free(&function_name);
        string_data_free(&function_params_json);
    }
    return js_undefined(env);
}


//--------------------------------------------------------- initialization

void unload(napi_env env, void* data, void* hint)
{
}

napi_value init(napi_env env, napi_value exports)
{
    napi_property_descriptor properties[4] = {
        { "setResponseHandler", nullptr, setResponseHandler, nullptr, nullptr, nullptr, napi_default, nullptr },
        { "createContext",      nullptr, createContext,      nullptr, nullptr, nullptr, napi_default, nullptr },
        { "destroyContext",     nullptr, destroyContext,     nullptr, nullptr, nullptr, napi_default, nullptr },
        { "sendRequest",        nullptr, sendRequest,        nullptr, nullptr, nullptr, napi_default, nullptr }
    };
    CHECK(napi_define_properties(env, exports, 4, properties));
    CHECK(napi_wrap(env, exports, nullptr, unload, nullptr, nullptr));
    return exports;
}

NAPI_MODULE(NODE_GYP_MODULE_NAME, init)
