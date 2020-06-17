#define NAPI_EXPERIMENTAL
#include <assert.h>
#include <memory.h>
#include <node_api.h>
#include <stdio.h>
#include <uv.h>
#include "ton_client.h"

#define CHECK(status) assert((status) == napi_ok)

// Utility

napi_value napiUndefined(napi_env env)
{
    napi_value undefined;
    CHECK(napi_get_undefined(env, &undefined));
    return undefined;
}

napi_value napiGlobal(napi_env env)
{
    napi_value global;
    CHECK(napi_get_global(env, &global));
    return global;
}

napi_value napiString(napi_env env, const InteropString ts)
{
    napi_value result;
    CHECK(napi_create_string_utf8(env, ts.content, ts.len, &result));
    return result;
}

napi_value napiNumber(napi_env env, const uint32_t value)
{
    napi_value result;
    CHECK(napi_create_uint32(env, value, &result));
    return result;
}

napi_value napiString(napi_env env, const char* s)
{
    napi_value value;
    CHECK(napi_create_string_utf8(env, s, NAPI_AUTO_LENGTH, &value));
    return value;
}

uint32_t getUInt32(napi_env env, napi_value value)
{
    uint32_t result = 0;
    CHECK(napi_get_value_uint32(env, value, &result));
    return result;
}

InteropString interopString(napi_env env, napi_value ns)
{
    InteropString result;
    size_t bytesRequired;
    CHECK(napi_get_value_string_utf8(env, ns, nullptr, 0, &bytesRequired));
    char* content = new char[bytesRequired + 1];
    CHECK(napi_get_value_string_utf8(env, ns, content, bytesRequired + 1, &result.len));
    result.content = content;
    return result;
}

InteropString interopString(InteropString source)
{
    InteropString result;
    result.content = new char[source.len];
    memcpy(result.content, source.content, source.len);
    result.len = source.len;
    return result;
}

void interopStringFree(InteropString* source)
{
    delete source->content;
    source->content = nullptr;
    source->len = 0;
}

// Request

// Adapter

struct NodeJsAdapter {
    struct Request {
        typedef int32_t Id;
        Id id;
        Request* next;
        napi_threadsafe_function onResult;

        napi_value onResultSync;
        napi_env envSync;

        Request(Id id, Request* next)
            : id(id)
            , next(next)
            , onResult(nullptr)
            , onResultSync(nullptr)
            , envSync(nullptr)
        {
        }
    };

    struct Result {
        Request::Id requestId;
        InteropString resultJson;
        InteropString errorJson;
        bool finished;

        Result(Request::Id requestId, InteropString resultJson, InteropString errorJson, bool finished)
            : requestId(requestId)
            , resultJson(interopString(resultJson))
            , errorJson(interopString(errorJson))
            , finished(finished)
        {
        }
        ~Result()
        {
            interopStringFree(&resultJson);
            interopStringFree(&errorJson);
        }
    };

    Request::Id nextRequestId = 0;
    Request* firstRequest = nullptr;
    uv_rwlock_t lock;

    NodeJsAdapter()
        : nextRequestId(1)
        , firstRequest(nullptr)
    {
        uv_rwlock_init(&lock);
    }

    ~NodeJsAdapter()
    {
        uv_rwlock_destroy(&lock);
    }

    void beginRead()
    {
        uv_rwlock_rdlock(&lock);
    }

    void endRead()
    {
        uv_rwlock_rdunlock(&lock);
    }

    void beginWrite()
    {
        uv_rwlock_wrlock(&lock);
    }

    void endWrite()
    {
        uv_rwlock_wrunlock(&lock);
    }

    Request* createRequest()
    {
        firstRequest = new Request(nextRequestId++, firstRequest);
        return firstRequest;
    }

    Request** findRequestPtr(Request::Id id)
    {
        auto ptr = &firstRequest;
        while (*ptr && (*ptr)->id != id) {
            ptr = &(*ptr)->next;
        }
        return ptr;
    }

