package io.tonlabs.client;

/**
 * Abstract Java Native Interface
 * @author Martin Zeitler
 **/
abstract public class NativeInterface {

    /**
     * JNI tc_create_context
     * @return API context
     */
    public native int createContext();

    /**
     * JNI tc_destroy_context
     * @param context API context
     */
    public native void destroyContext(int context);

    /**
     * JNI tc_json_request
     * @param context int the API context
     * @param method String the requested method
     * @param params String request parameter as JSON string
     * @return long the response handle
     */
    public native long jsonRequest(int context, String method, String params);

    /**
     * JNI tc_read_json_response
     * @param handle long the response handle
     * @return String JSON
     */
    public native String readJsonResponse(long handle);

    /**
     * JNI tc_destroy_json_response
     * @param handle long the response handle
     */
    public native void destroyJsonResponse(long handle);

    /**
     * JNI Custom
     * @param context int the API context
     * @param method String the requested method
     * @param params String request parameter as JSON string
     * @return String the JSON response
     */
    public native String request(int context, String method, String params);
}
