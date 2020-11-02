package io.tonlabs.client

import android.os.Bundle
import android.util.Log

import androidx.appcompat.app.AppCompatActivity
import androidx.navigation.findNavController
import androidx.navigation.ui.AppBarConfiguration
import androidx.navigation.ui.setupActionBarWithNavController
import androidx.navigation.ui.setupWithNavController

import com.google.android.material.bottomnavigation.BottomNavigationView

import io.tonlabs.client.kotlin.R

/**
 * TON Client Kotlin Integration
 * @author Martin Zeitler
 */
class MainActivity : AppCompatActivity(), TonClient.Callback {

    /** Log Tag  */
    private val LOG_TAG = TonClient::class.java.simpleName

    private lateinit var client: TonClient

    private var contextId = 0

    override fun onCreate(savedInstanceState: Bundle?) {

        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        // Passing each menu ID as a set of Ids because each menu should be considered as top level destinations.
        val navView: BottomNavigationView = findViewById(R.id.nav_view)
        val navController = findNavController(R.id.nav_host_fragment)
        val appBarConfiguration = AppBarConfiguration(setOf(R.id.navigation_home))
        setupActionBarWithNavController(navController, appBarConfiguration)
        navView.setupWithNavController(navController)

        this.createClient()
        this.selfTest()
    }

    protected fun createClient() {
        this.client = TonClient().newBuilder().setEndpointUrl(Constants.GRAPHQL_DEV).setListener(this@MainActivity).create()
        this.contextId = client.createContext()
    }

    /** https://github.com/tonlabs/TON-SDK/wiki/Core-Library-JSON-API  */
    protected fun selfTest() {

        val codeBase64 = resources.getString(R.string.code_base64)
        val dataBase64 = resources.getString(R.string.data_base64)
        var response: String?
        var params = ""

        with(client) {

            response = this.request(contextId, "version", params)
            Log.d(LOG_TAG, "" + response)

            params = "{\"servers\": [\"https://" + Constants.GRAPHQL_DEV + "/graphql\"]}"
            response = this.request(contextId, "setup", params)
            Log.d(LOG_TAG, "" + response)

            params = "{\"functionName\": \"participant_list\", \"codeBase64\": \"$codeBase64\", \"dataBase64\": \"$dataBase64\"}"
            response = this.request(contextId, "tvm.get", params)
            Log.d(LOG_TAG, "" + response)
        }
    }

    override fun onDestroy() {
        if (contextId != 0) {
            client.destroyContext(contextId)
        }
        super.onDestroy()
    }

    override fun onSuccess(value: String) {
        Log.e(LOG_TAG, "" + value)
    }

    override fun onFailure(value: String) {
        Log.d(LOG_TAG, "" + value)
    }
}