    //--------------------------------------------------------- Async request processing

    // function request(methodName, paramsJson, onResult)
    static napi_value requestHandler(napi_env env, napi_callback_info info)
    {
        size_t argc = 3;
        napi_value args[3];
        NodeJsAdapter* adapter;
        CHECK(napi_get_cb_info(env, info, &argc, args, nullptr, (void**)&adapter));
        adapter->request(env, argc, args);
        return napiUndefined(env);
    }

    void request(napi_env env, int argc, napi_value* args)
    {
        beginWrite();
        auto request = createRequest();
        CHECK(napi_create_threadsafe_function(
            env,
            args[3],
            nullptr,
            napiString(env, "TON Client Core"),
            0,
            1,
            nullptr,
            nullptr,
            nullptr,
            callHandler,
            &request->onResult));

        //TODO: need to investigate necessary of the napi_ref for the onResult function pointer
        //
        //CHECK(napi_ref_threadsafe_function(env, request->onResult));

        endWrite();

        auto context = getUInt32(env, args[0]);
        auto method = interopString(env, args[1]);
        auto paramsJson = interopString(env, args[2]);
        tc_json_request_async(context, method, paramsJson, request->id, resultHandler);
        interopStringFree(&method);
        interopStringFree(&paramsJson);
    }

    static void resultHandler(int32_t request_id, InteropString result_json, InteropString error_json, int32_t flags)
    {
        auto adapter = shared;
        if (adapter) {
            adapter->onResult(request_id, result_json, error_json, (OnResultFlags)flags);
        }
    }

    void onResult(Request::Id id, InteropString resultJson, InteropString errorJson, OnResultFlags flags)
    {
        beginWrite();
        auto request = *findRequestPtr(id);
        if (request) {
            auto result = new Result(id, resultJson, errorJson, (flags & OnResultFinished) != 0);
            CHECK(napi_acquire_threadsafe_function(request->onResult));
            CHECK(napi_call_threadsafe_function(request->onResult, result, napi_tsfn_blocking));
            CHECK(napi_release_threadsafe_function(request->onResult, napi_tsfn_release));
        }
        endWrite();
    }

    static void callHandler(napi_env env, napi_value onResult, void* context, void* data)
    {
        auto adapter = shared;
        if (adapter) {
            adapter->onCall(env, onResult, context, data);
        }
    }

    void onCall(napi_env env, napi_value onResult, void* context, void* data)
    {
        auto result = (Result*)data;
        beginWrite();
        auto ptr = findRequestPtr(result->requestId);
        auto request = *ptr;
        if (request && result->finished) {
            *ptr = request->next;
        }
        endWrite();
        if (request) {
            napi_value args[2];
            napi_value callResult;

            args[0] = napiString(env, result->resultJson);
            args[1] = napiString(env, result->errorJson);
            CHECK(napi_call_function(
                env,
                napiGlobal(env),
                onResult,
                2,
                args,
                &callResult));
            if (result->finished) {
                CHECK(napi_unref_threadsafe_function(env, request->onResult));
                delete request;
            }
        }
        delete result;
    }

    //--------------------------------------------------------- Sync request processing

    // function request(methodName, paramsJson, onResult)
    static napi_value requestHandlerSync(napi_env env, napi_callback_info info)
    {
        size_t argc = 3;
        napi_value args[3];
        NodeJsAdapter* adapter;
        CHECK(napi_get_cb_info(env, info, &argc, args, nullptr, (void**)&adapter));
        adapter->requestSync(env, argc, args);
        return napiUndefined(env);
    }

    void requestSync(napi_env env, int argc, napi_value* args)
    {
        beginWrite();
        auto request = createRequest();
        request->onResultSync = args[2];
        request->envSync = env;
        endWrite();

        auto context = tc_create_context();
        auto method = interopString(env, args[0]);
        auto paramsJson = interopString(env, args[1]);
        tc_json_request_async(context, method, paramsJson, request->id, resultHandlerSync);
        interopStringFree(&method);
        interopStringFree(&paramsJson);
        tc_destroy_context(context);
    }

