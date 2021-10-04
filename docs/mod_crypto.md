# Module crypto

## Module crypto

Crypto functions.

### Functions

[factorize](mod_crypto.md#factorize) – Integer factorization

[modular\_power](mod_crypto.md#modular_power) – Modular exponentiation

[ton\_crc16](mod_crypto.md#ton_crc16) – Calculates CRC16 using TON algorithm.

[generate\_random\_bytes](mod_crypto.md#generate_random_bytes) – Generates random byte array of the specified length and returns it in `base64` format

[convert\_public\_key\_to\_ton\_safe\_format](mod_crypto.md#convert_public_key_to_ton_safe_format) – Converts public key to ton safe\_format

[generate\_random\_sign\_keys](mod_crypto.md#generate_random_sign_keys) – Generates random ed25519 key pair.

[sign](mod_crypto.md#sign) – Signs a data using the provided keys.

[verify\_signature](mod_crypto.md#verify_signature) – Verifies signed data using the provided public key. Raises error if verification is failed.

[sha256](mod_crypto.md#sha256) – Calculates SHA256 hash of the specified data.

[sha512](mod_crypto.md#sha512) – Calculates SHA512 hash of the specified data.

[scrypt](mod_crypto.md#scrypt) – Perform `scrypt` encryption

[nacl\_sign\_keypair\_from\_secret\_key](mod_crypto.md#nacl_sign_keypair_from_secret_key) – Generates a key pair for signing from the secret key

[nacl\_sign](mod_crypto.md#nacl_sign) – Signs data using the signer's secret key.

[nacl\_sign\_open](mod_crypto.md#nacl_sign_open) – Verifies the signature and returns the unsigned message

[nacl\_sign\_detached](mod_crypto.md#nacl_sign_detached) – Signs the message using the secret key and returns a signature.

[nacl\_sign\_detached\_verify](mod_crypto.md#nacl_sign_detached_verify) – Verifies the signature with public key and `unsigned` data.

[nacl\_box\_keypair](mod_crypto.md#nacl_box_keypair) – Generates a random NaCl key pair

[nacl\_box\_keypair\_from\_secret\_key](mod_crypto.md#nacl_box_keypair_from_secret_key) – Generates key pair from a secret key

[nacl\_box](mod_crypto.md#nacl_box) – Public key authenticated encryption

[nacl\_box\_open](mod_crypto.md#nacl_box_open) – Decrypt and verify the cipher text using the receivers secret key, the senders public key, and the nonce.

[nacl\_secret\_box](mod_crypto.md#nacl_secret_box) – Encrypt and authenticate message using nonce and secret key.

[nacl\_secret\_box\_open](mod_crypto.md#nacl_secret_box_open) – Decrypts and verifies cipher text using `nonce` and secret `key`.

[mnemonic\_words](mod_crypto.md#mnemonic_words) – Prints the list of words from the specified dictionary

[mnemonic\_from\_random](mod_crypto.md#mnemonic_from_random) – Generates a random mnemonic

[mnemonic\_from\_entropy](mod_crypto.md#mnemonic_from_entropy) – Generates mnemonic from pre-generated entropy

[mnemonic\_verify](mod_crypto.md#mnemonic_verify) – Validates a mnemonic phrase

[mnemonic\_derive\_sign\_keys](mod_crypto.md#mnemonic_derive_sign_keys) – Derives a key pair for signing from the seed phrase

[hdkey\_xprv\_from\_mnemonic](mod_crypto.md#hdkey_xprv_from_mnemonic) – Generates an extended master private key that will be the root for all the derived keys

[hdkey\_derive\_from\_xprv](mod_crypto.md#hdkey_derive_from_xprv) – Returns extended private key derived from the specified extended private key and child index

[hdkey\_derive\_from\_xprv\_path](mod_crypto.md#hdkey_derive_from_xprv_path) – Derives the extended private key from the specified key and path

[hdkey\_secret\_from\_xprv](mod_crypto.md#hdkey_secret_from_xprv) – Extracts the private key from the serialized extended private key

[hdkey\_public\_from\_xprv](mod_crypto.md#hdkey_public_from_xprv) – Extracts the public key from the serialized extended private key

[chacha20](mod_crypto.md#chacha20) – Performs symmetric `chacha20` encryption.

[register\_signing\_box](mod_crypto.md#register_signing_box) – Register an application implemented signing box.

[get\_signing\_box](mod_crypto.md#get_signing_box) – Creates a default signing box implementation.

[signing\_box\_get\_public\_key](mod_crypto.md#signing_box_get_public_key) – Returns public key of signing key pair.

[signing\_box\_sign](mod_crypto.md#signing_box_sign) – Returns signed user data.

[remove\_signing\_box](mod_crypto.md#remove_signing_box) – Removes signing box from SDK.

[register\_encryption\_box](mod_crypto.md#register_encryption_box) – Register an application implemented encryption box.

[remove\_encryption\_box](mod_crypto.md#remove_encryption_box) – Removes encryption box from SDK

[encryption\_box\_get\_info](mod_crypto.md#encryption_box_get_info) – Queries info from the given encryption box

[encryption\_box\_encrypt](mod_crypto.md#encryption_box_encrypt) – Encrypts data using given encryption box Note.

[encryption\_box\_decrypt](mod_crypto.md#encryption_box_decrypt) – Decrypts data using given encryption box Note.

[create\_encryption\_box](mod_crypto.md#create_encryption_box) – Creates encryption box with specified algorithm

### Types

[CryptoErrorCode](mod_crypto.md#CryptoErrorCode)

[SigningBoxHandle](mod_crypto.md#SigningBoxHandle)

[EncryptionBoxHandle](mod_crypto.md#EncryptionBoxHandle)

[EncryptionBoxInfo](mod_crypto.md#EncryptionBoxInfo) – Encryption box information

[EncryptionAlgorithm](mod_crypto.md#EncryptionAlgorithm)

[CipherMode](mod_crypto.md#CipherMode)

[AesParams](mod_crypto.md#AesParams)

[AesInfo](mod_crypto.md#AesInfo)

[ParamsOfFactorize](mod_crypto.md#ParamsOfFactorize)

[ResultOfFactorize](mod_crypto.md#ResultOfFactorize)

[ParamsOfModularPower](mod_crypto.md#ParamsOfModularPower)

[ResultOfModularPower](mod_crypto.md#ResultOfModularPower)

[ParamsOfTonCrc16](mod_crypto.md#ParamsOfTonCrc16)

[ResultOfTonCrc16](mod_crypto.md#ResultOfTonCrc16)

[ParamsOfGenerateRandomBytes](mod_crypto.md#ParamsOfGenerateRandomBytes)

[ResultOfGenerateRandomBytes](mod_crypto.md#ResultOfGenerateRandomBytes)

[ParamsOfConvertPublicKeyToTonSafeFormat](mod_crypto.md#ParamsOfConvertPublicKeyToTonSafeFormat)

[ResultOfConvertPublicKeyToTonSafeFormat](mod_crypto.md#ResultOfConvertPublicKeyToTonSafeFormat)

[KeyPair](mod_crypto.md#KeyPair)

[ParamsOfSign](mod_crypto.md#ParamsOfSign)

[ResultOfSign](mod_crypto.md#ResultOfSign)

[ParamsOfVerifySignature](mod_crypto.md#ParamsOfVerifySignature)

[ResultOfVerifySignature](mod_crypto.md#ResultOfVerifySignature)

[ParamsOfHash](mod_crypto.md#ParamsOfHash)

[ResultOfHash](mod_crypto.md#ResultOfHash)

[ParamsOfScrypt](mod_crypto.md#ParamsOfScrypt)

[ResultOfScrypt](mod_crypto.md#ResultOfScrypt)

[ParamsOfNaclSignKeyPairFromSecret](mod_crypto.md#ParamsOfNaclSignKeyPairFromSecret)

[ParamsOfNaclSign](mod_crypto.md#ParamsOfNaclSign)

[ResultOfNaclSign](mod_crypto.md#ResultOfNaclSign)

[ParamsOfNaclSignOpen](mod_crypto.md#ParamsOfNaclSignOpen)

[ResultOfNaclSignOpen](mod_crypto.md#ResultOfNaclSignOpen)

[ResultOfNaclSignDetached](mod_crypto.md#ResultOfNaclSignDetached)

[ParamsOfNaclSignDetachedVerify](mod_crypto.md#ParamsOfNaclSignDetachedVerify)

[ResultOfNaclSignDetachedVerify](mod_crypto.md#ResultOfNaclSignDetachedVerify)

[ParamsOfNaclBoxKeyPairFromSecret](mod_crypto.md#ParamsOfNaclBoxKeyPairFromSecret)

[ParamsOfNaclBox](mod_crypto.md#ParamsOfNaclBox)

[ResultOfNaclBox](mod_crypto.md#ResultOfNaclBox)

[ParamsOfNaclBoxOpen](mod_crypto.md#ParamsOfNaclBoxOpen)

[ResultOfNaclBoxOpen](mod_crypto.md#ResultOfNaclBoxOpen)

[ParamsOfNaclSecretBox](mod_crypto.md#ParamsOfNaclSecretBox)

[ParamsOfNaclSecretBoxOpen](mod_crypto.md#ParamsOfNaclSecretBoxOpen)

[ParamsOfMnemonicWords](mod_crypto.md#ParamsOfMnemonicWords)

[ResultOfMnemonicWords](mod_crypto.md#ResultOfMnemonicWords)

[ParamsOfMnemonicFromRandom](mod_crypto.md#ParamsOfMnemonicFromRandom)

[ResultOfMnemonicFromRandom](mod_crypto.md#ResultOfMnemonicFromRandom)

[ParamsOfMnemonicFromEntropy](mod_crypto.md#ParamsOfMnemonicFromEntropy)

[ResultOfMnemonicFromEntropy](mod_crypto.md#ResultOfMnemonicFromEntropy)

[ParamsOfMnemonicVerify](mod_crypto.md#ParamsOfMnemonicVerify)

[ResultOfMnemonicVerify](mod_crypto.md#ResultOfMnemonicVerify)

[ParamsOfMnemonicDeriveSignKeys](mod_crypto.md#ParamsOfMnemonicDeriveSignKeys)

[ParamsOfHDKeyXPrvFromMnemonic](mod_crypto.md#ParamsOfHDKeyXPrvFromMnemonic)

[ResultOfHDKeyXPrvFromMnemonic](mod_crypto.md#ResultOfHDKeyXPrvFromMnemonic)

[ParamsOfHDKeyDeriveFromXPrv](mod_crypto.md#ParamsOfHDKeyDeriveFromXPrv)

[ResultOfHDKeyDeriveFromXPrv](mod_crypto.md#ResultOfHDKeyDeriveFromXPrv)

[ParamsOfHDKeyDeriveFromXPrvPath](mod_crypto.md#ParamsOfHDKeyDeriveFromXPrvPath)

[ResultOfHDKeyDeriveFromXPrvPath](mod_crypto.md#ResultOfHDKeyDeriveFromXPrvPath)

[ParamsOfHDKeySecretFromXPrv](mod_crypto.md#ParamsOfHDKeySecretFromXPrv)

[ResultOfHDKeySecretFromXPrv](mod_crypto.md#ResultOfHDKeySecretFromXPrv)

[ParamsOfHDKeyPublicFromXPrv](mod_crypto.md#ParamsOfHDKeyPublicFromXPrv)

[ResultOfHDKeyPublicFromXPrv](mod_crypto.md#ResultOfHDKeyPublicFromXPrv)

[ParamsOfChaCha20](mod_crypto.md#ParamsOfChaCha20)

[ResultOfChaCha20](mod_crypto.md#ResultOfChaCha20)

[RegisteredSigningBox](mod_crypto.md#RegisteredSigningBox)

[ParamsOfAppSigningBox](mod_crypto.md#ParamsOfAppSigningBox) – Signing box callbacks.

[ResultOfAppSigningBox](mod_crypto.md#ResultOfAppSigningBox) – Returning values from signing box callbacks.

[ResultOfSigningBoxGetPublicKey](mod_crypto.md#ResultOfSigningBoxGetPublicKey)

[ParamsOfSigningBoxSign](mod_crypto.md#ParamsOfSigningBoxSign)

[ResultOfSigningBoxSign](mod_crypto.md#ResultOfSigningBoxSign)

[RegisteredEncryptionBox](mod_crypto.md#RegisteredEncryptionBox)

[ParamsOfAppEncryptionBox](mod_crypto.md#ParamsOfAppEncryptionBox) – Encryption box callbacks.

[ResultOfAppEncryptionBox](mod_crypto.md#ResultOfAppEncryptionBox) – Returning values from signing box callbacks.

[ParamsOfEncryptionBoxGetInfo](mod_crypto.md#ParamsOfEncryptionBoxGetInfo)

[ResultOfEncryptionBoxGetInfo](mod_crypto.md#ResultOfEncryptionBoxGetInfo)

[ParamsOfEncryptionBoxEncrypt](mod_crypto.md#ParamsOfEncryptionBoxEncrypt)

[ResultOfEncryptionBoxEncrypt](mod_crypto.md#ResultOfEncryptionBoxEncrypt)

[ParamsOfEncryptionBoxDecrypt](mod_crypto.md#ParamsOfEncryptionBoxDecrypt)

[ResultOfEncryptionBoxDecrypt](mod_crypto.md#ResultOfEncryptionBoxDecrypt)

[ParamsOfCreateEncryptionBox](mod_crypto.md#ParamsOfCreateEncryptionBox)

[AppSigningBox](mod_crypto.md#AppSigningBox)

[AppEncryptionBox](mod_crypto.md#AppEncryptionBox)

## Functions

### factorize

Integer factorization

Performs prime factorization – decomposition of a composite number into a product of smaller prime integers \(factors\). See \[[https://en.wikipedia.org/wiki/Integer\_factorization](https://en.wikipedia.org/wiki/Integer_factorization)\]

```typescript
type ParamsOfFactorize = {
    composite: string
}

type ResultOfFactorize = {
    factors: string[]
}

function factorize(
    params: ParamsOfFactorize,
): Promise<ResultOfFactorize>;
```

#### Parameters

* `composite`: _string_ – Hexadecimal representation of u64 composite number.

#### Result

* `factors`: _string\[\]_ – Two factors of composite or empty if composite can't be factorized.

### modular\_power

Modular exponentiation

Performs modular exponentiation for big integers \(`base`^`exponent` mod `modulus`\). See \[[https://en.wikipedia.org/wiki/Modular\_exponentiation](https://en.wikipedia.org/wiki/Modular_exponentiation)\]

```typescript
type ParamsOfModularPower = {
    base: string,
    exponent: string,
    modulus: string
}

type ResultOfModularPower = {
    modular_power: string
}

function modular_power(
    params: ParamsOfModularPower,
): Promise<ResultOfModularPower>;
```

#### Parameters

* `base`: _string_ – `base` argument of calculation.
* `exponent`: _string_ – `exponent` argument of calculation.
* `modulus`: _string_ – `modulus` argument of calculation.

#### Result

* `modular_power`: _string_ – Result of modular exponentiation

### ton\_crc16

Calculates CRC16 using TON algorithm.

```typescript
type ParamsOfTonCrc16 = {
    data: string
}

type ResultOfTonCrc16 = {
    crc: number
}

function ton_crc16(
    params: ParamsOfTonCrc16,
): Promise<ResultOfTonCrc16>;
```

#### Parameters

* `data`: _string_ – Input data for CRC calculation.

  
  Encoded with `base64`.

#### Result

* `crc`: _number_ – Calculated CRC for input data.

### generate\_random\_bytes

Generates random byte array of the specified length and returns it in `base64` format

```typescript
type ParamsOfGenerateRandomBytes = {
    length: number
}

type ResultOfGenerateRandomBytes = {
    bytes: string
}

function generate_random_bytes(
    params: ParamsOfGenerateRandomBytes,
): Promise<ResultOfGenerateRandomBytes>;
```

#### Parameters

* `length`: _number_ – Size of random byte array.

#### Result

* `bytes`: _string_ – Generated bytes encoded in `base64`.

### convert\_public\_key\_to\_ton\_safe\_format

Converts public key to ton safe\_format

```typescript
type ParamsOfConvertPublicKeyToTonSafeFormat = {
    public_key: string
}

type ResultOfConvertPublicKeyToTonSafeFormat = {
    ton_public_key: string
}

function convert_public_key_to_ton_safe_format(
    params: ParamsOfConvertPublicKeyToTonSafeFormat,
): Promise<ResultOfConvertPublicKeyToTonSafeFormat>;
```

#### Parameters

* `public_key`: _string_ – Public key - 64 symbols hex string

#### Result

* `ton_public_key`: _string_ – Public key represented in TON safe format.

### generate\_random\_sign\_keys

Generates random ed25519 key pair.

```typescript
type KeyPair = {
    public: string,
    secret: string
}

function generate_random_sign_keys(): Promise<KeyPair>;
```

#### Result

* `public`: _string_ – Public key - 64 symbols hex string
* `secret`: _string_ – Private key - u64 symbols hex string

### sign

Signs a data using the provided keys.

```typescript
type ParamsOfSign = {
    unsigned: string,
    keys: KeyPair
}

type ResultOfSign = {
    signed: string,
    signature: string
}

function sign(
    params: ParamsOfSign,
): Promise<ResultOfSign>;
```

#### Parameters

* `unsigned`: _string_ – Data that must be signed encoded in `base64`.
* `keys`: [_KeyPair_](mod_crypto.md#KeyPair) – Sign keys.

#### Result

* `signed`: _string_ – Signed data combined with signature encoded in `base64`.
* `signature`: _string_ – Signature encoded in `hex`.

### verify\_signature

Verifies signed data using the provided public key. Raises error if verification is failed.

```typescript
type ParamsOfVerifySignature = {
    signed: string,
    public: string
}

type ResultOfVerifySignature = {
    unsigned: string
}

function verify_signature(
    params: ParamsOfVerifySignature,
): Promise<ResultOfVerifySignature>;
```

#### Parameters

* `signed`: _string_ – Signed data that must be verified encoded in `base64`.
* `public`: _string_ – Signer's public key - 64 symbols hex string

#### Result

* `unsigned`: _string_ – Unsigned data encoded in `base64`.

### sha256

Calculates SHA256 hash of the specified data.

```typescript
type ParamsOfHash = {
    data: string
}

type ResultOfHash = {
    hash: string
}

function sha256(
    params: ParamsOfHash,
): Promise<ResultOfHash>;
```

#### Parameters

* `data`: _string_ – Input data for hash calculation.

  
  Encoded with `base64`.

#### Result

* `hash`: _string_ – Hash of input `data`.

  
  Encoded with 'hex'.

### sha512

Calculates SHA512 hash of the specified data.

```typescript
type ParamsOfHash = {
    data: string
}

type ResultOfHash = {
    hash: string
}

function sha512(
    params: ParamsOfHash,
): Promise<ResultOfHash>;
```

#### Parameters

* `data`: _string_ – Input data for hash calculation.

  
  Encoded with `base64`.

#### Result

* `hash`: _string_ – Hash of input `data`.

  
  Encoded with 'hex'.

### scrypt

Perform `scrypt` encryption

Derives key from `password` and `key` using `scrypt` algorithm. See \[[https://en.wikipedia.org/wiki/Scrypt](https://en.wikipedia.org/wiki/Scrypt)\].

## Arguments

* `log_n` - The log2 of the Scrypt parameter `N`
* `r` - The Scrypt parameter `r`
* `p` - The Scrypt parameter `p`

  **Conditions**

* `log_n` must be less than `64`
* `r` must be greater than `0` and less than or equal to `4294967295`
* `p` must be greater than `0` and less than `4294967295`

  **Recommended values sufficient for most use-cases**

* `log_n = 15` \(`n = 32768`\)
* `r = 8`
* `p = 1`

```typescript
type ParamsOfScrypt = {
    password: string,
    salt: string,
    log_n: number,
    r: number,
    p: number,
    dk_len: number
}

type ResultOfScrypt = {
    key: string
}

function scrypt(
    params: ParamsOfScrypt,
): Promise<ResultOfScrypt>;
```

#### Parameters

* `password`: _string_ – The password bytes to be hashed. Must be encoded with `base64`.
* `salt`: _string_ – Salt bytes that modify the hash to protect against Rainbow table attacks. Must be encoded with `base64`.
* `log_n`: _number_ – CPU/memory cost parameter
* `r`: _number_ – The block size parameter, which fine-tunes sequential memory read size and performance.
* `p`: _number_ – Parallelization parameter.
* `dk_len`: _number_ – Intended output length in octets of the derived key.

#### Result

* `key`: _string_ – Derived key.

  
  Encoded with `hex`.

### nacl\_sign\_keypair\_from\_secret\_key

Generates a key pair for signing from the secret key

**NOTE:** In the result the secret key is actually the concatenation of secret and public keys \(128 symbols hex string\) by design of [NaCL](http://nacl.cr.yp.to/sign.html). See also [the stackexchange question](https://crypto.stackexchange.com/questions/54353/).

```typescript
type ParamsOfNaclSignKeyPairFromSecret = {
    secret: string
}

type KeyPair = {
    public: string,
    secret: string
}

function nacl_sign_keypair_from_secret_key(
    params: ParamsOfNaclSignKeyPairFromSecret,
): Promise<KeyPair>;
```

#### Parameters

* `secret`: _string_ – Secret key - unprefixed 0-padded to 64 symbols hex string

#### Result

* `public`: _string_ – Public key - 64 symbols hex string
* `secret`: _string_ – Private key - u64 symbols hex string

### nacl\_sign

Signs data using the signer's secret key.

```typescript
type ParamsOfNaclSign = {
    unsigned: string,
    secret: string
}

type ResultOfNaclSign = {
    signed: string
}

function nacl_sign(
    params: ParamsOfNaclSign,
): Promise<ResultOfNaclSign>;
```

#### Parameters

* `unsigned`: _string_ – Data that must be signed encoded in `base64`.
* `secret`: _string_ – Signer's secret key - unprefixed 0-padded to 128 symbols hex string \(concatenation of 64 symbols secret and 64 symbols public keys\). See `nacl_sign_keypair_from_secret_key`.

#### Result

* `signed`: _string_ – Signed data, encoded in `base64`.

### nacl\_sign\_open

Verifies the signature and returns the unsigned message

Verifies the signature in `signed` using the signer's public key `public` and returns the message `unsigned`.

If the signature fails verification, crypto\_sign\_open raises an exception.

```typescript
type ParamsOfNaclSignOpen = {
    signed: string,
    public: string
}

type ResultOfNaclSignOpen = {
    unsigned: string
}

function nacl_sign_open(
    params: ParamsOfNaclSignOpen,
): Promise<ResultOfNaclSignOpen>;
```

#### Parameters

* `signed`: _string_ – Signed data that must be unsigned.

  
  Encoded with `base64`.

* `public`: _string_ – Signer's public key - unprefixed 0-padded to 64 symbols hex string

#### Result

* `unsigned`: _string_ – Unsigned data, encoded in `base64`.

### nacl\_sign\_detached

Signs the message using the secret key and returns a signature.

Signs the message `unsigned` using the secret key `secret` and returns a signature `signature`.

```typescript
type ParamsOfNaclSign = {
    unsigned: string,
    secret: string
}

type ResultOfNaclSignDetached = {
    signature: string
}

function nacl_sign_detached(
    params: ParamsOfNaclSign,
): Promise<ResultOfNaclSignDetached>;
```

#### Parameters

* `unsigned`: _string_ – Data that must be signed encoded in `base64`.
* `secret`: _string_ – Signer's secret key - unprefixed 0-padded to 128 symbols hex string \(concatenation of 64 symbols secret and 64 symbols public keys\). See `nacl_sign_keypair_from_secret_key`.

#### Result

* `signature`: _string_ – Signature encoded in `hex`.

### nacl\_sign\_detached\_verify

Verifies the signature with public key and `unsigned` data.

```typescript
type ParamsOfNaclSignDetachedVerify = {
    unsigned: string,
    signature: string,
    public: string
}

type ResultOfNaclSignDetachedVerify = {
    succeeded: boolean
}

function nacl_sign_detached_verify(
    params: ParamsOfNaclSignDetachedVerify,
): Promise<ResultOfNaclSignDetachedVerify>;
```

#### Parameters

* `unsigned`: _string_ – Unsigned data that must be verified.

  
  Encoded with `base64`.

* `signature`: _string_ – Signature that must be verified.

  
  Encoded with `hex`.

* `public`: _string_ – Signer's public key - unprefixed 0-padded to 64 symbols hex string.

#### Result

* `succeeded`: _boolean_ – `true` if verification succeeded or `false` if it failed

### nacl\_box\_keypair

Generates a random NaCl key pair

```typescript
type KeyPair = {
    public: string,
    secret: string
}

function nacl_box_keypair(): Promise<KeyPair>;
```

#### Result

* `public`: _string_ – Public key - 64 symbols hex string
* `secret`: _string_ – Private key - u64 symbols hex string

### nacl\_box\_keypair\_from\_secret\_key

Generates key pair from a secret key

```typescript
type ParamsOfNaclBoxKeyPairFromSecret = {
    secret: string
}

type KeyPair = {
    public: string,
    secret: string
}

function nacl_box_keypair_from_secret_key(
    params: ParamsOfNaclBoxKeyPairFromSecret,
): Promise<KeyPair>;
```

#### Parameters

* `secret`: _string_ – Secret key - unprefixed 0-padded to 64 symbols hex string

#### Result

* `public`: _string_ – Public key - 64 symbols hex string
* `secret`: _string_ – Private key - u64 symbols hex string

### nacl\_box

Public key authenticated encryption

Encrypt and authenticate a message using the senders secret key, the receivers public key, and a nonce.

```typescript
type ParamsOfNaclBox = {
    decrypted: string,
    nonce: string,
    their_public: string,
    secret: string
}

type ResultOfNaclBox = {
    encrypted: string
}

function nacl_box(
    params: ParamsOfNaclBox,
): Promise<ResultOfNaclBox>;
```

#### Parameters

* `decrypted`: _string_ – Data that must be encrypted encoded in `base64`.
* `nonce`: _string_ – Nonce, encoded in `hex`
* `their_public`: _string_ – Receiver's public key - unprefixed 0-padded to 64 symbols hex string
* `secret`: _string_ – Sender's private key - unprefixed 0-padded to 64 symbols hex string

#### Result

* `encrypted`: _string_ – Encrypted data encoded in `base64`.

### nacl\_box\_open

Decrypt and verify the cipher text using the receivers secret key, the senders public key, and the nonce.

```typescript
type ParamsOfNaclBoxOpen = {
    encrypted: string,
    nonce: string,
    their_public: string,
    secret: string
}

type ResultOfNaclBoxOpen = {
    decrypted: string
}

function nacl_box_open(
    params: ParamsOfNaclBoxOpen,
): Promise<ResultOfNaclBoxOpen>;
```

#### Parameters

* `encrypted`: _string_ – Data that must be decrypted.

  
  Encoded with `base64`.

* `nonce`: _string_
* `their_public`: _string_ – Sender's public key - unprefixed 0-padded to 64 symbols hex string
* `secret`: _string_ – Receiver's private key - unprefixed 0-padded to 64 symbols hex string

#### Result

* `decrypted`: _string_ – Decrypted data encoded in `base64`.

### nacl\_secret\_box

Encrypt and authenticate message using nonce and secret key.

```typescript
type ParamsOfNaclSecretBox = {
    decrypted: string,
    nonce: string,
    key: string
}

type ResultOfNaclBox = {
    encrypted: string
}

function nacl_secret_box(
    params: ParamsOfNaclSecretBox,
): Promise<ResultOfNaclBox>;
```

#### Parameters

* `decrypted`: _string_ – Data that must be encrypted.

  
  Encoded with `base64`.

* `nonce`: _string_ – Nonce in `hex`
* `key`: _string_ – Secret key - unprefixed 0-padded to 64 symbols hex string

#### Result

* `encrypted`: _string_ – Encrypted data encoded in `base64`.

### nacl\_secret\_box\_open

Decrypts and verifies cipher text using `nonce` and secret `key`.

```typescript
type ParamsOfNaclSecretBoxOpen = {
    encrypted: string,
    nonce: string,
    key: string
}

type ResultOfNaclBoxOpen = {
    decrypted: string
}

function nacl_secret_box_open(
    params: ParamsOfNaclSecretBoxOpen,
): Promise<ResultOfNaclBoxOpen>;
```

#### Parameters

* `encrypted`: _string_ – Data that must be decrypted.

  
  Encoded with `base64`.

* `nonce`: _string_ – Nonce in `hex`
* `key`: _string_ – Public key - unprefixed 0-padded to 64 symbols hex string

#### Result

* `decrypted`: _string_ – Decrypted data encoded in `base64`.

### mnemonic\_words

Prints the list of words from the specified dictionary

```typescript
type ParamsOfMnemonicWords = {
    dictionary?: number
}

type ResultOfMnemonicWords = {
    words: string
}

function mnemonic_words(
    params: ParamsOfMnemonicWords,
): Promise<ResultOfMnemonicWords>;
```

#### Parameters

* `dictionary`?: _number_ – Dictionary identifier

#### Result

* `words`: _string_ – The list of mnemonic words

### mnemonic\_from\_random

Generates a random mnemonic

Generates a random mnemonic from the specified dictionary and word count

```typescript
type ParamsOfMnemonicFromRandom = {
    dictionary?: number,
    word_count?: number
}

type ResultOfMnemonicFromRandom = {
    phrase: string
}

function mnemonic_from_random(
    params: ParamsOfMnemonicFromRandom,
): Promise<ResultOfMnemonicFromRandom>;
```

#### Parameters

* `dictionary`?: _number_ – Dictionary identifier
* `word_count`?: _number_ – Mnemonic word count

#### Result

* `phrase`: _string_ – String of mnemonic words

### mnemonic\_from\_entropy

Generates mnemonic from pre-generated entropy

```typescript
type ParamsOfMnemonicFromEntropy = {
    entropy: string,
    dictionary?: number,
    word_count?: number
}

type ResultOfMnemonicFromEntropy = {
    phrase: string
}

function mnemonic_from_entropy(
    params: ParamsOfMnemonicFromEntropy,
): Promise<ResultOfMnemonicFromEntropy>;
```

#### Parameters

* `entropy`: _string_ – Entropy bytes.

  
  Hex encoded.

* `dictionary`?: _number_ – Dictionary identifier
* `word_count`?: _number_ – Mnemonic word count

#### Result

* `phrase`: _string_ – Phrase

### mnemonic\_verify

Validates a mnemonic phrase

The phrase supplied will be checked for word length and validated according to the checksum specified in BIP0039.

```typescript
type ParamsOfMnemonicVerify = {
    phrase: string,
    dictionary?: number,
    word_count?: number
}

type ResultOfMnemonicVerify = {
    valid: boolean
}

function mnemonic_verify(
    params: ParamsOfMnemonicVerify,
): Promise<ResultOfMnemonicVerify>;
```

#### Parameters

* `phrase`: _string_ – Phrase
* `dictionary`?: _number_ – Dictionary identifier
* `word_count`?: _number_ – Word count

#### Result

* `valid`: _boolean_ – Flag indicating if the mnemonic is valid or not

### mnemonic\_derive\_sign\_keys

Derives a key pair for signing from the seed phrase

Validates the seed phrase, generates master key and then derives the key pair from the master key and the specified path

```typescript
type ParamsOfMnemonicDeriveSignKeys = {
    phrase: string,
    path?: string,
    dictionary?: number,
    word_count?: number
}

type KeyPair = {
    public: string,
    secret: string
}

function mnemonic_derive_sign_keys(
    params: ParamsOfMnemonicDeriveSignKeys,
): Promise<KeyPair>;
```

#### Parameters

* `phrase`: _string_ – Phrase
* `path`?: _string_ – Derivation path, for instance "m/44'/396'/0'/0/0"
* `dictionary`?: _number_ – Dictionary identifier
* `word_count`?: _number_ – Word count

#### Result

* `public`: _string_ – Public key - 64 symbols hex string
* `secret`: _string_ – Private key - u64 symbols hex string

### hdkey\_xprv\_from\_mnemonic

Generates an extended master private key that will be the root for all the derived keys

```typescript
type ParamsOfHDKeyXPrvFromMnemonic = {
    phrase: string,
    dictionary?: number,
    word_count?: number
}

type ResultOfHDKeyXPrvFromMnemonic = {
    xprv: string
}

function hdkey_xprv_from_mnemonic(
    params: ParamsOfHDKeyXPrvFromMnemonic,
): Promise<ResultOfHDKeyXPrvFromMnemonic>;
```

#### Parameters

* `phrase`: _string_ – String with seed phrase
* `dictionary`?: _number_ – Dictionary identifier
* `word_count`?: _number_ – Mnemonic word count

#### Result

* `xprv`: _string_ – Serialized extended master private key

### hdkey\_derive\_from\_xprv

Returns extended private key derived from the specified extended private key and child index

```typescript
type ParamsOfHDKeyDeriveFromXPrv = {
    xprv: string,
    child_index: number,
    hardened: boolean
}

type ResultOfHDKeyDeriveFromXPrv = {
    xprv: string
}

function hdkey_derive_from_xprv(
    params: ParamsOfHDKeyDeriveFromXPrv,
): Promise<ResultOfHDKeyDeriveFromXPrv>;
```

#### Parameters

* `xprv`: _string_ – Serialized extended private key
* `child_index`: _number_ – Child index \(see BIP-0032\)
* `hardened`: _boolean_ – Indicates the derivation of hardened/not-hardened key \(see BIP-0032\)

#### Result

* `xprv`: _string_ – Serialized extended private key

### hdkey\_derive\_from\_xprv\_path

Derives the extended private key from the specified key and path

```typescript
type ParamsOfHDKeyDeriveFromXPrvPath = {
    xprv: string,
    path: string
}

type ResultOfHDKeyDeriveFromXPrvPath = {
    xprv: string
}

function hdkey_derive_from_xprv_path(
    params: ParamsOfHDKeyDeriveFromXPrvPath,
): Promise<ResultOfHDKeyDeriveFromXPrvPath>;
```

#### Parameters

* `xprv`: _string_ – Serialized extended private key
* `path`: _string_ – Derivation path, for instance "m/44'/396'/0'/0/0"

#### Result

* `xprv`: _string_ – Derived serialized extended private key

### hdkey\_secret\_from\_xprv

Extracts the private key from the serialized extended private key

```typescript
type ParamsOfHDKeySecretFromXPrv = {
    xprv: string
}

type ResultOfHDKeySecretFromXPrv = {
    secret: string
}

function hdkey_secret_from_xprv(
    params: ParamsOfHDKeySecretFromXPrv,
): Promise<ResultOfHDKeySecretFromXPrv>;
```

#### Parameters

* `xprv`: _string_ – Serialized extended private key

#### Result

* `secret`: _string_ – Private key - 64 symbols hex string

### hdkey\_public\_from\_xprv

Extracts the public key from the serialized extended private key

```typescript
type ParamsOfHDKeyPublicFromXPrv = {
    xprv: string
}

type ResultOfHDKeyPublicFromXPrv = {
    public: string
}

function hdkey_public_from_xprv(
    params: ParamsOfHDKeyPublicFromXPrv,
): Promise<ResultOfHDKeyPublicFromXPrv>;
```

#### Parameters

* `xprv`: _string_ – Serialized extended private key

#### Result

* `public`: _string_ – Public key - 64 symbols hex string

### chacha20

Performs symmetric `chacha20` encryption.

```typescript
type ParamsOfChaCha20 = {
    data: string,
    key: string,
    nonce: string
}

type ResultOfChaCha20 = {
    data: string
}

function chacha20(
    params: ParamsOfChaCha20,
): Promise<ResultOfChaCha20>;
```

#### Parameters

* `data`: _string_ – Source data to be encrypted or decrypted.

  
  Must be encoded with `base64`.

* `key`: _string_ – 256-bit key.

  
  Must be encoded with `hex`.

* `nonce`: _string_ – 96-bit nonce.

  
  Must be encoded with `hex`.

#### Result

* `data`: _string_ – Encrypted/decrypted data.

  
  Encoded with `base64`.

### register\_signing\_box

Register an application implemented signing box.

```typescript
type RegisteredSigningBox = {
    handle: SigningBoxHandle
}

function register_signing_box(
    obj: AppSigningBox,
): Promise<RegisteredSigningBox>;
```

#### Result

* `handle`: [_SigningBoxHandle_](mod_crypto.md#SigningBoxHandle) – Handle of the signing box.

### get\_signing\_box

Creates a default signing box implementation.

```typescript
type KeyPair = {
    public: string,
    secret: string
}

type RegisteredSigningBox = {
    handle: SigningBoxHandle
}

function get_signing_box(
    params: KeyPair,
): Promise<RegisteredSigningBox>;
```

#### Parameters

* `public`: _string_ – Public key - 64 symbols hex string
* `secret`: _string_ – Private key - u64 symbols hex string

#### Result

* `handle`: [_SigningBoxHandle_](mod_crypto.md#SigningBoxHandle) – Handle of the signing box.

### signing\_box\_get\_public\_key

Returns public key of signing key pair.

```typescript
type RegisteredSigningBox = {
    handle: SigningBoxHandle
}

type ResultOfSigningBoxGetPublicKey = {
    pubkey: string
}

function signing_box_get_public_key(
    params: RegisteredSigningBox,
): Promise<ResultOfSigningBoxGetPublicKey>;
```

#### Parameters

* `handle`: [_SigningBoxHandle_](mod_crypto.md#SigningBoxHandle) – Handle of the signing box.

#### Result

* `pubkey`: _string_ – Public key of signing box.

  
  Encoded with hex

### signing\_box\_sign

Returns signed user data.

```typescript
type ParamsOfSigningBoxSign = {
    signing_box: SigningBoxHandle,
    unsigned: string
}

type ResultOfSigningBoxSign = {
    signature: string
}

function signing_box_sign(
    params: ParamsOfSigningBoxSign,
): Promise<ResultOfSigningBoxSign>;
```

#### Parameters

* `signing_box`: [_SigningBoxHandle_](mod_crypto.md#SigningBoxHandle) – Signing Box handle.
* `unsigned`: _string_ – Unsigned user data.

  
  Must be encoded with `base64`.

#### Result

* `signature`: _string_ – Data signature.

  
  Encoded with `hex`.

### remove\_signing\_box

Removes signing box from SDK.

```typescript
type RegisteredSigningBox = {
    handle: SigningBoxHandle
}

function remove_signing_box(
    params: RegisteredSigningBox,
): Promise<void>;
```

#### Parameters

* `handle`: [_SigningBoxHandle_](mod_crypto.md#SigningBoxHandle) – Handle of the signing box.

### register\_encryption\_box

Register an application implemented encryption box.

```typescript
type RegisteredEncryptionBox = {
    handle: EncryptionBoxHandle
}

function register_encryption_box(
    obj: AppEncryptionBox,
): Promise<RegisteredEncryptionBox>;
```

#### Result

* `handle`: [_EncryptionBoxHandle_](mod_crypto.md#EncryptionBoxHandle) – Handle of the encryption box

### remove\_encryption\_box

Removes encryption box from SDK

```typescript
type RegisteredEncryptionBox = {
    handle: EncryptionBoxHandle
}

function remove_encryption_box(
    params: RegisteredEncryptionBox,
): Promise<void>;
```

#### Parameters

* `handle`: [_EncryptionBoxHandle_](mod_crypto.md#EncryptionBoxHandle) – Handle of the encryption box

### encryption\_box\_get\_info

Queries info from the given encryption box

```typescript
type ParamsOfEncryptionBoxGetInfo = {
    encryption_box: EncryptionBoxHandle
}

type ResultOfEncryptionBoxGetInfo = {
    info: EncryptionBoxInfo
}

function encryption_box_get_info(
    params: ParamsOfEncryptionBoxGetInfo,
): Promise<ResultOfEncryptionBoxGetInfo>;
```

#### Parameters

* `encryption_box`: [_EncryptionBoxHandle_](mod_crypto.md#EncryptionBoxHandle) – Encryption box handle

#### Result

* `info`: [_EncryptionBoxInfo_](mod_crypto.md#EncryptionBoxInfo) – Encryption box information

### encryption\_box\_encrypt

Encrypts data using given encryption box Note.

Block cipher algorithms pad data to cipher block size so encrypted data can be longer then original data. Client should store the original data size after encryption and use it after decryption to retrieve the original data from decrypted data.

```typescript
type ParamsOfEncryptionBoxEncrypt = {
    encryption_box: EncryptionBoxHandle,
    data: string
}

type ResultOfEncryptionBoxEncrypt = {
    data: string
}

function encryption_box_encrypt(
    params: ParamsOfEncryptionBoxEncrypt,
): Promise<ResultOfEncryptionBoxEncrypt>;
```

#### Parameters

* `encryption_box`: [_EncryptionBoxHandle_](mod_crypto.md#EncryptionBoxHandle) – Encryption box handle
* `data`: _string_ – Data to be encrypted, encoded in Base64

#### Result

* `data`: _string_ – Encrypted data, encoded in Base64.

  
  Padded to cipher block size

### encryption\_box\_decrypt

Decrypts data using given encryption box Note.

Block cipher algorithms pad data to cipher block size so encrypted data can be longer then original data. Client should store the original data size after encryption and use it after decryption to retrieve the original data from decrypted data.

```typescript
type ParamsOfEncryptionBoxDecrypt = {
    encryption_box: EncryptionBoxHandle,
    data: string
}

type ResultOfEncryptionBoxDecrypt = {
    data: string
}

function encryption_box_decrypt(
    params: ParamsOfEncryptionBoxDecrypt,
): Promise<ResultOfEncryptionBoxDecrypt>;
```

#### Parameters

* `encryption_box`: [_EncryptionBoxHandle_](mod_crypto.md#EncryptionBoxHandle) – Encryption box handle
* `data`: _string_ – Data to be decrypted, encoded in Base64

#### Result

* `data`: _string_ – Decrypted data, encoded in Base64.

### create\_encryption\_box

Creates encryption box with specified algorithm

```typescript
type ParamsOfCreateEncryptionBox = {
    algorithm: EncryptionAlgorithm
}

type RegisteredEncryptionBox = {
    handle: EncryptionBoxHandle
}

function create_encryption_box(
    params: ParamsOfCreateEncryptionBox,
): Promise<RegisteredEncryptionBox>;
```

#### Parameters

* `algorithm`: [_EncryptionAlgorithm_](mod_crypto.md#EncryptionAlgorithm) – Encryption algorithm specifier including cipher parameters \(key, IV, etc\)

#### Result

* `handle`: [_EncryptionBoxHandle_](mod_crypto.md#EncryptionBoxHandle) – Handle of the encryption box

## Types

### CryptoErrorCode

```typescript
enum CryptoErrorCode {
    InvalidPublicKey = 100,
    InvalidSecretKey = 101,
    InvalidKey = 102,
    InvalidFactorizeChallenge = 106,
    InvalidBigInt = 107,
    ScryptFailed = 108,
    InvalidKeySize = 109,
    NaclSecretBoxFailed = 110,
    NaclBoxFailed = 111,
    NaclSignFailed = 112,
    Bip39InvalidEntropy = 113,
    Bip39InvalidPhrase = 114,
    Bip32InvalidKey = 115,
    Bip32InvalidDerivePath = 116,
    Bip39InvalidDictionary = 117,
    Bip39InvalidWordCount = 118,
    MnemonicGenerationFailed = 119,
    MnemonicFromEntropyFailed = 120,
    SigningBoxNotRegistered = 121,
    InvalidSignature = 122,
    EncryptionBoxNotRegistered = 123,
    InvalidIvSize = 124,
    UnsupportedCipherMode = 125,
    CannotCreateCipher = 126,
    EncryptDataError = 127,
    DecryptDataError = 128,
    IvRequired = 129
}
```

One of the following value:

* `InvalidPublicKey = 100`
* `InvalidSecretKey = 101`
* `InvalidKey = 102`
* `InvalidFactorizeChallenge = 106`
* `InvalidBigInt = 107`
* `ScryptFailed = 108`
* `InvalidKeySize = 109`
* `NaclSecretBoxFailed = 110`
* `NaclBoxFailed = 111`
* `NaclSignFailed = 112`
* `Bip39InvalidEntropy = 113`
* `Bip39InvalidPhrase = 114`
* `Bip32InvalidKey = 115`
* `Bip32InvalidDerivePath = 116`
* `Bip39InvalidDictionary = 117`
* `Bip39InvalidWordCount = 118`
* `MnemonicGenerationFailed = 119`
* `MnemonicFromEntropyFailed = 120`
* `SigningBoxNotRegistered = 121`
* `InvalidSignature = 122`
* `EncryptionBoxNotRegistered = 123`
* `InvalidIvSize = 124`
* `UnsupportedCipherMode = 125`
* `CannotCreateCipher = 126`
* `EncryptDataError = 127`
* `DecryptDataError = 128`
* `IvRequired = 129`

### SigningBoxHandle

```typescript
type SigningBoxHandle = number
```

### EncryptionBoxHandle

```typescript
type EncryptionBoxHandle = number
```

### EncryptionBoxInfo

Encryption box information

```typescript
type EncryptionBoxInfo = {
    hdpath?: string,
    algorithm?: string,
    options?: any,
    public?: any
}
```

* `hdpath`?: _string_ – Derivation path, for instance "m/44'/396'/0'/0/0"
* `algorithm`?: _string_ – Cryptographic algorithm, used by this encryption box
* `options`?: _any_ – Options, depends on algorithm and specific encryption box implementation
* `public`?: _any_ – Public information, depends on algorithm

### EncryptionAlgorithm

```typescript
type EncryptionAlgorithm = ({
    type: 'AES'
} & AesParams)
```

Depends on value of the `type` field.

When _type_ is _'AES'_

* `mode`: [_CipherMode_](mod_crypto.md#CipherMode)
* `key`: _string_
* `iv`?: _string_

Variant constructors:

```typescript
function encryptionAlgorithmAES(params: AesParams): EncryptionAlgorithm;
```

### CipherMode

```typescript
enum CipherMode {
    CBC = "CBC",
    CFB = "CFB",
    CTR = "CTR",
    ECB = "ECB",
    OFB = "OFB"
}
```

One of the following value:

* `CBC = "CBC"`
* `CFB = "CFB"`
* `CTR = "CTR"`
* `ECB = "ECB"`
* `OFB = "OFB"`

### AesParams

```typescript
type AesParams = {
    mode: CipherMode,
    key: string,
    iv?: string
}
```

* `mode`: [_CipherMode_](mod_crypto.md#CipherMode)
* `key`: _string_
* `iv`?: _string_

### AesInfo

```typescript
type AesInfo = {
    mode: CipherMode,
    iv?: string
}
```

* `mode`: [_CipherMode_](mod_crypto.md#CipherMode)
* `iv`?: _string_

### ParamsOfFactorize

```typescript
type ParamsOfFactorize = {
    composite: string
}
```

* `composite`: _string_ – Hexadecimal representation of u64 composite number.

### ResultOfFactorize

```typescript
type ResultOfFactorize = {
    factors: string[]
}
```

* `factors`: _string\[\]_ – Two factors of composite or empty if composite can't be factorized.

### ParamsOfModularPower

```typescript
type ParamsOfModularPower = {
    base: string,
    exponent: string,
    modulus: string
}
```

* `base`: _string_ – `base` argument of calculation.
* `exponent`: _string_ – `exponent` argument of calculation.
* `modulus`: _string_ – `modulus` argument of calculation.

### ResultOfModularPower

```typescript
type ResultOfModularPower = {
    modular_power: string
}
```

* `modular_power`: _string_ – Result of modular exponentiation

### ParamsOfTonCrc16

```typescript
type ParamsOfTonCrc16 = {
    data: string
}
```

* `data`: _string_ – Input data for CRC calculation.

  
  Encoded with `base64`.

### ResultOfTonCrc16

```typescript
type ResultOfTonCrc16 = {
    crc: number
}
```

* `crc`: _number_ – Calculated CRC for input data.

### ParamsOfGenerateRandomBytes

```typescript
type ParamsOfGenerateRandomBytes = {
    length: number
}
```

* `length`: _number_ – Size of random byte array.

### ResultOfGenerateRandomBytes

```typescript
type ResultOfGenerateRandomBytes = {
    bytes: string
}
```

* `bytes`: _string_ – Generated bytes encoded in `base64`.

### ParamsOfConvertPublicKeyToTonSafeFormat

```typescript
type ParamsOfConvertPublicKeyToTonSafeFormat = {
    public_key: string
}
```

* `public_key`: _string_ – Public key - 64 symbols hex string

### ResultOfConvertPublicKeyToTonSafeFormat

```typescript
type ResultOfConvertPublicKeyToTonSafeFormat = {
    ton_public_key: string
}
```

* `ton_public_key`: _string_ – Public key represented in TON safe format.

### KeyPair

```typescript
type KeyPair = {
    public: string,
    secret: string
}
```

* `public`: _string_ – Public key - 64 symbols hex string
* `secret`: _string_ – Private key - u64 symbols hex string

### ParamsOfSign

```typescript
type ParamsOfSign = {
    unsigned: string,
    keys: KeyPair
}
```

* `unsigned`: _string_ – Data that must be signed encoded in `base64`.
* `keys`: [_KeyPair_](mod_crypto.md#KeyPair) – Sign keys.

### ResultOfSign

```typescript
type ResultOfSign = {
    signed: string,
    signature: string
}
```

* `signed`: _string_ – Signed data combined with signature encoded in `base64`.
* `signature`: _string_ – Signature encoded in `hex`.

### ParamsOfVerifySignature

```typescript
type ParamsOfVerifySignature = {
    signed: string,
    public: string
}
```

* `signed`: _string_ – Signed data that must be verified encoded in `base64`.
* `public`: _string_ – Signer's public key - 64 symbols hex string

### ResultOfVerifySignature

```typescript
type ResultOfVerifySignature = {
    unsigned: string
}
```

* `unsigned`: _string_ – Unsigned data encoded in `base64`.

### ParamsOfHash

```typescript
type ParamsOfHash = {
    data: string
}
```

* `data`: _string_ – Input data for hash calculation.

  
  Encoded with `base64`.

### ResultOfHash

```typescript
type ResultOfHash = {
    hash: string
}
```

* `hash`: _string_ – Hash of input `data`.

  
  Encoded with 'hex'.

### ParamsOfScrypt

```typescript
type ParamsOfScrypt = {
    password: string,
    salt: string,
    log_n: number,
    r: number,
    p: number,
    dk_len: number
}
```

* `password`: _string_ – The password bytes to be hashed. Must be encoded with `base64`.
* `salt`: _string_ – Salt bytes that modify the hash to protect against Rainbow table attacks. Must be encoded with `base64`.
* `log_n`: _number_ – CPU/memory cost parameter
* `r`: _number_ – The block size parameter, which fine-tunes sequential memory read size and performance.
* `p`: _number_ – Parallelization parameter.
* `dk_len`: _number_ – Intended output length in octets of the derived key.

### ResultOfScrypt

```typescript
type ResultOfScrypt = {
    key: string
}
```

* `key`: _string_ – Derived key.

  
  Encoded with `hex`.

### ParamsOfNaclSignKeyPairFromSecret

```typescript
type ParamsOfNaclSignKeyPairFromSecret = {
    secret: string
}
```

* `secret`: _string_ – Secret key - unprefixed 0-padded to 64 symbols hex string

### ParamsOfNaclSign

```typescript
type ParamsOfNaclSign = {
    unsigned: string,
    secret: string
}
```

* `unsigned`: _string_ – Data that must be signed encoded in `base64`.
* `secret`: _string_ – Signer's secret key - unprefixed 0-padded to 128 symbols hex string \(concatenation of 64 symbols secret and 64 symbols public keys\). See `nacl_sign_keypair_from_secret_key`.

### ResultOfNaclSign

```typescript
type ResultOfNaclSign = {
    signed: string
}
```

* `signed`: _string_ – Signed data, encoded in `base64`.

### ParamsOfNaclSignOpen

```typescript
type ParamsOfNaclSignOpen = {
    signed: string,
    public: string
}
```

* `signed`: _string_ – Signed data that must be unsigned.

  
  Encoded with `base64`.

* `public`: _string_ – Signer's public key - unprefixed 0-padded to 64 symbols hex string

### ResultOfNaclSignOpen

```typescript
type ResultOfNaclSignOpen = {
    unsigned: string
}
```

* `unsigned`: _string_ – Unsigned data, encoded in `base64`.

### ResultOfNaclSignDetached

```typescript
type ResultOfNaclSignDetached = {
    signature: string
}
```

* `signature`: _string_ – Signature encoded in `hex`.

### ParamsOfNaclSignDetachedVerify

```typescript
type ParamsOfNaclSignDetachedVerify = {
    unsigned: string,
    signature: string,
    public: string
}
```

* `unsigned`: _string_ – Unsigned data that must be verified.

  
  Encoded with `base64`.

* `signature`: _string_ – Signature that must be verified.

  
  Encoded with `hex`.

* `public`: _string_ – Signer's public key - unprefixed 0-padded to 64 symbols hex string.

### ResultOfNaclSignDetachedVerify

```typescript
type ResultOfNaclSignDetachedVerify = {
    succeeded: boolean
}
```

* `succeeded`: _boolean_ – `true` if verification succeeded or `false` if it failed

### ParamsOfNaclBoxKeyPairFromSecret

```typescript
type ParamsOfNaclBoxKeyPairFromSecret = {
    secret: string
}
```

* `secret`: _string_ – Secret key - unprefixed 0-padded to 64 symbols hex string

### ParamsOfNaclBox

```typescript
type ParamsOfNaclBox = {
    decrypted: string,
    nonce: string,
    their_public: string,
    secret: string
}
```

* `decrypted`: _string_ – Data that must be encrypted encoded in `base64`.
* `nonce`: _string_ – Nonce, encoded in `hex`
* `their_public`: _string_ – Receiver's public key - unprefixed 0-padded to 64 symbols hex string
* `secret`: _string_ – Sender's private key - unprefixed 0-padded to 64 symbols hex string

### ResultOfNaclBox

```typescript
type ResultOfNaclBox = {
    encrypted: string
}
```

* `encrypted`: _string_ – Encrypted data encoded in `base64`.

### ParamsOfNaclBoxOpen

```typescript
type ParamsOfNaclBoxOpen = {
    encrypted: string,
    nonce: string,
    their_public: string,
    secret: string
}
```

* `encrypted`: _string_ – Data that must be decrypted.

  
  Encoded with `base64`.

* `nonce`: _string_
* `their_public`: _string_ – Sender's public key - unprefixed 0-padded to 64 symbols hex string
* `secret`: _string_ – Receiver's private key - unprefixed 0-padded to 64 symbols hex string

### ResultOfNaclBoxOpen

```typescript
type ResultOfNaclBoxOpen = {
    decrypted: string
}
```

* `decrypted`: _string_ – Decrypted data encoded in `base64`.

### ParamsOfNaclSecretBox

```typescript
type ParamsOfNaclSecretBox = {
    decrypted: string,
    nonce: string,
    key: string
}
```

* `decrypted`: _string_ – Data that must be encrypted.

  
  Encoded with `base64`.

* `nonce`: _string_ – Nonce in `hex`
* `key`: _string_ – Secret key - unprefixed 0-padded to 64 symbols hex string

### ParamsOfNaclSecretBoxOpen

```typescript
type ParamsOfNaclSecretBoxOpen = {
    encrypted: string,
    nonce: string,
    key: string
}
```

* `encrypted`: _string_ – Data that must be decrypted.

  
  Encoded with `base64`.

* `nonce`: _string_ – Nonce in `hex`
* `key`: _string_ – Public key - unprefixed 0-padded to 64 symbols hex string

### ParamsOfMnemonicWords

```typescript
type ParamsOfMnemonicWords = {
    dictionary?: number
}
```

* `dictionary`?: _number_ – Dictionary identifier

### ResultOfMnemonicWords

```typescript
type ResultOfMnemonicWords = {
    words: string
}
```

* `words`: _string_ – The list of mnemonic words

### ParamsOfMnemonicFromRandom

```typescript
type ParamsOfMnemonicFromRandom = {
    dictionary?: number,
    word_count?: number
}
```

* `dictionary`?: _number_ – Dictionary identifier
* `word_count`?: _number_ – Mnemonic word count

### ResultOfMnemonicFromRandom

```typescript
type ResultOfMnemonicFromRandom = {
    phrase: string
}
```

* `phrase`: _string_ – String of mnemonic words

### ParamsOfMnemonicFromEntropy

```typescript
type ParamsOfMnemonicFromEntropy = {
    entropy: string,
    dictionary?: number,
    word_count?: number
}
```

* `entropy`: _string_ – Entropy bytes.

  
  Hex encoded.

* `dictionary`?: _number_ – Dictionary identifier
* `word_count`?: _number_ – Mnemonic word count

### ResultOfMnemonicFromEntropy

```typescript
type ResultOfMnemonicFromEntropy = {
    phrase: string
}
```

* `phrase`: _string_ – Phrase

### ParamsOfMnemonicVerify

```typescript
type ParamsOfMnemonicVerify = {
    phrase: string,
    dictionary?: number,
    word_count?: number
}
```

* `phrase`: _string_ – Phrase
* `dictionary`?: _number_ – Dictionary identifier
* `word_count`?: _number_ – Word count

### ResultOfMnemonicVerify

```typescript
type ResultOfMnemonicVerify = {
    valid: boolean
}
```

* `valid`: _boolean_ – Flag indicating if the mnemonic is valid or not

### ParamsOfMnemonicDeriveSignKeys

```typescript
type ParamsOfMnemonicDeriveSignKeys = {
    phrase: string,
    path?: string,
    dictionary?: number,
    word_count?: number
}
```

* `phrase`: _string_ – Phrase
* `path`?: _string_ – Derivation path, for instance "m/44'/396'/0'/0/0"
* `dictionary`?: _number_ – Dictionary identifier
* `word_count`?: _number_ – Word count

### ParamsOfHDKeyXPrvFromMnemonic

```typescript
type ParamsOfHDKeyXPrvFromMnemonic = {
    phrase: string,
    dictionary?: number,
    word_count?: number
}
```

* `phrase`: _string_ – String with seed phrase
* `dictionary`?: _number_ – Dictionary identifier
* `word_count`?: _number_ – Mnemonic word count

### ResultOfHDKeyXPrvFromMnemonic

```typescript
type ResultOfHDKeyXPrvFromMnemonic = {
    xprv: string
}
```

* `xprv`: _string_ – Serialized extended master private key

### ParamsOfHDKeyDeriveFromXPrv

```typescript
type ParamsOfHDKeyDeriveFromXPrv = {
    xprv: string,
    child_index: number,
    hardened: boolean
}
```

* `xprv`: _string_ – Serialized extended private key
* `child_index`: _number_ – Child index \(see BIP-0032\)
* `hardened`: _boolean_ – Indicates the derivation of hardened/not-hardened key \(see BIP-0032\)

### ResultOfHDKeyDeriveFromXPrv

```typescript
type ResultOfHDKeyDeriveFromXPrv = {
    xprv: string
}
```

* `xprv`: _string_ – Serialized extended private key

### ParamsOfHDKeyDeriveFromXPrvPath

```typescript
type ParamsOfHDKeyDeriveFromXPrvPath = {
    xprv: string,
    path: string
}
```

* `xprv`: _string_ – Serialized extended private key
* `path`: _string_ – Derivation path, for instance "m/44'/396'/0'/0/0"

### ResultOfHDKeyDeriveFromXPrvPath

```typescript
type ResultOfHDKeyDeriveFromXPrvPath = {
    xprv: string
}
```

* `xprv`: _string_ – Derived serialized extended private key

### ParamsOfHDKeySecretFromXPrv

```typescript
type ParamsOfHDKeySecretFromXPrv = {
    xprv: string
}
```

* `xprv`: _string_ – Serialized extended private key

### ResultOfHDKeySecretFromXPrv

```typescript
type ResultOfHDKeySecretFromXPrv = {
    secret: string
}
```

* `secret`: _string_ – Private key - 64 symbols hex string

### ParamsOfHDKeyPublicFromXPrv

```typescript
type ParamsOfHDKeyPublicFromXPrv = {
    xprv: string
}
```

* `xprv`: _string_ – Serialized extended private key

### ResultOfHDKeyPublicFromXPrv

```typescript
type ResultOfHDKeyPublicFromXPrv = {
    public: string
}
```

* `public`: _string_ – Public key - 64 symbols hex string

### ParamsOfChaCha20

```typescript
type ParamsOfChaCha20 = {
    data: string,
    key: string,
    nonce: string
}
```

* `data`: _string_ – Source data to be encrypted or decrypted.

  
  Must be encoded with `base64`.

* `key`: _string_ – 256-bit key.

  
  Must be encoded with `hex`.

* `nonce`: _string_ – 96-bit nonce.

  
  Must be encoded with `hex`.

### ResultOfChaCha20

```typescript
type ResultOfChaCha20 = {
    data: string
}
```

* `data`: _string_ – Encrypted/decrypted data.

  
  Encoded with `base64`.

### RegisteredSigningBox

```typescript
type RegisteredSigningBox = {
    handle: SigningBoxHandle
}
```

* `handle`: [_SigningBoxHandle_](mod_crypto.md#SigningBoxHandle) – Handle of the signing box.

### ParamsOfAppSigningBox

Signing box callbacks.

```typescript
type ParamsOfAppSigningBox = {
    type: 'GetPublicKey'
} | {
    type: 'Sign'
    unsigned: string
}
```

Depends on value of the `type` field.

When _type_ is _'GetPublicKey'_

Get signing box public key

When _type_ is _'Sign'_

Sign data

* `unsigned`: _string_ – Data to sign encoded as base64

Variant constructors:

```typescript
function paramsOfAppSigningBoxGetPublicKey(): ParamsOfAppSigningBox;
function paramsOfAppSigningBoxSign(unsigned: string): ParamsOfAppSigningBox;
```

### ResultOfAppSigningBox

Returning values from signing box callbacks.

```typescript
type ResultOfAppSigningBox = {
    type: 'GetPublicKey'
    public_key: string
} | {
    type: 'Sign'
    signature: string
}
```

Depends on value of the `type` field.

When _type_ is _'GetPublicKey'_

Result of getting public key

* `public_key`: _string_ – Signing box public key

When _type_ is _'Sign'_

Result of signing data

* `signature`: _string_ – Data signature encoded as hex

Variant constructors:

```typescript
function resultOfAppSigningBoxGetPublicKey(public_key: string): ResultOfAppSigningBox;
function resultOfAppSigningBoxSign(signature: string): ResultOfAppSigningBox;
```

### ResultOfSigningBoxGetPublicKey

```typescript
type ResultOfSigningBoxGetPublicKey = {
    pubkey: string
}
```

* `pubkey`: _string_ – Public key of signing box.

  
  Encoded with hex

### ParamsOfSigningBoxSign

```typescript
type ParamsOfSigningBoxSign = {
    signing_box: SigningBoxHandle,
    unsigned: string
}
```

* `signing_box`: [_SigningBoxHandle_](mod_crypto.md#SigningBoxHandle) – Signing Box handle.
* `unsigned`: _string_ – Unsigned user data.

  
  Must be encoded with `base64`.

### ResultOfSigningBoxSign

```typescript
type ResultOfSigningBoxSign = {
    signature: string
}
```

* `signature`: _string_ – Data signature.

  
  Encoded with `hex`.

### RegisteredEncryptionBox

```typescript
type RegisteredEncryptionBox = {
    handle: EncryptionBoxHandle
}
```

* `handle`: [_EncryptionBoxHandle_](mod_crypto.md#EncryptionBoxHandle) – Handle of the encryption box

### ParamsOfAppEncryptionBox

Encryption box callbacks.

```typescript
type ParamsOfAppEncryptionBox = {
    type: 'GetInfo'
} | {
    type: 'Encrypt'
    data: string
} | {
    type: 'Decrypt'
    data: string
}
```

Depends on value of the `type` field.

When _type_ is _'GetInfo'_

Get encryption box info

When _type_ is _'Encrypt'_

Encrypt data

* `data`: _string_ – Data, encoded in Base64

When _type_ is _'Decrypt'_

Decrypt data

* `data`: _string_ – Data, encoded in Base64

Variant constructors:

```typescript
function paramsOfAppEncryptionBoxGetInfo(): ParamsOfAppEncryptionBox;
function paramsOfAppEncryptionBoxEncrypt(data: string): ParamsOfAppEncryptionBox;
function paramsOfAppEncryptionBoxDecrypt(data: string): ParamsOfAppEncryptionBox;
```

### ResultOfAppEncryptionBox

Returning values from signing box callbacks.

```typescript
type ResultOfAppEncryptionBox = {
    type: 'GetInfo'
    info: EncryptionBoxInfo
} | {
    type: 'Encrypt'
    data: string
} | {
    type: 'Decrypt'
    data: string
}
```

Depends on value of the `type` field.

When _type_ is _'GetInfo'_

Result of getting encryption box info

* `info`: [_EncryptionBoxInfo_](mod_crypto.md#EncryptionBoxInfo)

When _type_ is _'Encrypt'_

Result of encrypting data

* `data`: _string_ – Encrypted data, encoded in Base64

When _type_ is _'Decrypt'_

Result of decrypting data

* `data`: _string_ – Decrypted data, encoded in Base64

Variant constructors:

```typescript
function resultOfAppEncryptionBoxGetInfo(info: EncryptionBoxInfo): ResultOfAppEncryptionBox;
function resultOfAppEncryptionBoxEncrypt(data: string): ResultOfAppEncryptionBox;
function resultOfAppEncryptionBoxDecrypt(data: string): ResultOfAppEncryptionBox;
```

### ParamsOfEncryptionBoxGetInfo

```typescript
type ParamsOfEncryptionBoxGetInfo = {
    encryption_box: EncryptionBoxHandle
}
```

* `encryption_box`: [_EncryptionBoxHandle_](mod_crypto.md#EncryptionBoxHandle) – Encryption box handle

### ResultOfEncryptionBoxGetInfo

```typescript
type ResultOfEncryptionBoxGetInfo = {
    info: EncryptionBoxInfo
}
```

* `info`: [_EncryptionBoxInfo_](mod_crypto.md#EncryptionBoxInfo) – Encryption box information

### ParamsOfEncryptionBoxEncrypt

```typescript
type ParamsOfEncryptionBoxEncrypt = {
    encryption_box: EncryptionBoxHandle,
    data: string
}
```

* `encryption_box`: [_EncryptionBoxHandle_](mod_crypto.md#EncryptionBoxHandle) – Encryption box handle
* `data`: _string_ – Data to be encrypted, encoded in Base64

### ResultOfEncryptionBoxEncrypt

```typescript
type ResultOfEncryptionBoxEncrypt = {
    data: string
}
```

* `data`: _string_ – Encrypted data, encoded in Base64.

  
  Padded to cipher block size

### ParamsOfEncryptionBoxDecrypt

```typescript
type ParamsOfEncryptionBoxDecrypt = {
    encryption_box: EncryptionBoxHandle,
    data: string
}
```

* `encryption_box`: [_EncryptionBoxHandle_](mod_crypto.md#EncryptionBoxHandle) – Encryption box handle
* `data`: _string_ – Data to be decrypted, encoded in Base64

### ResultOfEncryptionBoxDecrypt

```typescript
type ResultOfEncryptionBoxDecrypt = {
    data: string
}
```

* `data`: _string_ – Decrypted data, encoded in Base64.

### ParamsOfCreateEncryptionBox

```typescript
type ParamsOfCreateEncryptionBox = {
    algorithm: EncryptionAlgorithm
}
```

* `algorithm`: [_EncryptionAlgorithm_](mod_crypto.md#EncryptionAlgorithm) – Encryption algorithm specifier including cipher parameters \(key, IV, etc\)

### AppSigningBox

```typescript
type ResultOfAppSigningBoxGetPublicKey = {
    public_key: string
}

type ParamsOfAppSigningBoxSign = {
    unsigned: string
}

type ResultOfAppSigningBoxSign = {
    signature: string
}

export interface AppSigningBox {
    get_public_key(): Promise<ResultOfAppSigningBoxGetPublicKey>,
    sign(params: ParamsOfAppSigningBoxSign): Promise<ResultOfAppSigningBoxSign>,
}
```

### get\_public\_key

Get signing box public key

```typescript
type ResultOfAppSigningBoxGetPublicKey = {
    public_key: string
}

function get_public_key(): Promise<ResultOfAppSigningBoxGetPublicKey>;
```

#### Result

* `public_key`: _string_ – Signing box public key

### sign

Sign data

```typescript
type ParamsOfAppSigningBoxSign = {
    unsigned: string
}

type ResultOfAppSigningBoxSign = {
    signature: string
}

function sign(
    params: ParamsOfAppSigningBoxSign,
): Promise<ResultOfAppSigningBoxSign>;
```

#### Parameters

* `unsigned`: _string_ – Data to sign encoded as base64

#### Result

* `signature`: _string_ – Data signature encoded as hex

### AppEncryptionBox

```typescript
type ResultOfAppEncryptionBoxGetInfo = {
    info: EncryptionBoxInfo
}

type ParamsOfAppEncryptionBoxEncrypt = {
    data: string
}

type ResultOfAppEncryptionBoxEncrypt = {
    data: string
}

type ParamsOfAppEncryptionBoxDecrypt = {
    data: string
}

type ResultOfAppEncryptionBoxDecrypt = {
    data: string
}

export interface AppEncryptionBox {
    get_info(): Promise<ResultOfAppEncryptionBoxGetInfo>,
    encrypt(params: ParamsOfAppEncryptionBoxEncrypt): Promise<ResultOfAppEncryptionBoxEncrypt>,
    decrypt(params: ParamsOfAppEncryptionBoxDecrypt): Promise<ResultOfAppEncryptionBoxDecrypt>,
}
```

### get\_info

Get encryption box info

```typescript
type ResultOfAppEncryptionBoxGetInfo = {
    info: EncryptionBoxInfo
}

function get_info(): Promise<ResultOfAppEncryptionBoxGetInfo>;
```

#### Result

* `info`: [_EncryptionBoxInfo_](mod_crypto.md#EncryptionBoxInfo)

### encrypt

Encrypt data

```typescript
type ParamsOfAppEncryptionBoxEncrypt = {
    data: string
}

type ResultOfAppEncryptionBoxEncrypt = {
    data: string
}

function encrypt(
    params: ParamsOfAppEncryptionBoxEncrypt,
): Promise<ResultOfAppEncryptionBoxEncrypt>;
```

#### Parameters

* `data`: _string_ – Data, encoded in Base64

#### Result

* `data`: _string_ – Encrypted data, encoded in Base64

### decrypt

Decrypt data

```typescript
type ParamsOfAppEncryptionBoxDecrypt = {
    data: string
}

type ResultOfAppEncryptionBoxDecrypt = {
    data: string
}

function decrypt(
    params: ParamsOfAppEncryptionBoxDecrypt,
): Promise<ResultOfAppEncryptionBoxDecrypt>;
```

#### Parameters

* `data`: _string_ – Data, encoded in Base64

#### Result

* `data`: _string_ – Decrypted data, encoded in Base64

