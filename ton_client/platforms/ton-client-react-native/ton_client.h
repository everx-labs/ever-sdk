
// TON SDK JsonRpc C/C++ adapter

#pragma once

#include <stdint.h>

typedef struct {
    char* content;
    uint32_t len;
} InteropString;

enum OnResultFlags {
    OnResultFinished = 1,
};

typedef void (*OnResult)(
    int32_t request_id,
    InteropString result_json,
    InteropString error_json,
    int32_t flags);

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

void ton_sdk_json_rpc_request(
    uint32_t context,
    InteropString* method,
    InteropString* params_json,
    int32_t request_id,
    OnResult on_result);


#ifdef __cplusplus
}
#endif
