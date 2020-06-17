
// TON SDK JsonRpc C/C++ adapter

#pragma once

#include <stdint.h>

typedef struct {
    char* content;
    size_t len;
} InteropString;

enum OnResultFlags {
    OnResultFlagsFinished = 1,
};

#ifdef __cplusplus
extern "C" {
#endif

uint32_t core_create_context();
void core_destroy_context(utin32_t context);
void core_request(
    uint32_t context,
    InteropString* method,
    InteropString* params_json,
    int32_t request_id,
    void (*on_result)(int32_t request_id, InteropString result_json, InteropString error_json, int32_t flags));


#ifdef __cplusplus
}
#endif
