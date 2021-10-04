# Add Contract to your App

## Add Contract to your App

Define a contract in your application to start working with it

* [About contract artifacts](3_add_contract_to_your_app.md#about-contract-artifacts)
* [Create contract wrapper](3_add_contract_to_your_app.md#create-contract-wrapper)

## About contract artifacts

Make sure your [added SDK to your app](1_add_sdk_to_your_app.md) and [configured your Client](2_configure_sdk.md) before proceeding.

If you need to work with a previously deployed contract, you will need only its ABI and address. Get it in public repositories or ask the contract developer for it.

If you need to deploy a contract, then you will also need its tvc file. This file, along with ABI, is an artifact of contract compilation. Ask contract developer for this file.

If you plan to develop and compile the contracts yourself then these docs will help you:

* [Compiling contract with one command with tondev](https://github.com/tonlabs/tondev#compile)
* [Solidity Compiler](https://docs.ton.dev/86757ecb2/p/950f8a-write-smart-contract-in-solidity)
* [Public TON Labs repository with contracts](https://github.com/tonlabs/ton-labs-contracts)

## Create contract wrapper

Use TONDEV tool to [generate contract wrapper](https://github.com/tonlabs/tondev#create-contract-js-wrapper) that will have the following structure:

```text
export type Contract = {
/**
 * ABI of smart contract
 */
abi: AbiContract,
/**
 * Compiled artifact of the smart contract converted to base64.
 * This field contains BOC with code and initial data (init state).
 * If it is missing, then application can't deploy account of this contracts.
 */
tvc?: string,
```

}

If you have tvc file, `TONDEV` will convert it into base64 that is suitable for SDK. If you don't have tvc, then this wrapper will be only useful for interaction with an already deployed contract.

Run this command:

```text
tondev js wrap contractName.abi.json
```

The result name of the wrapper will be "ContractName\|\|"Contract".js".

The result file will look like this:

```text
module.exports = {
HelloContract: {
    abi: {
        "ABI version": 2,
        "header": ["time", "expire"],
        "functions": [
            {
                "name": "constructor",
                "inputs": [],
                "outputs": [],
            },
            {
                "name": "touch",
                "inputs": [],
                "outputs": [],
            },
            {
                "name": "getTimestamp",
                "inputs": [],
                "outputs": [
                    {
                        "name": "value0",
                        "type": "uint256",
                    },
                ],
            },
        ],
        "data": [],
        "events": [],
    },
    tvc: "te6ccgECEgEAAisAAgE0AwEBAcACAEPQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgAib/APSkICLAAZL0oOGK7VNYMPShCAQBCvSkIPShBQIDzsAHBgAv12omhp/+mf6YBrhf/8NT/8MPwzfDH8MUAC/3whZGX//CHnhZ/8I2eFgHwlAOX/5PaqQCASALCQH+/38h7UTQINdJwgGOFNP/0z/TANcL//hqf/hh+Gb4Y/hijhv0BXD4anABgED0DvK91wv/+GJw+GNw+GZ/+GHi0wABn4ECANcYIPkBWPhC+RDyqN7TPwGOHvhDIbkgnzAg+COBA+iogggbd0Cgud6S+GPggDTyNNjTHwH4I7zyuQoAONMfIcEDIoIQ/////byxkvI84AHwAfhHbpLyPN4CASANDACzvUWq+f/CC3Rx52omgQa6ThAMcKaf/pn+mAa4X//DU//DD8M3wx/DFHDfoCuHw1OADAIHoHeV7rhf/8MTh8Mbh8Mz/8MPFvfCN5Obj8M2j8AHwR/DV4Ab/8M8AgEgDw4AL7tzEuRfhBbpLwBN7R+AD4I/hq8AN/+GeAIBIBEQAIO586yQfwgt0l4Am9o/CUQ4H/HEZHoaYD9IBgY5GfDkGdAMGegZ8DnwOfJPzrJBxDnhf/kuP2Abxhgf8l4Ae8//DPAAatxwItDWAjHSADDcIccAkOAh1w0fkvI84VMRkOHBAyKCEP////28sZLyPOAB8AH4R26S8jze",
}
};
```

**You're all done!**

Find out how to [deploy](../work_with_contracts/1_deploy.md) and [run](../work_with_contracts/2_run_onchain.md) your contract in the next sections.

