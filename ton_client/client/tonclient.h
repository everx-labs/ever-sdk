#pragma once

#include <stdint.h>

typedef struct {
    const char* content;
    uint32_t len;
} tc_string_t;

typedef struct  {
    tc_string_t result_json;
    tc_string_t error_json;
} tc_response_t;

typedef struct  {
} tc_response_handle_t;

enum tc_response_flags_t {
    tc_response_finished = 1,
};

typedef void (*tc_on_response_t)(
    uint32_t request_id,
    tc_string_t result_json,
    tc_string_t error_json,
    uint32_t flags);

#ifdef __cplusplus
extern "C" {
#endif

tc_response_handle_t* tc_create_context(tc_string_t config);
void tc_destroy_context(uint32_t context);
void tc_json_request_async(
    uint32_t context,
    tc_string_t method,
    tc_string_t params_json,
    uint32_t request_id,
    tc_on_response_t on_result);

tc_response_handle_t* tc_json_request(
    uint32_t context,
    tc_string_t method,
    tc_string_t params_json);

tc_response_t tc_read_json_response(const tc_response_handle_t* handle);
void tc_destroy_json_response(const tc_response_handle_t* handle);

#ifdef __cplusplus
}
#endif
