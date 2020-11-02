package io.tonlabs.client;

import android.util.Log;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;

/**
 * TON Client
 * @author Martin Zeitler
**/
public class TonClient extends NativeInterface {

    /** Log Tag */
    private static final String LOG_TAG = TonClient.class.getSimpleName();

    static final String[] libNames = {"tonclient",  "tonclientjni"};

    /** {@link TonClient} instance handle */
    static TonClient instance;

    /** {@link String} endpointUrl */
    String endpointUrl;

    /** {@link TonClient.Callback} listener */
    TonClient.Callback listener;

    public interface Callback {
        void onSuccess(String value);
        void onFailure(String value);
    }

    static {
        for (String libName: libNames) {
            try {

                // Attempting to load system-installed native assembly
                System.loadLibrary(libName);
                Log.d(LOG_TAG, "Loaded native assembly: " + libName);

            } catch (UnsatisfiedLinkError | SecurityException e) {
                Log.e(LOG_TAG, "" + e.getMessage());
            }
        }
    }

    /** Constructor */
    public TonClient() {

    }

    /** Constructor */
    public TonClient(@NonNull String endpointUrl, @Nullable TonClient.Callback listener) {
        this.endpointUrl = endpointUrl;
        this.listener = listener;
    }

    /** @return Singleton instance of {@link TonClient} */
    public static TonClient getInstance(@NonNull String endpointUrl, @Nullable TonClient.Callback listener) {
        if(instance == null) {instance = new TonClient(endpointUrl, listener);}
        return instance;
    }

    public Builder newBuilder() {
        return new Builder();
    }
    /**
     * JNI tc_create_context
     * @return API context
     */
    public int createContext() {
        return super.createContext();
    }

    /**
     * JNI tc_destroy_context
     * @param context API context
     */
    public void destroyContext(int context){
        super.destroyContext(context);
    }

    /**
     * JNI tc_json_request
     * @param context int the API context
     * @param method String the requested method
     * @param params String request parameter as JSON string
     * @return long the response handle
     */
    public long jsonRequest(int context, String method, String params) {
        return super.jsonRequest(context, method, params);
    }

    /**
     * JNI tc_read_json_response
     * @param handle long the response handle
     * @return String JSON
     */
    public String readJsonResponse(long handle) {
        return super.readJsonResponse(handle);
    }

    /**
     * JNI tc_destroy_json_response
     * @param handle long the response handle
     */
    public void destroyJsonResponse(long handle) {
        super.destroyJsonResponse(handle);
    }

    /**
     * JNI Custom
     * @param context int the API context
     * @param method String the requested method
     * @param params String request parameter as JSON string
     * @return String the JSON response
     */
    public String request(int context, String method, String params) {

        String json = super.request(context, method, params);

        if(method.equals("setup") && json.equals("null")) {return null;}

        if(method.equals("version")) {return json.replaceAll("^\"|\"$", "");}

        if(method.equals("tvm.get") && json.startsWith("{\"output\":[")) {
            return json.replaceAll("^\\{\"output\":|\\}$", "");
        }
        return json;
    }
}
