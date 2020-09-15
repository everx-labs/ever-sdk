/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crate::tests::*;
use super::*;

#[test]
fn parse_message() {
    let client = TestClient::new();

    let result: ResultOfParse = client.request(
        "boc.parse_message",
        ParamsOfParse {
            boc: String::from("te6ccgEBAQEAWAAAq2n+AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAE/zMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzSsG8DgAAAAAjuOu9NAL7BxYpA")
        }
    );

    assert_eq!(result.result["id"], "dfd47194f3058ee058bfbfad3ea40cbbd9ad17ca77cd0904d4d9f18a48c2fbca");
    assert_eq!(result.result["src"], "-1:0000000000000000000000000000000000000000000000000000000000000000");
    assert_eq!(result.result["dst"], "-1:3333333333333333333333333333333333333333333333333333333333333333");
}

#[test]
fn parse_account() {
    let client = TestClient::new();

    let result: ResultOfParse = client.request(
        "boc.parse_account",
        ParamsOfParse {
            boc: String::from("te6ccgECHQEAA/wAAnfAArtKDoOR5+qId/SCUGSSS9Qc4RD86X6TnTMjmZ4e+7EyOobmQvsHNngAAAg6t/34DgJWKJuuOehjU0ADAQFBlcBqp0PR+QAN1kt1SY8QavS350RCNNfeZ+ommI9hgd/gAgBToB6t2E3E7a7aW2YkvXv2hTmSWVRTvSYmCVdH4HjgZ4Z94AAAAAvsHNwwAib/APSkICLAAZL0oOGK7VNYMPShBgQBCvSkIPShBQAAAgEgCgcBAv8IAf5/Ie1E0CDXScIBn9P/0wD0Bfhqf/hh+Gb4Yo4b9AVt+GpwAYBA9A7yvdcL//hicPhjcPhmf/hh4tMAAY4SgQIA1xgg+QFY+EIg+GX5EPKo3iP4RSBukjBw3vhCuvLgZSHTP9MfNCD4I7zyuSL5ACD4SoEBAPQOIJEx3vLQZvgACQA2IPhKI8jLP1mBAQD0Q/hqXwTTHwHwAfhHbvJ8AgEgEQsCAVgPDAEJuOiY/FANAdb4QW6OEu1E0NP/0wD0Bfhqf/hh+Gb4Yt7RcG1vAvhKgQEA9IaVAdcLP3+TcHBw4pEgjjJfM8gizwv/Ic8LPzExAW8iIaQDWYAg9ENvAjQi+EqBAQD0fJUB1ws/f5NwcHDiAjUzMehfAyHA/w4AmI4uI9DTAfpAMDHIz4cgzo0EAAAAAAAAAAAAAAAAD3RMfijPFiFvIgLLH/QAyXH7AN4wwP+OEvhCyMv/+EbPCwD4SgH0AMntVN5/+GcBCbkWq+fwEAC2+EFujjbtRNAg10nCAZ/T/9MA9AX4an/4Yfhm+GKOG/QFbfhqcAGAQPQO8r3XC//4YnD4Y3D4Zn/4YeLe+Ebyc3H4ZtH4APhCyMv/+EbPCwD4SgH0AMntVH/4ZwIBIBUSAQm7Fe+TWBMBtvhBbo4S7UTQ0//TAPQF+Gp/+GH4Zvhi3vpA1w1/ldTR0NN/39cMAJXU0dDSAN/RVHEgyM+FgMoAc89AzgH6AoBrz0DJc/sA+EqBAQD0hpUB1ws/f5NwcHDikSAUAISOKCH4I7ubIvhKgQEA9Fsw+GreIvhKgQEA9HyVAdcLP3+TcHBw4gI1MzHoXwb4QsjL//hGzwsA+EoB9ADJ7VR/+GcCASAYFgEJuORhh1AXAL74QW6OEu1E0NP/0wD0Bfhqf/hh+Gb4Yt7U0fhFIG6SMHDe+EK68uBl+AD4QsjL//hGzwsA+EoB9ADJ7VT4DyD7BCDQ7R7tU/ACMPhCyMv/+EbPCwD4SgH0AMntVH/4ZwIC2hsZAQFIGgAs+ELIy//4Rs8LAPhKAfQAye1U+A/yAAEBSBwAWHAi0NYCMdIAMNwhxwDcIdcNH/K8UxHdwQQighD////9vLHyfAHwAfhHbvJ8")
        }
    );

    assert_eq!(result.result["id"], "0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13");
    assert_eq!(result.result["last_trans_lt"], "0x20eadff7e03");
    assert_eq!(result.result["balance"], "0x958a26eb8e7a18d");
}

#[test]
fn parse_transaction() {
    let client = TestClient::new();

    let result: ResultOfParse = client.request(
        "boc.parse_transaction",
        ParamsOfParse {
            boc: String::from("te6ccgECBwEAAZQAA7V75gA6WK5sEDTiHFGnH9ILOy2irjKLWTkWQMyMogsg40AAACDribjoE3gOAbYNpCaX4uLeXPQHt2Kw/Jp2OKkR2s+BASyeQM6wAAAg64IXyBX2DobAABRrMENIBQQBAhUEQojmJaAYazBCEQMCAFvAAAAAAAAAAAAAAAABLUUtpEnlC4z33SeGHxRhIq/htUa7i3D8ghbwxhQTn44EAJxC3UicQAAAAAAAAAAAdwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgnJAnYEvIQY6SnQKc3lXk6x1Z/lyplGFRbwAuNtVBi9EeceU3Ojl0F3EkRdylowY5x2qlgHNv4lNZUjhq0WqrLMNAQGgBgC3aADLL4ChL2HyLHwOLub5Mep87W3xdnMW8BpxKyVoGe3RPQAvmADpYrmwQNOIcUacf0gs7LaKuMotZORZAzIyiCyDjQ5iWgAGFFhgAAAEHXC9CwS+wdDGKTmMFkA=")
        }
    );

    assert_eq!(result.result["id"], "d6315dbb2a741a2765da250bea4a186adf942469369c703c57c2050e2d6e9fe3");
    assert_eq!(result.result["lt"], "0x20eb89b8e81");
    assert_eq!(result.result["now"], 1600186476);
}
