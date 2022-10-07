---
description: How to connect to GraphQL API endpoint
---

# Connect to GraphQL API

{% hint style="warning" %}
If you received "Unauthorized access" error, please check that you specified the Evercloud access credentials correctly. Follow the Guide [https://docs.everos.dev/evernode-platform/products/evercloud/get-started](https://docs.everos.dev/evernode-platform/products/evercloud/get-started) for more info.&#x20;
{% endhint %}

### HTTPS

#### Without secret

{% tabs %}
{% tab title="Curl" %}
```bash
curl --location --request POST 'endpoint' \
--header 'Content-Type: application/json' \
--data-raw '{"query":"query($address: String!){\n  blockchain{\n    account(address:$address){\n      info{\n        balance(format:DEC)\n      }\n    }\n  }\n}","variables":{"address":"0:e17ac4e77f46626579c7c4fefe35286117384c5ccfc8745c9780cdf056c378bf"}}'
```
{% endtab %}

{% tab title="ever-sdk-js" %}
```javascript
const {TonClient} = require("@eversdk/core");
const {libNode} = require("@eversdk/lib-node");

TonClient.useBinaryLibrary(libNode)

const client = new TonClient({
    network: {
        endpoints: [
            "endpoint"
        ],
    },
});

(async () => {
    try {
        queryString = `
            query{
                blockchain{
                blocks(workchain:-1, last:1){
                    edges{
                    node{
                        hash
                        seq_no
                    }
                    }
                }
                }
            }
        `
        let {seq_no, hash} = (await client.net.query({ 
            "query": queryString }))
        .result.data.blockchain.blocks.edges[0].node;
        console.log("The last masterchain block seqNo is " + seq_no+ '\n' + "the hash is" + hash);
        client.close();
}
    catch (error) {
            console.error(error);
    }
}
)()
```
{% endtab %}

{% tab title="JS fetch" %}
```javascript
var myHeaders = new Headers();
myHeaders.append("Content-Type", "application/json");

var graphql = JSON.stringify({
  query: "query{\n  blockchain{\n    blocks(workchain:-1, last:1){\n      edges{\n        node{\n          hash\n          seq_no\n        }\n      }\n    }\n  }\n}",
  variables: {}
})
var requestOptions = {
  method: 'POST',
  headers: myHeaders,
  body: graphql,
  redirect: 'follow'
};

fetch("endpoint", requestOptions)
  .then(response => response.text())
  .then(result => console.log(result))
  .catch(error => console.log('error', error));
```
{% endtab %}

{% tab title="Postman" %}
```
URL: endpoint
Body: GraphQL
Query:

query{
  blockchain{
    blocks(workchain:-1, last:1){
      edges{
        node{
          hash
          seq_no
        }
      }
    }
  }
}
```
{% endtab %}
{% endtabs %}

#### With secret

{% tabs %}
{% tab title="Curl" %}
```bash
curl --location --request POST 'endpoint' \
--header 'Authorization: Basic OmM1NWY3Y2Q4YzZmZTRjNTBhMDRjOTM0ODE0NTg3OWRi' \
--header 'Content-Type: application/json' \
--data-raw '{"query":"query{\n  blockchain{\n    blocks(workchain:-1, last:1){\n      edges{\n        node{\n          hash\n          seq_no\n        }\n      }\n    }\n  }\n}","variables":{}}'
```
{% endtab %}

{% tab title="ever-sdk-js" %}
```javascript
WIP
Support of Project Secret is in development
```
{% endtab %}

{% tab title="JS fetch" %}
```javascript
var myHeaders = new Headers();
myHeaders.append("Authorization", "Basic OmM1NWY3Y2Q4YzZmZTRjNTBhMDRjOTM0ODE0NTg3OWRi");
myHeaders.append("Content-Type", "application/json");

var graphql = JSON.stringify({
  query: "query{\n  blockchain{\n    blocks(workchain:-1, last:1){\n      edges{\n        node{\n          hash\n          seq_no\n        }\n      }\n    }\n  }\n}",
  variables: {}
})
var requestOptions = {
  method: 'POST',
  headers: myHeaders,
  body: graphql,
  redirect: 'follow'
};

fetch("endpoint", requestOptions)
  .then(response => response.text())
  .then(result => console.log(result))
  .catch(error => console.log('error', error));
```
{% endtab %}

{% tab title="Postman" %}
```
URL: endpoint
Authorization: Basic Auth
Username: empty
Password: <Project Secret>
Body: GraphQL
Query:

query{
  blockchain{
    blocks(workchain:-1, last:1){
      edges{
        node{
          hash
          seq_no
        }
      }
    }
  }
}
```
{% endtab %}
{% endtabs %}

### WSS

#### Without secret

{% tabs %}
{% tab title="ever-sdk-js" %}
```javascript
const {TonClient} = require("@eversdk/core");
const {libNode} = require("@eversdk/lib-node");

TonClient.useBinaryLibrary(libNode)

const client = new TonClient({
    network: {
        endpoints: [
            "endpoint"
        ],
    },
});

async function _callback(response, responseType){
       /*
         * Where responseType:
         * 100 - GraphQL data received
         * 101 - GraphQL error received
         */

        if (responseType === 100) {
            if (response.result) {
                console.log("New block seq_no: "+ response.result.blocks.seq_no);

            }
        } else {
            console.log("Subscription failed with result: "+ JSON.stringify(response))
        }
    }

(async () => {
    try {
        subscriptionString = `
            subscription{
                blocks(filter:{
                workchain_id:{
                    eq:-1
                }
                }){
                seq_no
                id
                }
            }
        `
        const subscriptionHandler =  await client.net.subscribe({ 
            "subscription": subscriptionString }, _callback);        
        
        await new Promise(r => setTimeout(r, 10000));

        await client.net.unsubscribe(subscriptionHandler);

            
        client.close();
}
    catch (error) {
            console.error(error);
    }
}
)()
```
{% endtab %}

{% tab title="wscat" %}
```bash
wscat -c endpoint -s graphql-ws
{"id":"1","type":"start","payload":{"variables":{},"extensions":{},"operationName":null,"query":"subscription{\n  blocks(filter:{\n    workchain_id:{\n      eq:-1\n    }\n  }){\n    seq_no\n    id\n  }\n}"}}
```
{% endtab %}

{% tab title="Postman" %}
```json
URL: endpoint
Sec-WebSocket-Protocol: graphql-ws

message
{
  "id": "1",
  "type": "start",
  "payload": {
    "variables": {},
    "extensions": {},
    "operationName": null,
    "query": "subscription{\n  blocks(filter:{\n    workchain_id:{\n      eq:-1\n    }\n  }){\n    seq_no\n    id\n  }\n}"
  }
}
```
{% endtab %}
{% endtabs %}

In the next section find out how to work with GraphQL Web playground and easily explore blockchain data with it.
