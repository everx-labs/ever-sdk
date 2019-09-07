
// TON SDK JsonRpc C/C++ adapter

#pragma once

#include <stdint.h>

typedef struct {
} TonSdkRustString;

typedef struct {
    char* ptr;
    size_t len;
} TonSdkUtf8String;

enum OnResultFlags {
    OnResultFinished = 1,
};

#ifdef __APPLE__

#include <iostream>
#include <dlfcn.h>
#include <string>

using std::cerr;

typedef void (*on_result_t)(int32_t request_id, TonSdkUtf8String result_json, TonSdkUtf8String error_json, int32_t flags);
typedef void (*ton_sdk_json_rpc_request_t)(
    TonSdkUtf8String* method,
    TonSdkUtf8String* params_json,
    int32_t requestId,
    on_result_t on_result);

static ton_sdk_json_rpc_request_t ton_sdk_json_rpc_request_impl = NULL;

void ton_sdk_json_rpc_request(
    TonSdkUtf8String* method,
    TonSdkUtf8String* params_json,
    int32_t requestId,
    on_result_t on_result)
{
    if (!ton_sdk_json_rpc_request_impl) {
        Dl_info info;
        if (!dladdr((void*)ton_sdk_json_rpc_request, &info)) {
            cerr << "[" << __FILE__ << "]: Unable to get lib info: "
                 << dlerror() << "\n";
            exit(EXIT_FAILURE);
        }
        auto libpath = std::string(info.dli_fname);
        auto slash_pos = libpath.find_last_of("/\\");
        if (slash_pos != std::string::npos) {
            libpath = libpath.substr(0, slash_pos) + "/libtonclientnodejs.dylib";
        }

        void* lib_handle = dlopen(libpath.c_str(), RTLD_LOCAL);
        if (!lib_handle) {
            cerr << "[" << __FILE__ << "]: Unable to open library: "
                 << dlerror() << "\n";
            exit(EXIT_FAILURE);
        }

        ton_sdk_json_rpc_request_impl = (ton_sdk_json_rpc_request_t)dlsym(lib_handle, "ton_sdk_json_rpc_request");
        if (!ton_sdk_json_rpc_request_impl) {
            cerr << "[" << __FILE__ << "] Unable to find [ton_sdk_json_rpc_request] function: "
              << dlerror() << "\n";
            exit(EXIT_FAILURE);
        }
    }
    (*ton_sdk_json_rpc_request_impl)(method, params_json, requestId, on_result);
}

#else

#ifdef __cplusplus
extern "C" {
#endif

void ton_sdk_json_rpc_request(
    TonSdkUtf8String* method,
    TonSdkUtf8String* params_json,
    int32_t requestId,
    void (*on_result)(int32_t request_id, TonSdkUtf8String result_json, TonSdkUtf8String error_json, int32_t flags));

#ifdef __cplusplus
}
#endif

#endif
