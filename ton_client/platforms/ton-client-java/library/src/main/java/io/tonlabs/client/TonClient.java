package io.tonlabs.client;

import java.io.*;

/**
 * TON Client
 * It loads system-wide assembly from `java.library.path` or it falls back to extract and load it's own assembly.<br>
 * @see <a href="https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/jniTOC.html">JNI Specification</a>
 * @author Martin Zeitler
**/
public class TonClient extends NativeInterface {

    private static final String[] libNames = {"ton_client", "tonclientjni"};

    /** {@link TonClient} instance handle */
    static TonClient instance;

    /** {@link String} API endpointUrl */
    String endpointUrl;

    /** {@link TonClient.Callback} listener */
    TonClient.Callback listener;

    public interface Callback {
        void onSuccess(String value);
        void onFailure(String value);
    }

    static {

        for (String libName: libNames) {
            // Attempting to load system-installed native assembly
            try {

                System.loadLibrary(libName);
                System.out.println("> Loaded native assembly: " + libName + getSuffix());

            } catch (UnsatisfiedLinkError e) {

                // Falling back to self-supplied assembly
                loadAssembly(libName);

            } catch (SecurityException e) {
                System.out.println(e.getMessage());
            }
        }
    }

    /** Constructor */
    public TonClient() {

    }

    /**
     * Constructor
     * @param endpointUrl An API endpoint URL.
     * @param listener A {@link TonClient.Callback} listener
     */
    public TonClient(String endpointUrl, TonClient.Callback listener) {
        this.endpointUrl = endpointUrl;
        this.listener = listener;
    }

    /**
     * @param endpointUrl An API endpoint URL.
     * @param listener A {@link TonClient.Callback} listener
     * @return Singleton instance of {@link TonClient}
     */
    public static TonClient getInstance(String endpointUrl, TonClient.Callback listener) {
        if(instance == null) {instance = new TonClient(endpointUrl, listener);}
        return instance;
    }

    /**
     * @return instance of {@link Builder}
     */
    public Builder newBuilder() {
        return new Builder();
    }

    /*
     * Extracts and loads native assembly from jar file.<br/>
     * On Windows / VisualCpp the DLL fails to link.
     * @param void
     * @exception IOException
     * @exception SecurityException If a security manager exists and its <code>checkLink</code> method doesn't allow
     * loading of the specified dynamic library
     * @exception UnsatisfiedLinkError If either the filename is not an absolute path name, the native library is
     * not statically linked with the VM, or the library cannot be mapped to a native library image by the host system.
     */
    private static void loadAssembly(String libName) {
        String tmpFile;
        try {
            tmpFile = extractAssembly(libName);
            System.out.println("> Extracted native assembly: " + tmpFile);
            System.load(tmpFile);
            System.out.println("> Loaded native assembly: " + tmpFile);
        } catch (IOException | UnsatisfiedLinkError | SecurityException e) {
            System.out.println("> " + e.getClass().getSimpleName() + ": " + e.getMessage());
            // e.printStackTrace();
        }
    }

    /*
     * It extracts native assembly from the jar file.
     * @return String The path of the extracted library.
     */
    private static String extractAssembly(String libName) throws IOException {
        InputStream in = TonClient.class.getResourceAsStream("/" + libName + getSuffix());
        if(in != null) {
            File temp = new File(System.getProperty("java.io.tmpdir") + "/" + libName + getSuffix());
            FileOutputStream fos = new FileOutputStream(temp);
            byte[] buffer = new byte[1024];
            int read;
            while ((read = in.read(buffer)) != -1) {fos.write(buffer, 0, read);}
            fos.close();
            in.close();
            return temp.getAbsolutePath();
        } else {
            throw new FileNotFoundException(libName + getSuffix());
        }
    }

    /*
     * Platform Compatibility Helper
     * @return String The filename suffix of the library to extract & load.
     */
    private static String getSuffix() {
        String suffix = ".so";
        String os = System.getProperty("os.name").toLowerCase();
        if(os.toLowerCase().matches(".*windows.*")) {os = "win32";}
        if (os.equals("darwin")) {suffix = ".dynlib";}
        else if (os.equals("win32")) {suffix = ".dll";}
        return suffix;
    }

    public static void nativeLog(String logTag, String logMsg) {
        System.out.println("> " + logTag + ": " + logMsg);
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
}
