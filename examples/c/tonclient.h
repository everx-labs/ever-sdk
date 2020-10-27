#pragma once

#include <cstdint>

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

#include <iostream>
#include <string>
#include <vector>

using namespace std;

tc_string_t tc_string(const char* s) {
    return tc_string_t { s, (uint32_t)strlen(s) };
}

class CoreContext {
public:
    struct Error {
        std::string error_json;
    };

    CoreContext(const char * config)
    {
        auto response_handle = tc_create_context(tc_string(config));
        auto response = tc_read_json_response(response_handle);
        auto result = std::string(response.result_json.content, response.result_json.len);
        auto error = std::string(response.error_json.content, response.error_json.len);
        cout << result << endl;
        cout << error << endl;
        _context = 0;
        tc_destroy_json_response(response_handle);
    }

    ~CoreContext()
    {
        if (_context > 0) {
            tc_destroy_context(_context);
            _context = 0;
        }
    }

    std::string request(const char* method, const char* params_json)
    {
        tc_string_t tc_method = { method, static_cast<uint32_t>(strlen(method)) };
        tc_string_t tc_params_json = { params_json, static_cast<uint32_t>(strlen(params_json)) };
        auto tc_response_handle = tc_json_request(_context, tc_method, tc_params_json);
        auto tc_response = tc_read_json_response(tc_response_handle);
        auto result_json = std::string(tc_response.result_json.content, tc_response.result_json.len);
        auto error_json = std::string(tc_response.error_json.content, tc_response.error_json.len);
        tc_destroy_json_response(tc_response_handle);
        if (error_json.length() > 0) {
            throw Error { error_json };
        }
        return result_json;
    }

private:
    uint32_t _context;
};
#endif
