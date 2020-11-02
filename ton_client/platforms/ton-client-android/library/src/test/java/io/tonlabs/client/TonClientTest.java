package io.tonlabs.client;

import android.util.Log;

import org.junit.AfterClass;
import org.junit.BeforeClass;
import org.junit.FixMethodOrder;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;
import static org.junit.Assert.*;

/**
 * Android Unit Tests
 * @author Martin Zeitler
 */
@FixMethodOrder
@RunWith(JUnit4.class)
public class TonClientTest {

    private static final String LOG_TAG = TonClientTest.class.getSimpleName();

    private static TonClient client;

    private static int contextId;

    @BeforeClass
    public static void before() {
        client = TonClient.getInstance(Constants.GRAPHQL_TEST, null);
        contextId = client.createContext();
    }

    @Test
    public void testApiVersion() {
        String response = client.request(contextId, "version", "");
        Log.d(LOG_TAG, response);
        assertNotNull(response);
    }

    @Test
    public void testApiSetup() {
        String response = client.request(contextId, "setup", "{\"servers\":[\"" + Constants.GRAPHQL_DEV + "\"]}");
        Log.d(LOG_TAG, response);
        assertNotNull(response);
    }

    @AfterClass
    public static void after() {
        client.destroyContext(contextId);
        contextId = 0;
    }
}