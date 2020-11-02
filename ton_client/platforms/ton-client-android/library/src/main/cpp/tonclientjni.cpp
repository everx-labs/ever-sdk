/**
 * TON Client JNI Bindings
 * @author Martin Zeitler
 *
 * @see <a href="https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/types.html">JNI Types and Data Structures</a>
 * @see <a href="https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#interface_function_table">JNI Functions</a>
 *
 * The JNI supports the conversion to and from native Unicode and UTF-8 strings.
 * In particular, UTF-8 strings use the highest bit-to-signal multibyte characters;
 * they are therefore upwards-compatible with 7-bit ASCII.
 * In Java, UTF-8 strings are always 0-terminated.
 *
 * Header `jni.h`:
 * typedef unsigned char   jboolean;
 * typedef unsigned short  jchar;
 * typedef short           jshort;
 * typedef float           jfloat;
 * typedef double          jdouble;
 * typedef jint            jsize;
 *
 * Machine Header `win32/jni_mh.h`:
 * typedef long            jint;
 * typedef __int64         jlong;
 * typedef signed char     jbyte;
 *
**/
#include <cstdio>
#include <cctype>
#include <cstring>
#include <string>

#include <android/log.h>
#include <tonclientjni.h>
#include <io_tonlabs_client_NativeInterface.h>
using namespace std;

string jstring_to_string(JNIEnv* env, jstring jstr) {
    jboolean isCopy;
    const char* utf_string = env->GetStringUTFChars(jstr, &isCopy);
    string str = string(utf_string);
    env->ReleaseStringUTFChars(jstr, utf_string);
    return str;
}

#ifdef __cplusplus
extern "C" {
#endif

#ifdef __ANDROID__
void android_debug(const char *message) {
    __android_log_print(ANDROID_LOG_DEBUG, "TonClient", "JNI result: %s", message);
}
void android_error(const char *message) {
    __android_log_print(ANDROID_LOG_ERROR, "TonClient", "JNI error: %s", message);
}
#else
    void android_debug(const char *message) {}
    void android_error(const char *message) {}
#endif

const char* jstring_to_chars(JNIEnv* env, jstring jstr) {
    jboolean isCopy;
    const char* str = env->GetStringUTFChars(jstr, &isCopy);
    return str;
}

jstring chars_to_jstring(JNIEnv* env, const char* str) {
    return env->NewStringUTF(str);
}

jstring string_to_jstring(JNIEnv* env, const string& str) {
    return env->NewStringUTF(str.c_str());
}

uint32_t jstrlen(JNIEnv* env, jstring jstr) {
    // GetStringChars takes the Java string and returns a pointer to an array of Unicode characters that comprise the string.
    jboolean isCopy;
    const char* utf_string = env->GetStringUTFChars(jstr, &isCopy);
    auto length = static_cast<uint32_t>(strlen(utf_string));
    env->ReleaseStringUTFChars(jstr, utf_string);
    return length;
}

tc_string_t get_tc_string(JNIEnv *env, jstring jstr) {
    const char* content = jstring_to_chars(env, jstr);
    tc_string_t tc_str = { content, jstrlen(env, jstr) };
    android_debug( jstring_to_chars(env, jstr));
    return tc_str;
}

jstring tc_on_response(JNIEnv *env, tc_response_t tc_response) {
    auto result_json = string(tc_response.result_json.content, tc_response.result_json.len);
    auto error_json = string(tc_response.error_json.content, tc_response.error_json.len);
    jstring data = env->NewStringUTF("");
    if (result_json.length() > 0) {
        data = string_to_jstring(env, result_json);
        android_debug(result_json.c_str());
    }
    if (error_json.length() > 0) {
        data = string_to_jstring(env, error_json);
        android_error(error_json.c_str());
    }
    return data;
}

/** tc_create_context */
JNIEXPORT jint JNICALL Java_io_tonlabs_client_NativeInterface_createContext(JNIEnv* env, jobject caller) {
    auto context = tc_create_context();
    #ifdef __ANDROID__
        __android_log_print(ANDROID_LOG_INFO, "TonClient", "JNI context created: %s", (to_string(context)).c_str());
    #endif
    return (jint) context;
}

/** tc_destroy_context */
JNIEXPORT void JNICALL Java_io_tonlabs_client_NativeInterface_destroyContext(JNIEnv* env, jobject caller, jint context) {
    tc_destroy_context((uint32_t) context);
    #ifdef __ANDROID__
        __android_log_print(ANDROID_LOG_INFO, "TonClient", "JNI context destroyed: %s", (to_string(context)).c_str());
    #endif
}

/** tc_json_request */
JNIEXPORT jlong JNICALL Java_io_tonlabs_client_NativeInterface_jsonRequest(JNIEnv* env, jobject caller, jint context, jstring method, jstring params_json) {
	tc_response_handle_t* tc_response_handle = tc_json_request((uint32_t) context, get_tc_string(env, method), get_tc_string(env, params_json));
    return (jlong) tc_response_handle;
}

/** tc_json_request_async */
// https://github.com/tonlabs/TON-SDK/blob/master/ton_client/platforms/ton-client-react-native/ton_client.h
JNIEXPORT void JNICALL Java_io_tonlabs_client_Native_jsonRequestAsync(JNIEnv* env, jobject caller, jint context, jstring method, jstring params_json, jint request_id, jobject on_response) {
    tc_on_response_t tc_on_response = {};
    tc_json_request_async(
        (uint32_t) context,
        get_tc_string(env, method),
        get_tc_string(env, params_json),
        (int32_t) request_id,
        (tc_on_response_t) tc_on_response
    );
}

/** tc_read_json_response */
JNIEXPORT jstring JNICALL Java_io_tonlabs_client_NativeInterface_readJsonResponse(JNIEnv* env, jobject caller, jlong handle) {
    const auto* tc_response_handle = (const tc_response_handle_t*) handle;
    auto tc_response = tc_read_json_response(tc_response_handle);
    return tc_on_response(env, tc_response);
}

/** tc_destroy_json_response */
JNIEXPORT void JNICALL Java_io_tonlabs_client_NativeInterface_destroyJsonResponse(JNIEnv* env, jobject caller, jlong handle) {
    tc_destroy_json_response((const tc_response_handle_t*) handle);
}

JNIEXPORT void JNICALL Java_io_tonlabs_client_Native_jniCallback(JNIEnv* env, jobject caller, jlong handle) {

}

JNIEXPORT jstring JNICALL Java_io_tonlabs_client_NativeInterface_request(JNIEnv* env, jobject caller, jint context, jstring method, jstring params) {
    auto tc_response_handle = tc_json_request(context, get_tc_string(env, method), get_tc_string(env, params));
    auto tc_response = tc_read_json_response(tc_response_handle);
    tc_destroy_json_response(tc_response_handle);
    return tc_on_response(env, tc_response);
}

#ifdef __cplusplus
}
#endif
