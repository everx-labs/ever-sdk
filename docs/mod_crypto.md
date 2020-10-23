# Module crypto

 Crypto functions.
## Functions
[factorize](#factorize) – Integer factorization

[modular_power](#modular_power) – Modular exponentiation

[ton_crc16](#ton_crc16) –  Calculates CRC16 using TON algorithm.

[generate_random_bytes](#generate_random_bytes) – Generates random byte array of the specified length in the spesified encoding

[convert_public_key_to_ton_safe_format](#convert_public_key_to_ton_safe_format) –  Converts public key to ton safe_format

[generate_random_sign_keys](#generate_random_sign_keys) –  Generates random ed25519 key pair.

[sign](#sign) –  Signs a data using the provided keys.

[verify_signature](#verify_signature) –  Verifies signed data using the provided public key.

[sha256](#sha256) –  Calculates SHA256 hash of the specified data.

[sha512](#sha512) – Calculates SHA512 hash of the specified data.

[scrypt](#scrypt) – Perform `scrypt` encryption

[nacl_sign_keypair_from_secret_key](#nacl_sign_keypair_from_secret_key) –  Generates a key pair for signing from the secret key

[nacl_sign](#nacl_sign) –  Signs data using the signer's secret key.

[nacl_sign_open](#nacl_sign_open)

[nacl_sign_detached](#nacl_sign_detached)

[nacl_box_keypair](#nacl_box_keypair)

[nacl_box_keypair_from_secret_key](#nacl_box_keypair_from_secret_key)

[nacl_box](#nacl_box)

[nacl_box_open](#nacl_box_open)

[nacl_secret_box](#nacl_secret_box)

[nacl_secret_box_open](#nacl_secret_box_open)

[mnemonic_words](#mnemonic_words) –  Prints the list of words from the specified dictionary

[mnemonic_from_random](#mnemonic_from_random) – Generates a random mnemonic

[mnemonic_from_entropy](#mnemonic_from_entropy) – Generates mnemonic from the specified entropy

[mnemonic_verify](#mnemonic_verify) – Validates a mnemonic phrase

[mnemonic_derive_sign_keys](#mnemonic_derive_sign_keys) – Derives a key pair for signing from the seed phrase

[hdkey_xprv_from_mnemonic](#hdkey_xprv_from_mnemonic) –  Generate the extended master private key that will be the root for all the derived keys

[hdkey_derive_from_xprv](#hdkey_derive_from_xprv) – Derives the next child extended private key

[hdkey_derive_from_xprv_path](#hdkey_derive_from_xprv_path) – Derives the extended private key from the specified key and path

[hdkey_secret_from_xprv](#hdkey_secret_from_xprv) –  Extracts the private key from the serialized extended private key

[hdkey_public_from_xprv](#hdkey_public_from_xprv) –  Extracts the public key from the serialized extended private key

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

 Performs prime factorization – decomposition of a composite number
 into a product of smaller prime integers (factors).
 See [https://en.wikipedia.org/wiki/Integer_factorization]

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
- `composite`: _string_ –  Hexadecimal representation of u64 composite number.
### Result

- `factors`: _string[]_ –  Two factors of composite or empty if composite can't be factorized.


## modular_power

 Performs modular exponentiation for big integers (`base`^`exponent` mod `modulus`).
 See [https://en.wikipedia.org/wiki/Modular_exponentiation]

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
- `base`: _string_ –  `base` argument of calculation.
- `exponent`: _string_ –  `exponent` argument of calculation.
- `modulus`: _string_ –  `modulus` argument of calculation.
### Result

- `modular_power`: _string_ –  result of modular exponentiation


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
- `data`: _string_ –  Input data for CRC calculation. Encoded with `base64`.
### Result

- `crc`: _number_ –  Calculated CRC for input data.


## generate_random_bytes

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
- `length`: _number_ –  Size of random byte array.
### Result

- `bytes`: _string_ –  Generated bytes, encoded with `base64`.


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
- `public_key`: _string_ –  Public key.
### Result

- `ton_public_key`: _string_ –  Public key represented in TON safe format.


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

- `public`: _string_ –  Public key. Encoded with `hex`.
- `secret`: _string_ –  Private key. Encoded with `hex`.


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
- `unsigned`: _string_ –  Data that must be signed.
- `keys`: _[KeyPair](mod_crypto.md#KeyPair)_ –  Sign keys.
### Result

- `signed`: _string_ –  Signed data combined with signature. Encoded with `base64`.
- `signature`: _string_ –  Signature. Encoded with `base64`.


## verify_signature

 Verifies signed data using the provided public key.
 Raises error in case when verification is failed.

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
- `signed`: _string_ –  Signed data that must be verified.
- `public`: _string_ –  Signer's public key.
### Result

- `unsigned`: _string_ –  Unsigned data.


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
- `data`: _string_ –  Input data for hash calculation. Encoded with `base64`.
### Result

- `hash`: _string_ –  Hex-encoded hash of input `data`.


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
- `data`: _string_ –  Input data for hash calculation. Encoded with `base64`.
### Result

- `hash`: _string_ –  Hex-encoded hash of input `data`.


## scrypt

 Derives key from `password` and `key` using `scrypt` algorithm.
 See [https://en.wikipedia.org/wiki/Scrypt].

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
- `password`: _string_ –  The password bytes to be hashed.
- `salt`: _string_ –  A salt bytes that modifies the hash to protect against Rainbow table attacks.
- `log_n`: _number_ –  CPU/memory cost parameter
- `r`: _number_ –  The block size parameter, which fine-tunes sequential memory read size and performance.
- `p`: _number_ –  Parallelization parameter.
- `dk_len`: _number_ –  Intended output length in octets of the derived key.
### Result

- `key`: _string_ –  Derived key. Encoded with `hex`.


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
- `secret`: _string_ –  secret key
### Result

- `public`: _string_ –  Public key. Encoded with `hex`.
- `secret`: _string_ –  Private key. Encoded with `hex`.


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
- `unsigned`: _string_ –  Data that must be signed. Encoded with `base64`.
- `secret`: _string_ –  Signer's secret key.
### Result

- `signed`: _string_ –  Signed data, encoded with `base64`.


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
- `signed`: _string_ –  Signed data that must be unsigned. Encoded with `base64`.
- `public`: _string_ –  Signer's public key.
### Result

- `unsigned`: _string_ –  Unsigned data, encoded with `base64`.


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
- `unsigned`: _string_ –  Data that must be signed. Encoded with `base64`.
- `secret`: _string_ –  Signer's secret key.
### Result

- `signature`: _string_ –  Hex encoded sign.


## nacl_box_keypair

```ts
type KeyPair = {
    public: string,
    secret: string
};

function nacl_box_keypair(): Promise<KeyPair>;
```
### Result

- `public`: _string_ –  Public key. Encoded with `hex`.
- `secret`: _string_ –  Private key. Encoded with `hex`.


## nacl_box_keypair_from_secret_key

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
- `secret`: _string_ –  Hex encoded secret key.
### Result

- `public`: _string_ –  Public key. Encoded with `hex`.
- `secret`: _string_ –  Private key. Encoded with `hex`.


## nacl_box

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
- `decrypted`: _string_ –  Data that must be encrypted. Encoded with `base64`.
- `nonce`: _string_
- `their_public`: _string_
- `secret`: _string_
### Result

- `encrypted`: _string_ –  Encrypted data. Encoded with `base64`.


## nacl_box_open

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
- `encrypted`: _string_ –  Data that must be decrypted. Encoded with `base64`.
- `nonce`: _string_
- `their_public`: _string_
- `secret`: _string_
### Result

- `decrypted`: _string_ –  Decrypted data. Encoded with `base64`.


## nacl_secret_box

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
- `decrypted`: _string_ –  Data that must be encrypted. Encoded with `base64`.
- `nonce`: _string_
- `key`: _string_
### Result

- `encrypted`: _string_ –  Encrypted data. Encoded with `base64`.


## nacl_secret_box_open

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
- `encrypted`: _string_ –  Data that must be decrypted. Encoded with `base64`.
- `nonce`: _string_
- `key`: _string_
### Result

- `decrypted`: _string_ –  Decrypted data. Encoded with `base64`.


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
- `dictionary`?: _number_ –  dictionary identifier
### Result

- `words`: _string_ –  the list of mnemonic words


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
- `dictionary`?: _number_ –  dictionary identifier
- `word_count`?: _number_ –  mnemonic word count
### Result

- `phrase`: _string_ –  string of mnemonic words


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
- `entropy`: _string_
- `dictionary`?: _number_
- `word_count`?: _number_
### Result

- `phrase`: _string_


## mnemonic_verify

 The phrase supplied will be checked for word length and validated according to the checksum
 specified in BIP0039.

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
- `phrase`: _string_ –  phrase
- `dictionary`?: _number_ –  dictionary identifier
- `word_count`?: _number_ –  word count
### Result

- `valid`: _boolean_ –  flag indicating the mnemonic is valid or not


## mnemonic_derive_sign_keys

 Validates the seed phrase, generates master key and then derives
 the key pair from the master key and the specified path

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
- `phrase`: _string_ –  phrase
- `path`?: _string_ –  derivation path, for instance "m/44'/396'/0'/0/0"
- `dictionary`?: _number_ –  dictionary identifier
- `word_count`?: _number_ –  word count
### Result

- `public`: _string_ –  Public key. Encoded with `hex`.
- `secret`: _string_ –  Private key. Encoded with `hex`.


## hdkey_xprv_from_mnemonic

 Generate the extended master private key that will be the root for all the derived keys

```ts
type ParamsOfHDKeyXPrvFromMnemonic = {
    phrase: string
};

type ResultOfHDKeyXPrvFromMnemonic = {
    xprv: string
};

function hdkey_xprv_from_mnemonic(
    params: ParamsOfHDKeyXPrvFromMnemonic,
): Promise<ResultOfHDKeyXPrvFromMnemonic>;
```
### Parameters
- `phrase`: _string_ – string with seed phrase
### Result

- `xprv`: _string_ –  serialized extended master private key


## hdkey_derive_from_xprv

 Returns derived extended private key derived from the specified extended private key and child index

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
- `xprv`: _string_ –  serialized extended private key
- `child_index`: _number_ –  child index (see BIP-0032)
- `hardened`: _boolean_ –  indicates the derivation of hardened/not-hardened key (see BIP-0032)
### Result

- `xprv`: _string_ –  serialized extended private key


## hdkey_derive_from_xprv_path

 Derives the extended private key from the specified key and path

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
- `xprv`: _string_ –  serialized extended private key
- `path`: _string_ –  derivation path, for instance "m/44'/396'/0'/0/0"
### Result

- `xprv`: _string_ –  derived serialized extended private key


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
- `xprv`: _string_ –  serialized extended private key
### Result

- `secret`: _string_ –  private key


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
- `xprv`: _string_ –  serialized extended private key
### Result

- `public`: _string_ –  public key


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
- `composite`: _string_ –  Hexadecimal representation of u64 composite number.


## ResultOfFactorize

```ts
type ResultOfFactorize = {
    factors: string[]
};
```
- `factors`: _string[]_ –  Two factors of composite or empty if composite can't be factorized.


## ParamsOfModularPower

```ts
type ParamsOfModularPower = {
    base: string,
    exponent: string,
    modulus: string
};
```
- `base`: _string_ –  `base` argument of calculation.
- `exponent`: _string_ –  `exponent` argument of calculation.
- `modulus`: _string_ –  `modulus` argument of calculation.


## ResultOfModularPower

```ts
type ResultOfModularPower = {
    modular_power: string
};
```
- `modular_power`: _string_ –  result of modular exponentiation


## ParamsOfTonCrc16

```ts
type ParamsOfTonCrc16 = {
    data: string
};
```
- `data`: _string_ –  Input data for CRC calculation. Encoded with `base64`.


## ResultOfTonCrc16

```ts
type ResultOfTonCrc16 = {
    crc: number
};
```
- `crc`: _number_ –  Calculated CRC for input data.


## ParamsOfGenerateRandomBytes

```ts
type ParamsOfGenerateRandomBytes = {
    length: number
};
```
- `length`: _number_ –  Size of random byte array.


## ResultOfGenerateRandomBytes

```ts
type ResultOfGenerateRandomBytes = {
    bytes: string
};
```
- `bytes`: _string_ –  Generated bytes, encoded with `base64`.


## ParamsOfConvertPublicKeyToTonSafeFormat

```ts
type ParamsOfConvertPublicKeyToTonSafeFormat = {
    public_key: string
};
```
- `public_key`: _string_ –  Public key.


## ResultOfConvertPublicKeyToTonSafeFormat

```ts
type ResultOfConvertPublicKeyToTonSafeFormat = {
    ton_public_key: string
};
```
- `ton_public_key`: _string_ –  Public key represented in TON safe format.


## KeyPair

```ts
type KeyPair = {
    public: string,
    secret: string
};
```
- `public`: _string_ –  Public key. Encoded with `hex`.
- `secret`: _string_ –  Private key. Encoded with `hex`.


## ParamsOfSign

```ts
type ParamsOfSign = {
    unsigned: string,
    keys: KeyPair
};
```
- `unsigned`: _string_ –  Data that must be signed.
- `keys`: _[KeyPair](mod_crypto.md#KeyPair)_ –  Sign keys.


## ResultOfSign

```ts
type ResultOfSign = {
    signed: string,
    signature: string
};
```
- `signed`: _string_ –  Signed data combined with signature. Encoded with `base64`.
- `signature`: _string_ –  Signature. Encoded with `base64`.


## ParamsOfVerifySignature

```ts
type ParamsOfVerifySignature = {
    signed: string,
    public: string
};
```
- `signed`: _string_ –  Signed data that must be verified.
- `public`: _string_ –  Signer's public key.


## ResultOfVerifySignature

```ts
type ResultOfVerifySignature = {
    unsigned: string
};
```
- `unsigned`: _string_ –  Unsigned data.


## ParamsOfHash

```ts
type ParamsOfHash = {
    data: string
};
```
- `data`: _string_ –  Input data for hash calculation. Encoded with `base64`.


## ResultOfHash

```ts
type ResultOfHash = {
    hash: string
};
```
- `hash`: _string_ –  Hex-encoded hash of input `data`.


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
- `password`: _string_ –  The password bytes to be hashed.
- `salt`: _string_ –  A salt bytes that modifies the hash to protect against Rainbow table attacks.
- `log_n`: _number_ –  CPU/memory cost parameter
- `r`: _number_ –  The block size parameter, which fine-tunes sequential memory read size and performance.
- `p`: _number_ –  Parallelization parameter.
- `dk_len`: _number_ –  Intended output length in octets of the derived key.


## ResultOfScrypt

```ts
type ResultOfScrypt = {
    key: string
};
```
- `key`: _string_ –  Derived key. Encoded with `hex`.


## ParamsOfNaclSignKeyPairFromSecret

```ts
type ParamsOfNaclSignKeyPairFromSecret = {
    secret: string
};
```
- `secret`: _string_ –  secret key


## ParamsOfNaclSign

```ts
type ParamsOfNaclSign = {
    unsigned: string,
    secret: string
};
```
- `unsigned`: _string_ –  Data that must be signed. Encoded with `base64`.
- `secret`: _string_ –  Signer's secret key.


## ResultOfNaclSign

```ts
type ResultOfNaclSign = {
    signed: string
};
```
- `signed`: _string_ –  Signed data, encoded with `base64`.


## ParamsOfNaclSignOpen

```ts
type ParamsOfNaclSignOpen = {
    signed: string,
    public: string
};
```
- `signed`: _string_ –  Signed data that must be unsigned. Encoded with `base64`.
- `public`: _string_ –  Signer's public key.


## ResultOfNaclSignOpen

```ts
type ResultOfNaclSignOpen = {
    unsigned: string
};
```
- `unsigned`: _string_ –  Unsigned data, encoded with `base64`.


## ResultOfNaclSignDetached

```ts
type ResultOfNaclSignDetached = {
    signature: string
};
```
- `signature`: _string_ –  Hex encoded sign.


## ParamsOfNaclBoxKeyPairFromSecret

```ts
type ParamsOfNaclBoxKeyPairFromSecret = {
    secret: string
};
```
- `secret`: _string_ –  Hex encoded secret key.


## ParamsOfNaclBox

```ts
type ParamsOfNaclBox = {
    decrypted: string,
    nonce: string,
    their_public: string,
    secret: string
};
```
- `decrypted`: _string_ –  Data that must be encrypted. Encoded with `base64`.
- `nonce`: _string_
- `their_public`: _string_
- `secret`: _string_


## ResultOfNaclBox

```ts
type ResultOfNaclBox = {
    encrypted: string
};
```
- `encrypted`: _string_ –  Encrypted data. Encoded with `base64`.


## ParamsOfNaclBoxOpen

```ts
type ParamsOfNaclBoxOpen = {
    encrypted: string,
    nonce: string,
    their_public: string,
    secret: string
};
```
- `encrypted`: _string_ –  Data that must be decrypted. Encoded with `base64`.
- `nonce`: _string_
- `their_public`: _string_
- `secret`: _string_


## ResultOfNaclBoxOpen

```ts
type ResultOfNaclBoxOpen = {
    decrypted: string
};
```
- `decrypted`: _string_ –  Decrypted data. Encoded with `base64`.


## ParamsOfNaclSecretBox

```ts
type ParamsOfNaclSecretBox = {
    decrypted: string,
    nonce: string,
    key: string
};
```
- `decrypted`: _string_ –  Data that must be encrypted. Encoded with `base64`.
- `nonce`: _string_
- `key`: _string_


## ParamsOfNaclSecretBoxOpen

```ts
type ParamsOfNaclSecretBoxOpen = {
    encrypted: string,
    nonce: string,
    key: string
};
```
- `encrypted`: _string_ –  Data that must be decrypted. Encoded with `base64`.
- `nonce`: _string_
- `key`: _string_


## ParamsOfMnemonicWords

```ts
type ParamsOfMnemonicWords = {
    dictionary?: number
};
```
- `dictionary`?: _number_ –  dictionary identifier


## ResultOfMnemonicWords

```ts
type ResultOfMnemonicWords = {
    words: string
};
```
- `words`: _string_ –  the list of mnemonic words


## ParamsOfMnemonicFromRandom

```ts
type ParamsOfMnemonicFromRandom = {
    dictionary?: number,
    word_count?: number
};
```
- `dictionary`?: _number_ –  dictionary identifier
- `word_count`?: _number_ –  mnemonic word count


## ResultOfMnemonicFromRandom

```ts
type ResultOfMnemonicFromRandom = {
    phrase: string
};
```
- `phrase`: _string_ –  string of mnemonic words


## ParamsOfMnemonicFromEntropy

```ts
type ParamsOfMnemonicFromEntropy = {
    entropy: string,
    dictionary?: number,
    word_count?: number
};
```
- `entropy`: _string_
- `dictionary`?: _number_
- `word_count`?: _number_


## ResultOfMnemonicFromEntropy

```ts
type ResultOfMnemonicFromEntropy = {
    phrase: string
};
```
- `phrase`: _string_


## ParamsOfMnemonicVerify

```ts
type ParamsOfMnemonicVerify = {
    phrase: string,
    dictionary?: number,
    word_count?: number
};
```
- `phrase`: _string_ –  phrase
- `dictionary`?: _number_ –  dictionary identifier
- `word_count`?: _number_ –  word count


## ResultOfMnemonicVerify

```ts
type ResultOfMnemonicVerify = {
    valid: boolean
};
```
- `valid`: _boolean_ –  flag indicating the mnemonic is valid or not


## ParamsOfMnemonicDeriveSignKeys

```ts
type ParamsOfMnemonicDeriveSignKeys = {
    phrase: string,
    path?: string,
    dictionary?: number,
    word_count?: number
};
```
- `phrase`: _string_ –  phrase
- `path`?: _string_ –  derivation path, for instance "m/44'/396'/0'/0/0"
- `dictionary`?: _number_ –  dictionary identifier
- `word_count`?: _number_ –  word count


## ParamsOfHDKeyXPrvFromMnemonic

```ts
type ParamsOfHDKeyXPrvFromMnemonic = {
    phrase: string
};
```
- `phrase`: _string_ – string with seed phrase


## ResultOfHDKeyXPrvFromMnemonic

```ts
type ResultOfHDKeyXPrvFromMnemonic = {
    xprv: string
};
```
- `xprv`: _string_ –  serialized extended master private key


## ParamsOfHDKeyDeriveFromXPrv

```ts
type ParamsOfHDKeyDeriveFromXPrv = {
    xprv: string,
    child_index: number,
    hardened: boolean
};
```
- `xprv`: _string_ –  serialized extended private key
- `child_index`: _number_ –  child index (see BIP-0032)
- `hardened`: _boolean_ –  indicates the derivation of hardened/not-hardened key (see BIP-0032)


## ResultOfHDKeyDeriveFromXPrv

```ts
type ResultOfHDKeyDeriveFromXPrv = {
    xprv: string
};
```
- `xprv`: _string_ –  serialized extended private key


## ParamsOfHDKeyDeriveFromXPrvPath

```ts
type ParamsOfHDKeyDeriveFromXPrvPath = {
    xprv: string,
    path: string
};
```
- `xprv`: _string_ –  serialized extended private key
- `path`: _string_ –  derivation path, for instance "m/44'/396'/0'/0/0"


## ResultOfHDKeyDeriveFromXPrvPath

```ts
type ResultOfHDKeyDeriveFromXPrvPath = {
    xprv: string
};
```
- `xprv`: _string_ –  derived serialized extended private key


## ParamsOfHDKeySecretFromXPrv

```ts
type ParamsOfHDKeySecretFromXPrv = {
    xprv: string
};
```
- `xprv`: _string_ –  serialized extended private key


## ResultOfHDKeySecretFromXPrv

```ts
type ResultOfHDKeySecretFromXPrv = {
    secret: string
};
```
- `secret`: _string_ –  private key


## ParamsOfHDKeyPublicFromXPrv

```ts
type ParamsOfHDKeyPublicFromXPrv = {
    xprv: string
};
```
- `xprv`: _string_ –  serialized extended private key


## ResultOfHDKeyPublicFromXPrv

```ts
type ResultOfHDKeyPublicFromXPrv = {
    public: string
};
```
- `public`: _string_ –  public key

