#define NAPI_EXPERIMENTAL
#include <assert.h>
#include <memory.h>
#include <node_api.h>
#include <uv.h>
#include <stdio.h>
#include "ton_client.h"

#define CHECK(status) assert((status) == napi_ok)

// Utility

napi_value napiUndefined(napi_env env) {
    napi_value undefined;
    CHECK(napi_get_undefined(env, &undefined));
    return undefined;
}

napi_value napiGlobal(napi_env env) {
    napi_value global;
    CHECK(napi_get_global(env, &global));
    return global;
}

napi_value napiString(napi_env env, const TonSdkUtf8String ts) {
    napi_value result;
    CHECK(napi_create_string_utf8(env, ts.ptr, ts.len, &result));
    return result;
}

napi_value napiString(napi_env env, const char* s) {
    napi_value value;
    CHECK(napi_create_string_utf8(env, s, NAPI_AUTO_LENGTH, &value));
    return value;
}

TonSdkUtf8String tonString(napi_env env, napi_value ns) {
    TonSdkUtf8String result;
    size_t bytesRequired;
    CHECK(napi_get_value_string_utf8(env, ns, nullptr, 0, &bytesRequired));
    char* ptr = new char[bytesRequired + 1];
    CHECK(napi_get_value_string_utf8(env, ns, ptr, bytesRequired + 1, &result.len));
    result.ptr = ptr;
    return result;
}

TonSdkUtf8String tonString(TonSdkUtf8String source) {
    TonSdkUtf8String result;
    result.ptr = new char[source.len];
    memcpy(result.ptr, source.ptr, source.len);
    result.len = source.len;
    return result;
}

void tonStringFree(TonSdkUtf8String* source) {
    delete source->ptr;
    source->ptr = nullptr;
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

        Request(Id id, Request* next)
            : id(id)
            , next(next)
            , onResult(nullptr) {
        }
    };

    struct Result {
        Request::Id requestId;
        TonSdkUtf8String resultJson;
        TonSdkUtf8String errorJson;
        bool finished;

        Result(Request::Id requestId, TonSdkUtf8String resultJson, TonSdkUtf8String errorJson, bool finished)
            : requestId(requestId)
            , resultJson(tonString(resultJson))
            , errorJson(tonString(errorJson))
            , finished(finished) {
        }
        ~Result() {
            tonStringFree(&resultJson);
            tonStringFree(&errorJson);
        }
    };

    Request::Id nextRequestId = 0;
    Request* firstRequest = nullptr;
    uv_rwlock_t lock;

    NodeJsAdapter()
        : nextRequestId(1)
        , firstRequest(nullptr) {
        uv_rwlock_init(&lock);
    }

    ~NodeJsAdapter() {
        uv_rwlock_destroy(&lock);
    }

    void beginRead() {
        uv_rwlock_rdlock(&lock);
    }

    void endRead() {
        uv_rwlock_rdunlock(&lock);
    }

    void beginWrite() {
        uv_rwlock_wrlock(&lock);
    }

    void endWrite() {
        uv_rwlock_wrunlock(&lock);
    }

    Request* createRequest() {
        firstRequest = new Request(nextRequestId++, firstRequest);
        return firstRequest;
    }

    Request** findRequestPtr(Request::Id id) {
        auto ptr = &firstRequest;
        while (*ptr && (*ptr)->id != id) {
            ptr = &(*ptr)->next;
        }
        return ptr;
    }

    void request(napi_env env, int argc, napi_value* args) {
        beginWrite();
        auto request = createRequest();
        CHECK(napi_create_threadsafe_function(
            env,
            args[2],
            nullptr,
            napiString(env, "TON SDK JsonApi"),
            0,
            1,
            nullptr,
            nullptr,
            nullptr,
            callHandler,
            &request->onResult));
        CHECK(napi_ref_threadsafe_function(env, request->onResult));
        endWrite();

        auto method = tonString(env, args[0]);
        auto paramsJson = tonString(env, args[1]);
        ton_sdk_json_rpc_request(&method, &paramsJson, request->id, resultHandler);
        tonStringFree(&method);
        tonStringFree(&paramsJson);
    }

    void onResult(Request::Id id, TonSdkUtf8String resultJson, TonSdkUtf8String errorJson, OnResultFlags flags) {
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

    void onCall(napi_env env, napi_value onResult, void* context, void* data) {
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

    // function request(methodName, paramsJson, onResult)
    static napi_value requestHandler(napi_env env, napi_callback_info info) {
        size_t argc = 3;
        napi_value args[3];
        NodeJsAdapter* adapter;
        CHECK(napi_get_cb_info(env, info, &argc, args, nullptr, (void**)&adapter));
        adapter->request(env, argc, args);
        return napiUndefined(env);
    }

    static void resultHandler(int32_t request_id, TonSdkUtf8String result_json, TonSdkUtf8String error_json, int32_t flags) {
        auto adapter = shared;
        if (adapter) {
            adapter->onResult(request_id, result_json, error_json, (OnResultFlags)flags);
        }
    }

    static void callHandler(napi_env env, napi_value onResult, void* context, void* data) {
        auto adapter = shared;
        if (adapter) {
            adapter->onCall(env, onResult, context, data);
        }
    }

    static napi_value initHandler(napi_env env, napi_value exports) {
        shared = new NodeJsAdapter;
        napi_property_descriptor requestProperty = {
            "request",
            nullptr,
            requestHandler,
            nullptr,
            nullptr,
            nullptr,
            napi_default,
            shared
        };

        CHECK(napi_define_properties(env, exports, 1, &requestProperty));
        CHECK(napi_wrap(env, exports, shared, unloadHandler, nullptr, nullptr));

        return exports;
    }

    static void unloadHandler(napi_env env, void* data, void* hint) {
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
