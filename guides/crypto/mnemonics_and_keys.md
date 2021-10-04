# Mnemonics and keys

This section explains how to generate mnemonics, derive keys and get a key pair
- [Mnemonic generation](#mnemonic-generation)
- [Key pair generation from mnemonic](#key-pair-generation-from-mnemonic)
- [Key pair generation without mnemonic](#key-pair-generation-without-mnemonic)
- [Keys derivation](#keys-derivation)
  - [Master (root) key](#master-root-key)
  - [Derived key](#derived-key)
  - [Generate keys for signature](#generate-keys-for-signature)


> Check [crypto module](../../docs/mod_crypto.md) reference for more info: 

# Mnemonic generation

To generate a random mnemonic, use `mnemonic_from_random` function. Specify the dictionary, and a number of words (12 or 24).

    const SEED_PHRASE_WORD_COUNT = 12; //Mnemonic word count
    const SEED_PHRASE_DICTIONARY_ENGLISH = 1; //Dictionary identifier

    const { phrase } = await client.crypto.mnemonic_from_random({
        dictionary: SEED_PHRASE_DICTIONARY_ENGLISH,
        word_count: SEED_PHRASE_WORD_COUNT,
    });
    console.log(`Generated seed phrase "${phrase}"`);

Result:

    Generated seed phrase: "garden wedding range mixed during left powder grid modify safe recycle cup"

# Key pair generation from mnemonic

Here is the fast way to generate a key pair for a signature from a specified mnemonic and path with `mnemonic_derive_sign_keys` method. The specified path, dictionary and word count is compatible with Surf and tonos-cli:

    const HD_PATH = "m/44'/396'/0'/0/0";

    const keyPair = await client.crypto.mnemonic_derive_sign_keys({
        phrase,
        path: HD_PATH,
        dictionary: SEED_PHRASE_DICTIONARY_ENGLISH,
        word_count: SEED_PHRASE_WORD_COUNT,
    });
    console.log(`Generated key pair:`);
    console.log(keyPair);

Result:

    Generated key pair:
    {
      public: '4085d11b6d607c44ef0e8ddc535786af1a4b1f971e758206cd222ed3eba47d8b',
      secret: 'e90866b307ea6a72c216a34786762e648e9b382779fdfb88cf7b1e900a6bf0e2'
    }

See the [Keys derivation](#keys-derivation) section for more information about the algorithm that is used in `mnemonic_derive_sign_keys`.

# Key pair generation without mnemonic

Sometimes there is no need for mnemonic.

For example, if you generate a key pair for a server that does not need a readable form. Then you can easily generate such a pair with `generate_random_sign_keys` function:

    const simpleKeys = await client.crypto.generate_random_sign_keys();
    console.log(`key pair not from mnemonic:`);
    console.log(simpleKeys);

Result:

    key pair not from mnemonic:
    {
      public: 'de996e3004e2bc73b47e8a4fce665847194e2245ddbfc30d9ec2014913249f50',
      secret: 'a761156ff1ad497d4d52a32e32720cc3ef8b0d7c259f6d91b9f236d6288e12a3'
    }

Great! Now you can use this key pair in methods of contracts module, such as `abi.encode_message`, `abi.encode_message_body`, etc.

# Keys derivation

## Master (root) key

To derive a key within a specified path, first, you need to generate a seed from the given mnemonic and then get the extended master private key that will be the root for all the derived keys.

Both these operations can be performed with `hdkey_xprv_from_mnemonic` method:

    const hdk_root = await client.crypto.hdkey_xprv_from_mnemonic({
            dictionary: SEED_PHRASE_DICTIONARY_ENGLISH, // 1
            wordCount: SEED_PHRASE_WORD_COUNT, // 12
            phrase: seedPhrase
        });
    console.log(`\nSerialized extended master private key: \n${hdk_root}`);

Result:

    Serialized extended master private key: 
    xprv9s21ZrQH143K45hXeaopM1rAUJDszLAcwFkxrZ4njANoGhFPYFsB7rzspWC8wAnWoZ2bPia7covh3mVVboC2nEswu18iEHs5LjVknSWMR2w

## Derived key

Now you can derive the key within the specified path with `hdkey_derive_from_xprv_path` method. The result will be an extended derived private key.

    const HD_PATH = "m/44'/396'/0'/0/0";
    const extended_prkey = await client.crypto.hdkey_derive_from_xprv_path({
        xprv: hdk_root,
        path: HD_PATH,
    });
    console.log(`Serialized derived extended private key: \n${extended_prkey}`);

Result:

    Serialized derived extended private key: 
    xprvA45BBKdrZKobCbeFvC316LZ6AVDXbDn8Sa3btCMCcgTRM4CRxX4Tg3fk7sNNXPza9aMiS6mBMp7wfHdmT23bri6YgwHbTJgXqKnJNNHAw98

Now lets extract the private key itself from the extended key with function:

    const secret = await сlient.crypto.hdkey_secret_from_xprv(extended_prkey);
    console.log(`Derived private key: \n${secret}`);

Result:

    Derived private key: 
    e90866b307ea6a72c216a34786762e648e9b382779fdfb88cf7b1e900a6bf0e2

## Generate keys for signature

After we've got the derived private key we can generate a ed25519 key pair for signature:

    let tonosKeyPair2 = await сlient.crypto.nacl_sign_keypair_from_secret_key({ secret })

    if (tonosKeyPair2.secret.length > tonosKeyPair2.public.length) {
    tonosKeyPair2.secret = tonosKeyPair2.secret.substring(0, tonosKeyPair2.public.length);
    }
    console.log(`Key pair for signature:`);
    console.log(tonosKeyPair2);

Result:

    Key pair for signature:
    {
      public: '4085d11b6d607c44ef0e8ddc535786af1a4b1f971e758206cd222ed3eba47d8b',
      secret: 'e90866b307ea6a72c216a34786762e648e9b382779fdfb88cf7b1e900a6bf0e2'
    }

Great! Now you can use this key pair in methods of contracts module, such as `abi.encode_message`, `abi.encode_message_body`, etc.