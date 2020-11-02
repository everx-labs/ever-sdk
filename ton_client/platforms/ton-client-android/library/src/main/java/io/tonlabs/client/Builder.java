package io.tonlabs.client;

/**
 * TON Client Builder
 * @author Martin Zeitler
**/
public class Builder {

    /** API_BASE_URL */
    private String endpointUrl = null;

    /** Callback Listener */
    private TonClient.Callback listener = null;

    public Builder() {}

    public Builder setEndpointUrl(String summary) {
        this.endpointUrl = summary;
        return this;
    }

    public Builder setListener(TonClient.Callback listener) {
        this.listener = listener;
        return this;
    }

    @SuppressWarnings("rawtypes")
    public TonClient create() {
        if (endpointUrl == null) {endpointUrl = Constants.GRAPHQL_DEV;}
        return new TonClient(endpointUrl, listener);
    }
}