    // function coreRequest(context, methodName, paramsJson, onResult)
    static napi_value coreRequestHandlerSync(napi_env env, napi_callback_info info)
    {
        size_t argc = 4;
        napi_value args[4];
        NodeJsAdapter* adapter;
        CHECK(napi_get_cb_info(env, info, &argc, args, nullptr, (void**)&adapter));
        adapter->coreRequestSync(env, argc, args);
        return napiUndefined(env);
    }

    void coreRequestSync(napi_env env, int argc, napi_value* args)
    {
        beginWrite();
        auto request = createRequest();
        request->onResultSync = args[3];
        request->envSync = env;
        endWrite();

        auto context = getUInt32(env, args[0]);
        auto method = interopString(env, args[1]);
        auto paramsJson = interopString(env, args[2]);
        tc_json_request_async(context, method, paramsJson, request->id, resultHandlerSync);
        interopStringFree(&method);
        interopStringFree(&paramsJson);
    }

    static void resultHandlerSync(int32_t request_id, InteropString result_json, InteropString error_json, int32_t flags)
    {
        auto adapter = shared;
        if (adapter) {
            adapter->onResultSync(request_id, result_json, error_json, (OnResultFlags)flags);
        }
    }

    void onResultSync(Request::Id id, InteropString resultJson, InteropString errorJson, OnResultFlags flags)
    {
        beginWrite();
        auto ptr = findRequestPtr(id);
        auto request = *ptr;
        if (request) {
            *ptr = request->next;
        }
        endWrite();
        if (request == nullptr) {
            return;
        }
        napi_value args[2];
        napi_value callResult;

        args[0] = napiString(request->envSync, resultJson);
        args[1] = napiString(request->envSync, errorJson);
        CHECK(napi_call_function(
            request->envSync,
            napiGlobal(request->envSync),
            request->onResultSync,
            2,
            args,
            &callResult));
        delete request;
    }

    static napi_value coreCreateContextHandler(napi_env env, napi_callback_info info)
    {
        return napiNumber(env, tc_create_context());
    }

    static napi_value coreDestroyContextHandler(napi_env env, napi_callback_info info)
    {
        size_t argc = 1;
        napi_value args[1];
        NodeJsAdapter* adapter;
        CHECK(napi_get_cb_info(env, info, &argc, args, nullptr, (void**)&adapter));
        tc_destroy_context(getUInt32(env, args[0]));
        return napiUndefined(env);
    }

    //--------------------------------------------------------- Initialization

    static napi_value initHandler(napi_env env, napi_value exports)
    {
        shared = new NodeJsAdapter;
        napi_property_descriptor properties[4] = {
            { "request",
                nullptr,
                requestHandlerSync,
                nullptr,
                nullptr,
                nullptr,
                napi_default,
                shared },
            { "coreRequest",
                nullptr,
                coreRequestHandlerSync,
                nullptr,
                nullptr,
                nullptr,
                napi_default,
                shared },
            { "coreCreateContext",
                nullptr,
                coreCreateContextHandler,
                nullptr,
                nullptr,
                nullptr,
                napi_default,
                shared },
            { "coreDestroyContext",
                nullptr,
                coreDestroyContextHandler,
                nullptr,
                nullptr,
                nullptr,
                napi_default,
                shared }
        };
        CHECK(napi_define_properties(env, exports, 4, properties));
        CHECK(napi_wrap(env, exports, shared, unloadHandler, nullptr, nullptr));

        return exports;
    }

    static void unloadHandler(napi_env env, void* data, void* hint)
    {
        auto adapter = shared;
        if (adapter) {
            shared = nullptr;
            delete adapter;
        }
    }

    static NodeJsAdapter* shared;
};

NodeJsAdapter* NodeJsAdapter::shared = nullptr;

NAPI_MODULE(NODE_GYP_MODULE_NAME, NodeJsAdapter::initHandler)
