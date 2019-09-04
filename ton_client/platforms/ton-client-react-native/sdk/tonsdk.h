
// TON SDK JsonRpc C/C++ adapter

#pragma once

#include <stdint.h>

typedef struct {
} TONSDKRustString;

typedef struct {
    char* ptr;
    size_t len;
} TONSDKUtf8String;

enum OnResultFlags {
    OnResultFinished = 1,
};

#ifdef __cplusplus
extern "C" {
#endif

void ton_sdk_json_rpc_request(
    TONSDKUtf8String* method,
    TONSDKUtf8String* params_json,
    int32_t requestId,
    void (*on_result)(int32_t request_id, TONSDKUtf8String result_json, TONSDKUtf8String error_json, int32_t flags));

#ifdef __cplusplus
}
#endif
