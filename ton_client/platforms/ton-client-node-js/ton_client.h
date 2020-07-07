
// TON SDK JsonRpc C/C++ adapter

#pragma once

#include <stdint.h>

typedef struct {
    char* content;
    size_t len;
} InteropString;

enum OnResultFlags {
    OnResultFinished = 1,
};

typedef void (*OnResult)(
    int32_t request_id,
    InteropString result_json,
    InteropString error_json,
    int32_t flags);

#ifdef __APPLE__

#include <dlfcn.h>
#include <iostream>
#include <string>

using std::cerr;

typedef uint32_t (*tc_create_context_t)();
typedef void (*tc_destroy_context_t)(uint32_t context);
typedef void (*tc_json_request_async_t)(
    uint32_t context,
    InteropString method,
    InteropString params_json,
    int32_t requestId,
    OnResult on_result);


static tc_create_context_t tc_create_context_impl = NULL;
static tc_destroy_context_t tc_destroy_context_impl = NULL;
static tc_json_request_async_t tc_json_request_async_impl = NULL;

void* ensure_func_impl(void* lib_handle, const char* name)
{
    void* impl = dlsym(lib_handle, name);
    if (!impl) {
        cerr << "[" << __FILE__ << "] Unable to find [" << name << "] function: "
             << dlerror() << "\n";
        exit(EXIT_FAILURE);
    }
    return impl;
}

void ensure_impl()
{
    if (tc_create_context_impl) {
        return;
    }
    Dl_info info;
    if (!dladdr((void*)ensure_impl, &info)) {
        cerr << "[" << __FILE__ << "]: Unable to get lib info: "
             << dlerror() << "\n";
        exit(EXIT_FAILURE);
    }
    auto libpath = std::string(info.dli_fname);
    auto slash_pos = libpath.find_last_of("/\\");
    if (slash_pos != std::string::npos) {
        libpath = libpath.substr(0, slash_pos) + "/libtonclient.dylib";
    }

    void* lib_handle = dlopen(libpath.c_str(), RTLD_LOCAL);
    if (!lib_handle) {
        cerr << "[" << __FILE__ << "]: Unable to open library: "
             << dlerror() << "\n";
        exit(EXIT_FAILURE);
    }

    tc_create_context_impl = (tc_create_context_t)ensure_func_impl(lib_handle, "tc_create_context");
    tc_destroy_context_impl = (tc_destroy_context_t)ensure_func_impl(lib_handle, "tc_destroy_context");
    tc_json_request_async_impl = (tc_json_request_async_t)ensure_func_impl(lib_handle, "tc_json_request_async");
}

uint32_t tc_create_context()
{
    ensure_impl();
    return (*tc_create_context_impl)();
}

void tc_destroy_context(uint32_t context)
{
    ensure_impl();
    return (*tc_destroy_context_impl)(context);
}

void tc_json_request_async(
    uint32_t context,
    InteropString method,
    InteropString params_json,
    int32_t request_id,
    OnResult on_result)
{
    ensure_impl();
    (*tc_json_request_async_impl)(context, method, params_json, request_id, on_result);
}

#else

#ifdef __cplusplus
extern "C" {
#endif

uint32_t tc_create_context();
void tc_destroy_context(uint32_t context);
void tc_json_request_async(
    uint32_t context,
    InteropString method,
    InteropString params_json,
    int32_t request_id,
    OnResult on_result);

#ifdef __cplusplus
}
#endif

#endif
