import io.tonlabs.client.Constants;
import io.tonlabs.client.TonClient;

/**
 * TON Client Test App
 * @author Martin Zeitler
**/
public class Main {

    static TonClient client;
    static int context = 0;

    public static void main(String[] args) {

        client = new TonClient();
        context = client.createContext();
        String response = client.request(context, Constants.CORE_API_VERSION, "");
        client.destroyContext(context);
        System.out.println(response);
    }
}
