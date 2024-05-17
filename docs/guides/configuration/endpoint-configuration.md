# Endpoint Configuration

## Create TONClient

`TONClient` is the main class of Ever SDK Library. To start using library one needs to create and setup a TONClient instance.

The simplest initialization code can look like this: just specify the endpoint.

Other parameters are used by default. See the reference below for more info.&#x20;

```javascript
const client = new TonClient({
network: { 
    endpoints: [
        'your-endpoint-here'
    ] 
    } 
});
```

## How to get my endpoint?

### Local node&#x20;

If you want to work with [local blockchain Evernode-SE](https://github.com/everx-labs/evernode-se), specify [http://localhost](http://localhost) in the `endpoints`

```javascript
const client = new TonClient({
network: { 
    endpoints: [
        'http://localhost'
    ] 
    } 
});
```

### Evercloud

If you don't want to manage your own infrastructure -  get your Evercloud endpoints **for free** to Mainnet and Devnet and configure security settings here [https://docs.everplatform.dev/products/evercloud/get-started](https://docs.everplatform.dev/products/evercloud/get-started).

Check the full list of [supported networks](https://docs.everplatform.dev/reference/graphql-api/networks).&#x20;

```javascript
const client = new TonClient({
network: { 
    endpoints: [
        'http://mainnet.evercloud.dev/your-project-id-here/graphql'
    ] 
    // access_key: "your-secret-here(optional, if you enabled "secret required" in your project)"
    } 
});
```

### Self-hosted dedicated node

If you want to run your own dedicated node yourself - see the[ Evernode-DS documentation ](https://docs.everplatform.dev/products/dapp-server-ds)how to run your dedicated node.

### Dedicated Evercloud

If you want your dedicated set of nodes to be run by Evernode Platform team, check this page [https://docs.everplatform.dev/products/dedicated-cloud-node](https://docs.everplatform.dev/products/dedicated-cloud-node).&#x20;

## Multiple endpoints configuration

If you have multiple endpoints in the same network, you can specify them all.

Library will automatically perform balancing based on endpoint health checks and availability.

```javascript
const client = new TonClient({
network: { 
    endpoints: [
        'ENDPOINT_URL1', 
        'ENDPOINT_URL2', 
        'ENDPOINT_URL3'
    ] 
    } 
});
```

You can also configure the message broadcast - how many nodes you want your message to be sent (it may improve delivery rate) like this.&#x20;

```javascript
const client = new TonClient({
network: { 
    endpoints: [
        'ENDPOINT_URL1', 
        'ENDPOINT_URL2', 
        'ENDPOINT_URL3'
    ] 
    sending_endpoint_count: 3
    } 
});
```
