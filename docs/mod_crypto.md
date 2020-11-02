# Module crypto

 Crypto functions.
## Functions
[factorize](#factorize) – Performs prime factorization – decomposition of a composite number into a product of smaller prime integers (factors). See [https://en.wikipedia.org/wiki/Integer_factorization]

[modular_power](#modular_power) – Performs modular exponentiation for big integers (`base`^`exponent` mod `modulus`). See [https://en.wikipedia.org/wiki/Modular_exponentiation]

[ton_crc16](#ton_crc16) – Calculates CRC16 using TON algorithm.

[generate_random_bytes](#generate_random_bytes) – Generates random byte array of the specified length and returns it in `base64` format

[convert_public_key_to_ton_safe_format](#convert_public_key_to_ton_safe_format) – Converts public key to ton safe_format

[generate_random_sign_keys](#generate_random_sign_keys) – Generates random ed25519 key pair.

[sign](#sign) – Signs a data using the provided keys.

[verify_signature](#verify_signature) – Verifies signed data using the provided public key. Raises error if verification is failed.

[sha256](#sha256) – Calculates SHA256 hash of the specified data.

[sha512](#sha512) – Calculates SHA512 hash of the specified data.

[scrypt](#scrypt) – Derives key from `password` and `key` using `scrypt` algorithm. See [https://en.wikipedia.org/wiki/Scrypt].

[nacl_sign_keypair_from_secret_key](#nacl_sign_keypair_from_secret_key) – Generates a key pair for signing from the secret key

[nacl_sign](#nacl_sign) – Signs data using the signer's secret key.

[nacl_sign_open](#nacl_sign_open)

[nacl_sign_detached](#nacl_sign_detached)

[nacl_box_keypair](#nacl_box_keypair)

[nacl_box_keypair_from_secret_key](#nacl_box_keypair_from_secret_key) – Generates key pair from a secret key

[nacl_box](#nacl_box) – Public key authenticated encryption

[nacl_box_open](#nacl_box_open) – Decrypt and verify the cipher text using the recievers secret key, the senders public key, and the nonce.

[nacl_secret_box](#nacl_secret_box) – Encrypt and authenticate message using nonce and secret key.

[nacl_secret_box_open](#nacl_secret_box_open) – Decrypts and verifies cipher text using `nonce` and secret `key`.

[mnemonic_words](#mnemonic_words) – Prints the list of words from the specified dictionary

[mnemonic_from_random](#mnemonic_from_random) – Generates a random mnemonic from the specified dictionary and word count

[mnemonic_from_entropy](#mnemonic_from_entropy) – Generates mnemonic from pre-generated entropy

[mnemonic_verify](#mnemonic_verify) – The phrase supplied will be checked for word length and validated according to the checksum specified in BIP0039.

[mnemonic_derive_sign_keys](#mnemonic_derive_sign_keys) – Validates the seed phrase, generates master key and then derives the key pair from the master key and the specified path

[hdkey_xprv_from_mnemonic](#hdkey_xprv_from_mnemonic) – Generates an extended master private key that will be the root for all the derived keys

[hdkey_derive_from_xprv](#hdkey_derive_from_xprv) – Returns extended private key derived from the specified extended private key and child index

[hdkey_derive_from_xprv_path](#hdkey_derive_from_xprv_path) – Derives the exented private key from the specified key and path

[hdkey_secret_from_xprv](#hdkey_secret_from_xprv) – Extracts the private key from the serialized extended private key

[hdkey_public_from_xprv](#hdkey_public_from_xprv) – Extracts the public key from the serialized extended private key

## Types
[SigningBoxHandle](#SigningBoxHandle)

[ParamsOfFactorize](#ParamsOfFactorize)

[ResultOfFactorize](#ResultOfFactorize)

[ParamsOfModularPower](#ParamsOfModularPower)

[ResultOfModularPower](#ResultOfModularPower)

[ParamsOfTonCrc16](#ParamsOfTonCrc16)

[ResultOfTonCrc16](#ResultOfTonCrc16)

[ParamsOfGenerateRandomBytes](#ParamsOfGenerateRandomBytes)

[ResultOfGenerateRandomBytes](#ResultOfGenerateRandomBytes)

[ParamsOfConvertPublicKeyToTonSafeFormat](#ParamsOfConvertPublicKeyToTonSafeFormat)

[ResultOfConvertPublicKeyToTonSafeFormat](#ResultOfConvertPublicKeyToTonSafeFormat)

[KeyPair](#KeyPair)

[ParamsOfSign](#ParamsOfSign)

[ResultOfSign](#ResultOfSign)

[ParamsOfVerifySignature](#ParamsOfVerifySignature)

[ResultOfVerifySignature](#ResultOfVerifySignature)

[ParamsOfHash](#ParamsOfHash)

[ResultOfHash](#ResultOfHash)

[ParamsOfScrypt](#ParamsOfScrypt)

[ResultOfScrypt](#ResultOfScrypt)

[ParamsOfNaclSignKeyPairFromSecret](#ParamsOfNaclSignKeyPairFromSecret)

[ParamsOfNaclSign](#ParamsOfNaclSign)

[ResultOfNaclSign](#ResultOfNaclSign)

[ParamsOfNaclSignOpen](#ParamsOfNaclSignOpen)

[ResultOfNaclSignOpen](#ResultOfNaclSignOpen)

[ResultOfNaclSignDetached](#ResultOfNaclSignDetached)

[ParamsOfNaclBoxKeyPairFromSecret](#ParamsOfNaclBoxKeyPairFromSecret)

[ParamsOfNaclBox](#ParamsOfNaclBox)

[ResultOfNaclBox](#ResultOfNaclBox)

[ParamsOfNaclBoxOpen](#ParamsOfNaclBoxOpen)

[ResultOfNaclBoxOpen](#ResultOfNaclBoxOpen)

[ParamsOfNaclSecretBox](#ParamsOfNaclSecretBox)

[ParamsOfNaclSecretBoxOpen](#ParamsOfNaclSecretBoxOpen)

[ParamsOfMnemonicWords](#ParamsOfMnemonicWords)

[ResultOfMnemonicWords](#ResultOfMnemonicWords)

[ParamsOfMnemonicFromRandom](#ParamsOfMnemonicFromRandom)

[ResultOfMnemonicFromRandom](#ResultOfMnemonicFromRandom)

[ParamsOfMnemonicFromEntropy](#ParamsOfMnemonicFromEntropy)

[ResultOfMnemonicFromEntropy](#ResultOfMnemonicFromEntropy)

[ParamsOfMnemonicVerify](#ParamsOfMnemonicVerify)

[ResultOfMnemonicVerify](#ResultOfMnemonicVerify)

[ParamsOfMnemonicDeriveSignKeys](#ParamsOfMnemonicDeriveSignKeys)

[ParamsOfHDKeyXPrvFromMnemonic](#ParamsOfHDKeyXPrvFromMnemonic)

[ResultOfHDKeyXPrvFromMnemonic](#ResultOfHDKeyXPrvFromMnemonic)

[ParamsOfHDKeyDeriveFromXPrv](#ParamsOfHDKeyDeriveFromXPrv)

[ResultOfHDKeyDeriveFromXPrv](#ResultOfHDKeyDeriveFromXPrv)

[ParamsOfHDKeyDeriveFromXPrvPath](#ParamsOfHDKeyDeriveFromXPrvPath)

[ResultOfHDKeyDeriveFromXPrvPath](#ResultOfHDKeyDeriveFromXPrvPath)

[ParamsOfHDKeySecretFromXPrv](#ParamsOfHDKeySecretFromXPrv)

[ResultOfHDKeySecretFromXPrv](#ResultOfHDKeySecretFromXPrv)

[ParamsOfHDKeyPublicFromXPrv](#ParamsOfHDKeyPublicFromXPrv)

[ResultOfHDKeyPublicFromXPrv](#ResultOfHDKeyPublicFromXPrv)


# Functions
## factorize

Performs prime factorization – decomposition of a composite number into a product of smaller prime integers (factors). See [https://en.wikipedia.org/wiki/Integer_factorization]

```ts
type ParamsOfFactorize = {
    composite: string
};

type ResultOfFactorize = {
    factors: string[]
};

function factorize(
    params: ParamsOfFactorize,
): Promise<ResultOfFactorize>;
```
### Parameters
- `composite`: _string_ – Hexadecimal representation of u64 composite number.
### Result

- `factors`: _string[]_ – Two factors of composite or empty if composite can't be factorized.


## modular_power

Performs modular exponentiation for big integers (`base`^`exponent` mod `modulus`). See [https://en.wikipedia.org/wiki/Modular_exponentiation]

```ts
type ParamsOfModularPower = {
    base: string,
    exponent: string,
    modulus: string
};

type ResultOfModularPower = {
    modular_power: string
};

function modular_power(
    params: ParamsOfModularPower,
): Promise<ResultOfModularPower>;
```
### Parameters
- `base`: _string_ – `base` argument of calculation.
- `exponent`: _string_ – `exponent` argument of calculation.
- `modulus`: _string_ – `modulus` argument of calculation.
### Result

- `modular_power`: _string_ – Result of modular exponentiation


## ton_crc16

Calculates CRC16 using TON algorithm.

```ts
type ParamsOfTonCrc16 = {
    data: string
};

type ResultOfTonCrc16 = {
    crc: number
};

function ton_crc16(
    params: ParamsOfTonCrc16,
): Promise<ResultOfTonCrc16>;
```
### Parameters
- `data`: _string_ – Input data for CRC calculation. Encoded with `base64`.
### Result

- `crc`: _number_ – Calculated CRC for input data.


## generate_random_bytes

Generates random byte array of the specified length and returns it in `base64` format

```ts
type ParamsOfGenerateRandomBytes = {
    length: number
};

type ResultOfGenerateRandomBytes = {
    bytes: string
};

function generate_random_bytes(
    params: ParamsOfGenerateRandomBytes,
): Promise<ResultOfGenerateRandomBytes>;
```
### Parameters
- `length`: _number_ – Size of random byte array.
### Result

- `bytes`: _string_ – Generated bytes encoded in `base64`.


## convert_public_key_to_ton_safe_format

Converts public key to ton safe_format

```ts
type ParamsOfConvertPublicKeyToTonSafeFormat = {
    public_key: string
};

type ResultOfConvertPublicKeyToTonSafeFormat = {
    ton_public_key: string
};

function convert_public_key_to_ton_safe_format(
    params: ParamsOfConvertPublicKeyToTonSafeFormat,
): Promise<ResultOfConvertPublicKeyToTonSafeFormat>;
```
### Parameters
- `public_key`: _string_ – Public key - 64 symbols hex string
### Result

- `ton_public_key`: _string_ – Public key represented in TON safe format.


## generate_random_sign_keys

Generates random ed25519 key pair.

```ts
type KeyPair = {
    public: string,
    secret: string
};

function generate_random_sign_keys(): Promise<KeyPair>;
```
### Result

- `public`: _string_ – Public key - 64 symbols hex string
- `secret`: _string_ – Private key - u64 symbols hex string


## sign

Signs a data using the provided keys.

```ts
type ParamsOfSign = {
    unsigned: string,
    keys: KeyPair
};

type ResultOfSign = {
    signed: string,
    signature: string
};

function sign(
    params: ParamsOfSign,
): Promise<ResultOfSign>;
```
### Parameters
- `unsigned`: _string_ – Data that must be signed encoded in `base64`.
- `keys`: _[KeyPair](mod_crypto.md#KeyPair)_ – Sign keys.
### Result

- `signed`: _string_ – Signed data combined with signature encoded in `base64`.
- `signature`: _string_ – Signature encoded in `hex`.


## verify_signature

Verifies signed data using the provided public key. Raises error if verification is failed.

```ts
type ParamsOfVerifySignature = {
    signed: string,
    public: string
};

type ResultOfVerifySignature = {
    unsigned: string
};

function verify_signature(
    params: ParamsOfVerifySignature,
): Promise<ResultOfVerifySignature>;
```
### Parameters
- `signed`: _string_ – Signed data that must be verified encoded in `base64`.
- `public`: _string_ – Signer's public key - 64 symbols hex string
### Result

- `unsigned`: _string_ – Unsigned data encoded in `base64`.


## sha256

Calculates SHA256 hash of the specified data.

```ts
type ParamsOfHash = {
    data: string
};

type ResultOfHash = {
    hash: string
};

function sha256(
    params: ParamsOfHash,
): Promise<ResultOfHash>;
```
### Parameters
- `data`: _string_ – Input data for hash calculation. Encoded with `base64`.
### Result

- `hash`: _string_ – Hash of input `data`. Encoded with 'hex'.


## sha512

Calculates SHA512 hash of the specified data.

```ts
type ParamsOfHash = {
    data: string
};

type ResultOfHash = {
    hash: string
};

function sha512(
    params: ParamsOfHash,
): Promise<ResultOfHash>;
```
### Parameters
- `data`: _string_ – Input data for hash calculation. Encoded with `base64`.
### Result

- `hash`: _string_ – Hash of input `data`. Encoded with 'hex'.


## scrypt

Derives key from `password` and `key` using `scrypt` algorithm. See [https://en.wikipedia.org/wiki/Scrypt].

# Arguments
- `log_n` - The log2 of the Scrypt parameter `N`
- `r` - The Scrypt parameter `r`
- `p` - The Scrypt parameter `p`
# Conditions
- `log_n` must be less than `64`
- `r` must be greater than `0` and less than or equal to `4294967295`
- `p` must be greater than `0` and less than `4294967295`
# Recommended values sufficient for most use-cases
- `log_n = 15` (`n = 32768`)
- `r = 8`
- `p = 1`

```ts
type ParamsOfScrypt = {
    password: string,
    salt: string,
    log_n: number,
    r: number,
    p: number,
    dk_len: number
};

type ResultOfScrypt = {
    key: string
};

function scrypt(
    params: ParamsOfScrypt,
): Promise<ResultOfScrypt>;
```
### Parameters
- `password`: _string_ – The password bytes to be hashed. Must be encoded with `base64`.
- `salt`: _string_ – A salt bytes that modifies the hash to protect against Rainbow table attacks. Must be encoded with `base64`.
- `log_n`: _number_ – CPU/memory cost parameter
- `r`: _number_ – The block size parameter, which fine-tunes sequential memory read size and performance.
- `p`: _number_ – Parallelization parameter.
- `dk_len`: _number_ – Intended output length in octets of the derived key.
### Result

- `key`: _string_ – Derived key. Encoded with `hex`.


## nacl_sign_keypair_from_secret_key

Generates a key pair for signing from the secret key

```ts
type ParamsOfNaclSignKeyPairFromSecret = {
    secret: string
};

type KeyPair = {
    public: string,
    secret: string
};

function nacl_sign_keypair_from_secret_key(
    params: ParamsOfNaclSignKeyPairFromSecret,
): Promise<KeyPair>;
```
### Parameters
- `secret`: _string_ – Secret key - unprefixed 0-padded to 64 symbols hex string
### Result

- `public`: _string_ – Public key - 64 symbols hex string
- `secret`: _string_ – Private key - u64 symbols hex string


## nacl_sign

Signs data using the signer's secret key.

```ts
type ParamsOfNaclSign = {
    unsigned: string,
    secret: string
};

type ResultOfNaclSign = {
    signed: string
};

function nacl_sign(
    params: ParamsOfNaclSign,
): Promise<ResultOfNaclSign>;
```
### Parameters
- `unsigned`: _string_ – Data that must be signed encoded in `base64`.
- `secret`: _string_ – Signer's secret key - unprefixed 0-padded to 64 symbols hex string
### Result

- `signed`: _string_ – Signed data, encoded in `base64`.


## nacl_sign_open

```ts
type ParamsOfNaclSignOpen = {
    signed: string,
    public: string
};

type ResultOfNaclSignOpen = {
    unsigned: string
};

function nacl_sign_open(
    params: ParamsOfNaclSignOpen,
): Promise<ResultOfNaclSignOpen>;
```
### Parameters
- `signed`: _string_ – Signed data that must be unsigned. Encoded with `base64`.
- `public`: _string_ – Signer's public key - unprefixed 0-padded to 64 symbols hex string
### Result

- `unsigned`: _string_ – Unsigned data, encoded in `base64`.


## nacl_sign_detached

```ts
type ParamsOfNaclSign = {
    unsigned: string,
    secret: string
};

type ResultOfNaclSignDetached = {
    signature: string
};

function nacl_sign_detached(
    params: ParamsOfNaclSign,
): Promise<ResultOfNaclSignDetached>;
```
### Parameters
- `unsigned`: _string_ – Data that must be signed encoded in `base64`.
- `secret`: _string_ – Signer's secret key - unprefixed 0-padded to 64 symbols hex string
### Result

- `signature`: _string_ – Signature encoded in `hex`.


## nacl_box_keypair

```ts
type KeyPair = {
    public: string,
    secret: string
};

function nacl_box_keypair(): Promise<KeyPair>;
```
### Result

- `public`: _string_ – Public key - 64 symbols hex string
- `secret`: _string_ – Private key - u64 symbols hex string


## nacl_box_keypair_from_secret_key

Generates key pair from a secret key

```ts
type ParamsOfNaclBoxKeyPairFromSecret = {
    secret: string
};

type KeyPair = {
    public: string,
    secret: string
};

function nacl_box_keypair_from_secret_key(
    params: ParamsOfNaclBoxKeyPairFromSecret,
): Promise<KeyPair>;
```
### Parameters
- `secret`: _string_ – Secret key - unprefixed 0-padded to 64 symbols hex string
### Result

- `public`: _string_ – Public key - 64 symbols hex string
- `secret`: _string_ – Private key - u64 symbols hex string


## nacl_box

Public key authenticated encryption

Encrypt and authenticate a message using the senders secret key, the recievers public
key, and a nonce.

```ts
type ParamsOfNaclBox = {
    decrypted: string,
    nonce: string,
    their_public: string,
    secret: string
};

type ResultOfNaclBox = {
    encrypted: string
};

function nacl_box(
    params: ParamsOfNaclBox,
): Promise<ResultOfNaclBox>;
```
### Parameters
- `decrypted`: _string_ – Data that must be encrypted encoded in `base64`.
- `nonce`: _string_ – Nonce, encoded in `hex`
- `their_public`: _string_ – Receiver's public key - unprefixed 0-padded to 64 symbols hex string
- `secret`: _string_ – Sender's private key - unprefixed 0-padded to 64 symbols hex string
### Result

- `encrypted`: _string_ – Encrypted data encoded in `base64`.


## nacl_box_open

Decrypt and verify the cipher text using the recievers secret key, the senders public key, and the nonce.

```ts
type ParamsOfNaclBoxOpen = {
    encrypted: string,
    nonce: string,
    their_public: string,
    secret: string
};

type ResultOfNaclBoxOpen = {
    decrypted: string
};

function nacl_box_open(
    params: ParamsOfNaclBoxOpen,
): Promise<ResultOfNaclBoxOpen>;
```
### Parameters
- `encrypted`: _string_ – Data that must be decrypted. Encoded with `base64`.
- `nonce`: _string_
- `their_public`: _string_ – Sender's public key - unprefixed 0-padded to 64 symbols hex string
- `secret`: _string_ – Receiver's private key - unprefixed 0-padded to 64 symbols hex string
### Result

- `decrypted`: _string_ – Decrypted data encoded in `base64`.


## nacl_secret_box

Encrypt and authenticate message using nonce and secret key.

```ts
type ParamsOfNaclSecretBox = {
    decrypted: string,
    nonce: string,
    key: string
};

type ResultOfNaclBox = {
    encrypted: string
};

function nacl_secret_box(
    params: ParamsOfNaclSecretBox,
): Promise<ResultOfNaclBox>;
```
### Parameters
- `decrypted`: _string_ – Data that must be encrypted. Encoded with `base64`.
- `nonce`: _string_ – Nonce in `hex`
- `key`: _string_ – Secret key - unprefixed 0-padded to 64 symbols hex string
### Result

- `encrypted`: _string_ – Encrypted data encoded in `base64`.


## nacl_secret_box_open

Decrypts and verifies cipher text using `nonce` and secret `key`.

```ts
type ParamsOfNaclSecretBoxOpen = {
    encrypted: string,
    nonce: string,
    key: string
};

type ResultOfNaclBoxOpen = {
    decrypted: string
};

function nacl_secret_box_open(
    params: ParamsOfNaclSecretBoxOpen,
): Promise<ResultOfNaclBoxOpen>;
```
### Parameters
- `encrypted`: _string_ – Data that must be decrypted. Encoded with `base64`.
- `nonce`: _string_ – Nonce in `hex`
- `key`: _string_ – Public key - unprefixed 0-padded to 64 symbols hex string
### Result

- `decrypted`: _string_ – Decrypted data encoded in `base64`.


## mnemonic_words

Prints the list of words from the specified dictionary

```ts
type ParamsOfMnemonicWords = {
    dictionary?: number
};

type ResultOfMnemonicWords = {
    words: string
};

function mnemonic_words(
    params: ParamsOfMnemonicWords,
): Promise<ResultOfMnemonicWords>;
```
### Parameters
- `dictionary`?: _number_ – Dictionary identifier
### Result

- `words`: _string_ – The list of mnemonic words


## mnemonic_from_random

Generates a random mnemonic from the specified dictionary and word count

```ts
type ParamsOfMnemonicFromRandom = {
    dictionary?: number,
    word_count?: number
};

type ResultOfMnemonicFromRandom = {
    phrase: string
};

function mnemonic_from_random(
    params: ParamsOfMnemonicFromRandom,
): Promise<ResultOfMnemonicFromRandom>;
```
### Parameters
- `dictionary`?: _number_ – Dictionary identifier
- `word_count`?: _number_ – Mnemonic word count
### Result

- `phrase`: _string_ – String of mnemonic words


## mnemonic_from_entropy

Generates mnemonic from pre-generated entropy

```ts
type ParamsOfMnemonicFromEntropy = {
    entropy: string,
    dictionary?: number,
    word_count?: number
};

type ResultOfMnemonicFromEntropy = {
    phrase: string
};

function mnemonic_from_entropy(
    params: ParamsOfMnemonicFromEntropy,
): Promise<ResultOfMnemonicFromEntropy>;
```
### Parameters
- `entropy`: _string_ – Entropy bytes. Hex encoded.
- `dictionary`?: _number_ – Dictionary identifier
- `word_count`?: _number_ – Mnemonic word count
### Result

- `phrase`: _string_ – Phrase


## mnemonic_verify

The phrase supplied will be checked for word length and validated according to the checksum specified in BIP0039.

```ts
type ParamsOfMnemonicVerify = {
    phrase: string,
    dictionary?: number,
    word_count?: number
};

type ResultOfMnemonicVerify = {
    valid: boolean
};

function mnemonic_verify(
    params: ParamsOfMnemonicVerify,
): Promise<ResultOfMnemonicVerify>;
```
### Parameters
- `phrase`: _string_ – Phrase
- `dictionary`?: _number_ – Dictionary identifier
- `word_count`?: _number_ – Word count
### Result

- `valid`: _boolean_ – Flag indicating the mnemonic is valid or not


## mnemonic_derive_sign_keys

Validates the seed phrase, generates master key and then derives the key pair from the master key and the specified path

```ts
type ParamsOfMnemonicDeriveSignKeys = {
    phrase: string,
    path?: string,
    dictionary?: number,
    word_count?: number
};

type KeyPair = {
    public: string,
    secret: string
};

function mnemonic_derive_sign_keys(
    params: ParamsOfMnemonicDeriveSignKeys,
): Promise<KeyPair>;
```
### Parameters
- `phrase`: _string_ – Phrase
- `path`?: _string_ – Derivation path, for instance "m/44'/396'/0'/0/0"
- `dictionary`?: _number_ – Dictionary identifier
- `word_count`?: _number_ – Word count
### Result

- `public`: _string_ – Public key - 64 symbols hex string
- `secret`: _string_ – Private key - u64 symbols hex string


## hdkey_xprv_from_mnemonic

Generates an extended master private key that will be the root for all the derived keys

```ts
type ParamsOfHDKeyXPrvFromMnemonic = {
    phrase: string,
    dictionary?: number,
    word_count?: number
};

type ResultOfHDKeyXPrvFromMnemonic = {
    xprv: string
};

function hdkey_xprv_from_mnemonic(
    params: ParamsOfHDKeyXPrvFromMnemonic,
): Promise<ResultOfHDKeyXPrvFromMnemonic>;
```
### Parameters
- `phrase`: _string_ – String with seed phrase
- `dictionary`?: _number_ – Dictionary identifier
- `word_count`?: _number_ – Mnemonic word count
### Result

- `xprv`: _string_ – Serialized extended master private key


## hdkey_derive_from_xprv

Returns extended private key derived from the specified extended private key and child index

```ts
type ParamsOfHDKeyDeriveFromXPrv = {
    xprv: string,
    child_index: number,
    hardened: boolean
};

type ResultOfHDKeyDeriveFromXPrv = {
    xprv: string
};

function hdkey_derive_from_xprv(
    params: ParamsOfHDKeyDeriveFromXPrv,
): Promise<ResultOfHDKeyDeriveFromXPrv>;
```
### Parameters
- `xprv`: _string_ – Serialized extended private key
- `child_index`: _number_ – Child index (see BIP-0032)
- `hardened`: _boolean_ – Indicates the derivation of hardened/not-hardened key (see BIP-0032)
### Result

- `xprv`: _string_ – Serialized extended private key


## hdkey_derive_from_xprv_path

Derives the exented private key from the specified key and path

```ts
type ParamsOfHDKeyDeriveFromXPrvPath = {
    xprv: string,
    path: string
};

type ResultOfHDKeyDeriveFromXPrvPath = {
    xprv: string
};

function hdkey_derive_from_xprv_path(
    params: ParamsOfHDKeyDeriveFromXPrvPath,
): Promise<ResultOfHDKeyDeriveFromXPrvPath>;
```
### Parameters
- `xprv`: _string_ – Serialized extended private key
- `path`: _string_ – Derivation path, for instance "m/44'/396'/0'/0/0"
### Result

- `xprv`: _string_ – Derived serialized extended private key


## hdkey_secret_from_xprv

Extracts the private key from the serialized extended private key

```ts
type ParamsOfHDKeySecretFromXPrv = {
    xprv: string
};

type ResultOfHDKeySecretFromXPrv = {
    secret: string
};

function hdkey_secret_from_xprv(
    params: ParamsOfHDKeySecretFromXPrv,
): Promise<ResultOfHDKeySecretFromXPrv>;
```
### Parameters
- `xprv`: _string_ – Serialized extended private key
### Result

- `secret`: _string_ – Private key - 64 symbols hex string


## hdkey_public_from_xprv

Extracts the public key from the serialized extended private key

```ts
type ParamsOfHDKeyPublicFromXPrv = {
    xprv: string
};

type ResultOfHDKeyPublicFromXPrv = {
    public: string
};

function hdkey_public_from_xprv(
    params: ParamsOfHDKeyPublicFromXPrv,
): Promise<ResultOfHDKeyPublicFromXPrv>;
```
### Parameters
- `xprv`: _string_ – Serialized extended private key
### Result

- `public`: _string_ – Public key - 64 symbols hex string


# Types
## SigningBoxHandle
```ts
type SigningBoxHandle = number;
```
- _number_


## ParamsOfFactorize
```ts
type ParamsOfFactorize = {
    composite: string
};
```
- `composite`: _string_ – Hexadecimal representation of u64 composite number.


## ResultOfFactorize
```ts
type ResultOfFactorize = {
    factors: string[]
};
```
- `factors`: _string[]_ – Two factors of composite or empty if composite can't be factorized.


## ParamsOfModularPower
```ts
type ParamsOfModularPower = {
    base: string,
    exponent: string,
    modulus: string
};
```
- `base`: _string_ – `base` argument of calculation.
- `exponent`: _string_ – `exponent` argument of calculation.
- `modulus`: _string_ – `modulus` argument of calculation.


## ResultOfModularPower
```ts
type ResultOfModularPower = {
    modular_power: string
};
```
- `modular_power`: _string_ – Result of modular exponentiation


## ParamsOfTonCrc16
```ts
type ParamsOfTonCrc16 = {
    data: string
};
```
- `data`: _string_ – Input data for CRC calculation. Encoded with `base64`.


## ResultOfTonCrc16
```ts
type ResultOfTonCrc16 = {
    crc: number
};
```
- `crc`: _number_ – Calculated CRC for input data.


## ParamsOfGenerateRandomBytes
```ts
type ParamsOfGenerateRandomBytes = {
    length: number
};
```
- `length`: _number_ – Size of random byte array.


## ResultOfGenerateRandomBytes
```ts
type ResultOfGenerateRandomBytes = {
    bytes: string
};
```
- `bytes`: _string_ – Generated bytes encoded in `base64`.


## ParamsOfConvertPublicKeyToTonSafeFormat
```ts
type ParamsOfConvertPublicKeyToTonSafeFormat = {
    public_key: string
};
```
- `public_key`: _string_ – Public key - 64 symbols hex string


## ResultOfConvertPublicKeyToTonSafeFormat
```ts
type ResultOfConvertPublicKeyToTonSafeFormat = {
    ton_public_key: string
};
```
- `ton_public_key`: _string_ – Public key represented in TON safe format.


## KeyPair
```ts
type KeyPair = {
    public: string,
    secret: string
};
```
- `public`: _string_ – Public key - 64 symbols hex string
- `secret`: _string_ – Private key - u64 symbols hex string


## ParamsOfSign
```ts
type ParamsOfSign = {
    unsigned: string,
    keys: KeyPair
};
```
- `unsigned`: _string_ – Data that must be signed encoded in `base64`.
- `keys`: _[KeyPair](mod_crypto.md#KeyPair)_ – Sign keys.


## ResultOfSign
```ts
type ResultOfSign = {
    signed: string,
    signature: string
};
```
- `signed`: _string_ – Signed data combined with signature encoded in `base64`.
- `signature`: _string_ – Signature encoded in `hex`.


## ParamsOfVerifySignature
```ts
type ParamsOfVerifySignature = {
    signed: string,
    public: string
};
```
- `signed`: _string_ – Signed data that must be verified encoded in `base64`.
- `public`: _string_ – Signer's public key - 64 symbols hex string


## ResultOfVerifySignature
```ts
type ResultOfVerifySignature = {
    unsigned: string
};
```
- `unsigned`: _string_ – Unsigned data encoded in `base64`.


## ParamsOfHash
```ts
type ParamsOfHash = {
    data: string
};
```
- `data`: _string_ – Input data for hash calculation. Encoded with `base64`.


## ResultOfHash
```ts
type ResultOfHash = {
    hash: string
};
```
- `hash`: _string_ – Hash of input `data`. Encoded with 'hex'.


## ParamsOfScrypt
```ts
type ParamsOfScrypt = {
    password: string,
    salt: string,
    log_n: number,
    r: number,
    p: number,
    dk_len: number
};
```
- `password`: _string_ – The password bytes to be hashed. Must be encoded with `base64`.
- `salt`: _string_ – A salt bytes that modifies the hash to protect against Rainbow table attacks. Must be encoded with `base64`.
- `log_n`: _number_ – CPU/memory cost parameter
- `r`: _number_ – The block size parameter, which fine-tunes sequential memory read size and performance.
- `p`: _number_ – Parallelization parameter.
- `dk_len`: _number_ – Intended output length in octets of the derived key.


## ResultOfScrypt
```ts
type ResultOfScrypt = {
    key: string
};
```
- `key`: _string_ – Derived key. Encoded with `hex`.


## ParamsOfNaclSignKeyPairFromSecret
```ts
type ParamsOfNaclSignKeyPairFromSecret = {
    secret: string
};
```
- `secret`: _string_ – Secret key - unprefixed 0-padded to 64 symbols hex string


## ParamsOfNaclSign
```ts
type ParamsOfNaclSign = {
    unsigned: string,
    secret: string
};
```
- `unsigned`: _string_ – Data that must be signed encoded in `base64`.
- `secret`: _string_ – Signer's secret key - unprefixed 0-padded to 64 symbols hex string


## ResultOfNaclSign
```ts
type ResultOfNaclSign = {
    signed: string
};
```
- `signed`: _string_ – Signed data, encoded in `base64`.


## ParamsOfNaclSignOpen
```ts
type ParamsOfNaclSignOpen = {
    signed: string,
    public: string
};
```
- `signed`: _string_ – Signed data that must be unsigned. Encoded with `base64`.
- `public`: _string_ – Signer's public key - unprefixed 0-padded to 64 symbols hex string


## ResultOfNaclSignOpen
```ts
type ResultOfNaclSignOpen = {
    unsigned: string
};
```
- `unsigned`: _string_ – Unsigned data, encoded in `base64`.


## ResultOfNaclSignDetached
```ts
type ResultOfNaclSignDetached = {
    signature: string
};
```
- `signature`: _string_ – Signature encoded in `hex`.


## ParamsOfNaclBoxKeyPairFromSecret
```ts
type ParamsOfNaclBoxKeyPairFromSecret = {
    secret: string
};
```
- `secret`: _string_ – Secret key - unprefixed 0-padded to 64 symbols hex string


## ParamsOfNaclBox
```ts
type ParamsOfNaclBox = {
    decrypted: string,
    nonce: string,
    their_public: string,
    secret: string
};
```
- `decrypted`: _string_ – Data that must be encrypted encoded in `base64`.
- `nonce`: _string_ – Nonce, encoded in `hex`
- `their_public`: _string_ – Receiver's public key - unprefixed 0-padded to 64 symbols hex string
- `secret`: _string_ – Sender's private key - unprefixed 0-padded to 64 symbols hex string


## ResultOfNaclBox
```ts
type ResultOfNaclBox = {
    encrypted: string
};
```
- `encrypted`: _string_ – Encrypted data encoded in `base64`.


## ParamsOfNaclBoxOpen
```ts
type ParamsOfNaclBoxOpen = {
    encrypted: string,
    nonce: string,
    their_public: string,
    secret: string
};
```
- `encrypted`: _string_ – Data that must be decrypted. Encoded with `base64`.
- `nonce`: _string_
- `their_public`: _string_ – Sender's public key - unprefixed 0-padded to 64 symbols hex string
- `secret`: _string_ – Receiver's private key - unprefixed 0-padded to 64 symbols hex string


## ResultOfNaclBoxOpen
```ts
type ResultOfNaclBoxOpen = {
    decrypted: string
};
```
- `decrypted`: _string_ – Decrypted data encoded in `base64`.


## ParamsOfNaclSecretBox
```ts
type ParamsOfNaclSecretBox = {
    decrypted: string,
    nonce: string,
    key: string
};
```
- `decrypted`: _string_ – Data that must be encrypted. Encoded with `base64`.
- `nonce`: _string_ – Nonce in `hex`
- `key`: _string_ – Secret key - unprefixed 0-padded to 64 symbols hex string


## ParamsOfNaclSecretBoxOpen
```ts
type ParamsOfNaclSecretBoxOpen = {
    encrypted: string,
    nonce: string,
    key: string
};
```
- `encrypted`: _string_ – Data that must be decrypted. Encoded with `base64`.
- `nonce`: _string_ – Nonce in `hex`
- `key`: _string_ – Public key - unprefixed 0-padded to 64 symbols hex string


## ParamsOfMnemonicWords
```ts
type ParamsOfMnemonicWords = {
    dictionary?: number
};
```
- `dictionary`?: _number_ – Dictionary identifier


## ResultOfMnemonicWords
```ts
type ResultOfMnemonicWords = {
    words: string
};
```
- `words`: _string_ – The list of mnemonic words


## ParamsOfMnemonicFromRandom
```ts
type ParamsOfMnemonicFromRandom = {
    dictionary?: number,
    word_count?: number
};
```
- `dictionary`?: _number_ – Dictionary identifier
- `word_count`?: _number_ – Mnemonic word count


## ResultOfMnemonicFromRandom
```ts
type ResultOfMnemonicFromRandom = {
    phrase: string
};
```
- `phrase`: _string_ – String of mnemonic words


## ParamsOfMnemonicFromEntropy
```ts
type ParamsOfMnemonicFromEntropy = {
    entropy: string,
    dictionary?: number,
    word_count?: number
};
```
- `entropy`: _string_ – Entropy bytes. Hex encoded.
- `dictionary`?: _number_ – Dictionary identifier
- `word_count`?: _number_ – Mnemonic word count


## ResultOfMnemonicFromEntropy
```ts
type ResultOfMnemonicFromEntropy = {
    phrase: string
};
```
- `phrase`: _string_ – Phrase


## ParamsOfMnemonicVerify
```ts
type ParamsOfMnemonicVerify = {
    phrase: string,
    dictionary?: number,
    word_count?: number
};
```
- `phrase`: _string_ – Phrase
- `dictionary`?: _number_ – Dictionary identifier
- `word_count`?: _number_ – Word count


## ResultOfMnemonicVerify
```ts
type ResultOfMnemonicVerify = {
    valid: boolean
};
```
- `valid`: _boolean_ – Flag indicating the mnemonic is valid or not


## ParamsOfMnemonicDeriveSignKeys
```ts
type ParamsOfMnemonicDeriveSignKeys = {
    phrase: string,
    path?: string,
    dictionary?: number,
    word_count?: number
};
```
- `phrase`: _string_ – Phrase
- `path`?: _string_ – Derivation path, for instance "m/44'/396'/0'/0/0"
- `dictionary`?: _number_ – Dictionary identifier
- `word_count`?: _number_ – Word count


## ParamsOfHDKeyXPrvFromMnemonic
```ts
type ParamsOfHDKeyXPrvFromMnemonic = {
    phrase: string,
    dictionary?: number,
    word_count?: number
};
```
- `phrase`: _string_ – String with seed phrase
- `dictionary`?: _number_ – Dictionary identifier
- `word_count`?: _number_ – Mnemonic word count


## ResultOfHDKeyXPrvFromMnemonic
```ts
type ResultOfHDKeyXPrvFromMnemonic = {
    xprv: string
};
```
- `xprv`: _string_ – Serialized extended master private key


## ParamsOfHDKeyDeriveFromXPrv
```ts
type ParamsOfHDKeyDeriveFromXPrv = {
    xprv: string,
    child_index: number,
    hardened: boolean
};
```
- `xprv`: _string_ – Serialized extended private key
- `child_index`: _number_ – Child index (see BIP-0032)
- `hardened`: _boolean_ – Indicates the derivation of hardened/not-hardened key (see BIP-0032)


## ResultOfHDKeyDeriveFromXPrv
```ts
type ResultOfHDKeyDeriveFromXPrv = {
    xprv: string
};
```
- `xprv`: _string_ – Serialized extended private key


## ParamsOfHDKeyDeriveFromXPrvPath
```ts
type ParamsOfHDKeyDeriveFromXPrvPath = {
    xprv: string,
    path: string
};
```
- `xprv`: _string_ – Serialized extended private key
- `path`: _string_ – Derivation path, for instance "m/44'/396'/0'/0/0"


## ResultOfHDKeyDeriveFromXPrvPath
```ts
type ResultOfHDKeyDeriveFromXPrvPath = {
    xprv: string
};
```
- `xprv`: _string_ – Derived serialized extended private key


## ParamsOfHDKeySecretFromXPrv
```ts
type ParamsOfHDKeySecretFromXPrv = {
    xprv: string
};
```
- `xprv`: _string_ – Serialized extended private key


## ResultOfHDKeySecretFromXPrv
```ts
type ResultOfHDKeySecretFromXPrv = {
    secret: string
};
```
- `secret`: _string_ – Private key - 64 symbols hex string


## ParamsOfHDKeyPublicFromXPrv
```ts
type ParamsOfHDKeyPublicFromXPrv = {
    xprv: string
};
```
- `xprv`: _string_ – Serialized extended private key


## ResultOfHDKeyPublicFromXPrv
```ts
type ResultOfHDKeyPublicFromXPrv = {
    public: string
};
```
- `public`: _string_ – Public key - 64 symbols hex string


