# Module crypto

Crypto functions.


## Functions
[factorize](#factorize) – Integer factorization

[modular_power](#modular_power) – Modular exponentiation

[ton_crc16](#ton_crc16) – Calculates CRC16 using TON algorithm.

[generate_random_bytes](#generate_random_bytes) – Generates random byte array of the specified length and returns it in `base64` format

[convert_public_key_to_ton_safe_format](#convert_public_key_to_ton_safe_format) – Converts public key to ton safe_format

[generate_random_sign_keys](#generate_random_sign_keys) – Generates random ed25519 key pair.

[sign](#sign) – Signs a data using the provided keys.

[verify_signature](#verify_signature) – Verifies signed data using the provided public key. Raises error if verification is failed.

[sha256](#sha256) – Calculates SHA256 hash of the specified data.

[sha512](#sha512) – Calculates SHA512 hash of the specified data.

[scrypt](#scrypt) – Perform `scrypt` encryption

[nacl_sign_keypair_from_secret_key](#nacl_sign_keypair_from_secret_key) – Generates a key pair for signing from the secret key

[nacl_sign](#nacl_sign) – Signs data using the signer's secret key.

[nacl_sign_open](#nacl_sign_open) – Verifies the signature and returns the unsigned message

[nacl_sign_detached](#nacl_sign_detached) – Signs the message using the secret key and returns a signature.

[nacl_sign_detached_verify](#nacl_sign_detached_verify) – Verifies the signature with public key and `unsigned` data.

[nacl_box_keypair](#nacl_box_keypair) – Generates a random NaCl key pair

[nacl_box_keypair_from_secret_key](#nacl_box_keypair_from_secret_key) – Generates key pair from a secret key

[nacl_box](#nacl_box) – Public key authenticated encryption

[nacl_box_open](#nacl_box_open) – Decrypt and verify the cipher text using the receivers secret key, the senders public key, and the nonce.

[nacl_secret_box](#nacl_secret_box) – Encrypt and authenticate message using nonce and secret key.

[nacl_secret_box_open](#nacl_secret_box_open) – Decrypts and verifies cipher text using `nonce` and secret `key`.

[mnemonic_words](#mnemonic_words) – Prints the list of words from the specified dictionary

[mnemonic_from_random](#mnemonic_from_random) – Generates a random mnemonic

[mnemonic_from_entropy](#mnemonic_from_entropy) – Generates mnemonic from pre-generated entropy

[mnemonic_verify](#mnemonic_verify) – Validates a mnemonic phrase

[mnemonic_derive_sign_keys](#mnemonic_derive_sign_keys) – Derives a key pair for signing from the seed phrase

[hdkey_xprv_from_mnemonic](#hdkey_xprv_from_mnemonic) – Generates an extended master private key that will be the root for all the derived keys

[hdkey_derive_from_xprv](#hdkey_derive_from_xprv) – Returns extended private key derived from the specified extended private key and child index

[hdkey_derive_from_xprv_path](#hdkey_derive_from_xprv_path) – Derives the extended private key from the specified key and path

[hdkey_secret_from_xprv](#hdkey_secret_from_xprv) – Extracts the private key from the serialized extended private key

[hdkey_public_from_xprv](#hdkey_public_from_xprv) – Extracts the public key from the serialized extended private key

[chacha20](#chacha20) – Performs symmetric `chacha20` encryption.

[register_signing_box](#register_signing_box) – Register an application implemented signing box.

[get_signing_box](#get_signing_box) – Creates a default signing box implementation.

[signing_box_get_public_key](#signing_box_get_public_key) – Returns public key of signing key pair.

[signing_box_sign](#signing_box_sign) – Returns signed user data.

[remove_signing_box](#remove_signing_box) – Removes signing box from SDK.

[register_encryption_box](#register_encryption_box) – Register an application implemented encryption box.

[remove_encryption_box](#remove_encryption_box) – Removes encryption box from SDK

[encryption_box_get_info](#encryption_box_get_info) – Queries info from the given encryption box

[encryption_box_encrypt](#encryption_box_encrypt) – Encrypts data using given encryption box Note.

[encryption_box_decrypt](#encryption_box_decrypt) – Decrypts data using given encryption box Note.

[create_encryption_box](#create_encryption_box) – Creates encryption box with specified algorithm

## Types
[CryptoErrorCode](#cryptoerrorcode)

[SigningBoxHandle](#signingboxhandle)

[EncryptionBoxHandle](#encryptionboxhandle)

[EncryptionBoxInfo](#encryptionboxinfo) – Encryption box information

[EncryptionAlgorithm](#encryptionalgorithm)

[CipherMode](#ciphermode)

[AesParams](#aesparams)

[AesInfo](#aesinfo)

[ParamsOfFactorize](#paramsoffactorize)

[ResultOfFactorize](#resultoffactorize)

[ParamsOfModularPower](#paramsofmodularpower)

[ResultOfModularPower](#resultofmodularpower)

[ParamsOfTonCrc16](#paramsoftoncrc16)

[ResultOfTonCrc16](#resultoftoncrc16)

[ParamsOfGenerateRandomBytes](#paramsofgeneraterandombytes)

[ResultOfGenerateRandomBytes](#resultofgeneraterandombytes)

[ParamsOfConvertPublicKeyToTonSafeFormat](#paramsofconvertpublickeytotonsafeformat)

[ResultOfConvertPublicKeyToTonSafeFormat](#resultofconvertpublickeytotonsafeformat)

[KeyPair](#keypair)

[ParamsOfSign](#paramsofsign)

[ResultOfSign](#resultofsign)

[ParamsOfVerifySignature](#paramsofverifysignature)

[ResultOfVerifySignature](#resultofverifysignature)

[ParamsOfHash](#paramsofhash)

[ResultOfHash](#resultofhash)

[ParamsOfScrypt](#paramsofscrypt)

[ResultOfScrypt](#resultofscrypt)

[ParamsOfNaclSignKeyPairFromSecret](#paramsofnaclsignkeypairfromsecret)

[ParamsOfNaclSign](#paramsofnaclsign)

[ResultOfNaclSign](#resultofnaclsign)

[ParamsOfNaclSignOpen](#paramsofnaclsignopen)

[ResultOfNaclSignOpen](#resultofnaclsignopen)

[ResultOfNaclSignDetached](#resultofnaclsigndetached)

[ParamsOfNaclSignDetachedVerify](#paramsofnaclsigndetachedverify)

[ResultOfNaclSignDetachedVerify](#resultofnaclsigndetachedverify)

[ParamsOfNaclBoxKeyPairFromSecret](#paramsofnaclboxkeypairfromsecret)

[ParamsOfNaclBox](#paramsofnaclbox)

[ResultOfNaclBox](#resultofnaclbox)

[ParamsOfNaclBoxOpen](#paramsofnaclboxopen)

[ResultOfNaclBoxOpen](#resultofnaclboxopen)

[ParamsOfNaclSecretBox](#paramsofnaclsecretbox)

[ParamsOfNaclSecretBoxOpen](#paramsofnaclsecretboxopen)

[ParamsOfMnemonicWords](#paramsofmnemonicwords)

[ResultOfMnemonicWords](#resultofmnemonicwords)

[ParamsOfMnemonicFromRandom](#paramsofmnemonicfromrandom)

[ResultOfMnemonicFromRandom](#resultofmnemonicfromrandom)

[ParamsOfMnemonicFromEntropy](#paramsofmnemonicfromentropy)

[ResultOfMnemonicFromEntropy](#resultofmnemonicfromentropy)

[ParamsOfMnemonicVerify](#paramsofmnemonicverify)

[ResultOfMnemonicVerify](#resultofmnemonicverify)

[ParamsOfMnemonicDeriveSignKeys](#paramsofmnemonicderivesignkeys)

[ParamsOfHDKeyXPrvFromMnemonic](#paramsofhdkeyxprvfrommnemonic)

[ResultOfHDKeyXPrvFromMnemonic](#resultofhdkeyxprvfrommnemonic)

[ParamsOfHDKeyDeriveFromXPrv](#paramsofhdkeyderivefromxprv)

[ResultOfHDKeyDeriveFromXPrv](#resultofhdkeyderivefromxprv)

[ParamsOfHDKeyDeriveFromXPrvPath](#paramsofhdkeyderivefromxprvpath)

[ResultOfHDKeyDeriveFromXPrvPath](#resultofhdkeyderivefromxprvpath)

[ParamsOfHDKeySecretFromXPrv](#paramsofhdkeysecretfromxprv)

[ResultOfHDKeySecretFromXPrv](#resultofhdkeysecretfromxprv)

[ParamsOfHDKeyPublicFromXPrv](#paramsofhdkeypublicfromxprv)

[ResultOfHDKeyPublicFromXPrv](#resultofhdkeypublicfromxprv)

[ParamsOfChaCha20](#paramsofchacha20)

[ResultOfChaCha20](#resultofchacha20)

[RegisteredSigningBox](#registeredsigningbox)

[ParamsOfAppSigningBox](#paramsofappsigningbox) – Signing box callbacks.

[ResultOfAppSigningBox](#resultofappsigningbox) – Returning values from signing box callbacks.

[ResultOfSigningBoxGetPublicKey](#resultofsigningboxgetpublickey)

[ParamsOfSigningBoxSign](#paramsofsigningboxsign)

[ResultOfSigningBoxSign](#resultofsigningboxsign)

[RegisteredEncryptionBox](#registeredencryptionbox)

[ParamsOfAppEncryptionBox](#paramsofappencryptionbox) – Encryption box callbacks.

[ResultOfAppEncryptionBox](#resultofappencryptionbox) – Returning values from signing box callbacks.

[ParamsOfEncryptionBoxGetInfo](#paramsofencryptionboxgetinfo)

[ResultOfEncryptionBoxGetInfo](#resultofencryptionboxgetinfo)

[ParamsOfEncryptionBoxEncrypt](#paramsofencryptionboxencrypt)

[ResultOfEncryptionBoxEncrypt](#resultofencryptionboxencrypt)

[ParamsOfEncryptionBoxDecrypt](#paramsofencryptionboxdecrypt)

[ResultOfEncryptionBoxDecrypt](#resultofencryptionboxdecrypt)

[ParamsOfCreateEncryptionBox](#paramsofcreateencryptionbox)

[AppSigningBox](#appsigningbox)

[AppEncryptionBox](#appencryptionbox)


# Functions
## factorize

Integer factorization

Performs prime factorization – decomposition of a composite number
into a product of smaller prime integers (factors).
See [https://en.wikipedia.org/wiki/Integer_factorization]

```ts
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
### Parameters
- `composite`: _string_ – Hexadecimal representation of u64 composite number.


### Result

- `factors`: _string[]_ – Two factors of composite or empty if composite can't be factorized.


## modular_power

Modular exponentiation

Performs modular exponentiation for big integers (`base`^`exponent` mod `modulus`).
See [https://en.wikipedia.org/wiki/Modular_exponentiation]

```ts
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
}

type ResultOfTonCrc16 = {
    crc: number
}

function ton_crc16(
    params: ParamsOfTonCrc16,
): Promise<ResultOfTonCrc16>;
```
### Parameters
- `data`: _string_ – Input data for CRC calculation.
<br>Encoded with `base64`.


### Result

- `crc`: _number_ – Calculated CRC for input data.


## generate_random_bytes

Generates random byte array of the specified length and returns it in `base64` format

```ts
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
### Parameters
- `length`: _number_ – Size of random byte array.


### Result

- `bytes`: _string_ – Generated bytes encoded in `base64`.


## convert_public_key_to_ton_safe_format

Converts public key to ton safe_format

```ts
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
}

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
}

type ResultOfSign = {
    signed: string,
    signature: string
}

function sign(
    params: ParamsOfSign,
): Promise<ResultOfSign>;
```
### Parameters
- `unsigned`: _string_ – Data that must be signed encoded in `base64`.
- `keys`: _[KeyPair](mod_crypto.md#keypair)_ – Sign keys.


### Result

- `signed`: _string_ – Signed data combined with signature encoded in `base64`.
- `signature`: _string_ – Signature encoded in `hex`.


## verify_signature

Verifies signed data using the provided public key. Raises error if verification is failed.

```ts
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
}

type ResultOfHash = {
    hash: string
}

function sha256(
    params: ParamsOfHash,
): Promise<ResultOfHash>;
```
### Parameters
- `data`: _string_ – Input data for hash calculation.
<br>Encoded with `base64`.


### Result

- `hash`: _string_ – Hash of input `data`.
<br>Encoded with 'hex'.


## sha512

Calculates SHA512 hash of the specified data.

```ts
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
### Parameters
- `data`: _string_ – Input data for hash calculation.
<br>Encoded with `base64`.


### Result

- `hash`: _string_ – Hash of input `data`.
<br>Encoded with 'hex'.


## scrypt

Perform `scrypt` encryption

Derives key from `password` and `key` using `scrypt` algorithm.
See [https://en.wikipedia.org/wiki/Scrypt].

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
}

type ResultOfScrypt = {
    key: string
}

function scrypt(
    params: ParamsOfScrypt,
): Promise<ResultOfScrypt>;
```
### Parameters
- `password`: _string_ – The password bytes to be hashed. Must be encoded with `base64`.
- `salt`: _string_ – Salt bytes that modify the hash to protect against Rainbow table attacks. Must be encoded with `base64`.
- `log_n`: _number_ – CPU/memory cost parameter
- `r`: _number_ – The block size parameter, which fine-tunes sequential memory read size and performance.
- `p`: _number_ – Parallelization parameter.
- `dk_len`: _number_ – Intended output length in octets of the derived key.


### Result

- `key`: _string_ – Derived key.
<br>Encoded with `hex`.


## nacl_sign_keypair_from_secret_key

Generates a key pair for signing from the secret key

**NOTE:** In the result the secret key is actually the concatenation
of secret and public keys (128 symbols hex string) by design of [NaCL](http://nacl.cr.yp.to/sign.html).
See also [the stackexchange question](https://crypto.stackexchange.com/questions/54353/).

```ts
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
}

type ResultOfNaclSign = {
    signed: string
}

function nacl_sign(
    params: ParamsOfNaclSign,
): Promise<ResultOfNaclSign>;
```
### Parameters
- `unsigned`: _string_ – Data that must be signed encoded in `base64`.
- `secret`: _string_ – Signer's secret key - unprefixed 0-padded to 128 symbols hex string (concatenation of 64 symbols secret and 64 symbols public keys). See `nacl_sign_keypair_from_secret_key`.


### Result

- `signed`: _string_ – Signed data, encoded in `base64`.


## nacl_sign_open

Verifies the signature and returns the unsigned message

Verifies the signature in `signed` using the signer's public key `public`
and returns the message `unsigned`.

If the signature fails verification, crypto_sign_open raises an exception.

```ts
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
### Parameters
- `signed`: _string_ – Signed data that must be unsigned.
<br>Encoded with `base64`.
- `public`: _string_ – Signer's public key - unprefixed 0-padded to 64 symbols hex string


### Result

- `unsigned`: _string_ – Unsigned data, encoded in `base64`.


## nacl_sign_detached

Signs the message using the secret key and returns a signature.

Signs the message `unsigned` using the secret key `secret`
and returns a signature `signature`.

```ts
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
### Parameters
- `unsigned`: _string_ – Data that must be signed encoded in `base64`.
- `secret`: _string_ – Signer's secret key - unprefixed 0-padded to 128 symbols hex string (concatenation of 64 symbols secret and 64 symbols public keys). See `nacl_sign_keypair_from_secret_key`.


### Result

- `signature`: _string_ – Signature encoded in `hex`.


## nacl_sign_detached_verify

Verifies the signature with public key and `unsigned` data.

```ts
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
### Parameters
- `unsigned`: _string_ – Unsigned data that must be verified.
<br>Encoded with `base64`.
- `signature`: _string_ – Signature that must be verified.
<br>Encoded with `hex`.
- `public`: _string_ – Signer's public key - unprefixed 0-padded to 64 symbols hex string.


### Result

- `succeeded`: _boolean_ – `true` if verification succeeded or `false` if it failed


## nacl_box_keypair

Generates a random NaCl key pair

```ts
type KeyPair = {
    public: string,
    secret: string
}

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
}

type KeyPair = {
    public: string,
    secret: string
}

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

Encrypt and authenticate a message using the senders secret key, the receivers public
key, and a nonce.

```ts
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
### Parameters
- `decrypted`: _string_ – Data that must be encrypted encoded in `base64`.
- `nonce`: _string_ – Nonce, encoded in `hex`
- `their_public`: _string_ – Receiver's public key - unprefixed 0-padded to 64 symbols hex string
- `secret`: _string_ – Sender's private key - unprefixed 0-padded to 64 symbols hex string


### Result

- `encrypted`: _string_ – Encrypted data encoded in `base64`.


## nacl_box_open

Decrypt and verify the cipher text using the receivers secret key, the senders public key, and the nonce.

```ts
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
### Parameters
- `encrypted`: _string_ – Data that must be decrypted.
<br>Encoded with `base64`.
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
}

type ResultOfNaclBox = {
    encrypted: string
}

function nacl_secret_box(
    params: ParamsOfNaclSecretBox,
): Promise<ResultOfNaclBox>;
```
### Parameters
- `decrypted`: _string_ – Data that must be encrypted.
<br>Encoded with `base64`.
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
}

type ResultOfNaclBoxOpen = {
    decrypted: string
}

function nacl_secret_box_open(
    params: ParamsOfNaclSecretBoxOpen,
): Promise<ResultOfNaclBoxOpen>;
```
### Parameters
- `encrypted`: _string_ – Data that must be decrypted.
<br>Encoded with `base64`.
- `nonce`: _string_ – Nonce in `hex`
- `key`: _string_ – Public key - unprefixed 0-padded to 64 symbols hex string


### Result

- `decrypted`: _string_ – Decrypted data encoded in `base64`.


## mnemonic_words

Prints the list of words from the specified dictionary

```ts
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
### Parameters
- `dictionary`?: _number_ – Dictionary identifier


### Result

- `words`: _string_ – The list of mnemonic words


## mnemonic_from_random

Generates a random mnemonic

Generates a random mnemonic from the specified dictionary and word count

```ts
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
}

type ResultOfMnemonicFromEntropy = {
    phrase: string
}

function mnemonic_from_entropy(
    params: ParamsOfMnemonicFromEntropy,
): Promise<ResultOfMnemonicFromEntropy>;
```
### Parameters
- `entropy`: _string_ – Entropy bytes.
<br>Hex encoded.
- `dictionary`?: _number_ – Dictionary identifier
- `word_count`?: _number_ – Mnemonic word count


### Result

- `phrase`: _string_ – Phrase


## mnemonic_verify

Validates a mnemonic phrase

The phrase supplied will be checked for word length and validated according to the checksum
specified in BIP0039.

```ts
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
### Parameters
- `phrase`: _string_ – Phrase
- `dictionary`?: _number_ – Dictionary identifier
- `word_count`?: _number_ – Word count


### Result

- `valid`: _boolean_ – Flag indicating if the mnemonic is valid or not


## mnemonic_derive_sign_keys

Derives a key pair for signing from the seed phrase

Validates the seed phrase, generates master key and then derives
the key pair from the master key and the specified path

```ts
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
}

type ResultOfHDKeyXPrvFromMnemonic = {
    xprv: string
}

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
}

type ResultOfHDKeyDeriveFromXPrv = {
    xprv: string
}

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

Derives the extended private key from the specified key and path

```ts
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
}

type ResultOfHDKeySecretFromXPrv = {
    secret: string
}

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
}

type ResultOfHDKeyPublicFromXPrv = {
    public: string
}

function hdkey_public_from_xprv(
    params: ParamsOfHDKeyPublicFromXPrv,
): Promise<ResultOfHDKeyPublicFromXPrv>;
```
### Parameters
- `xprv`: _string_ – Serialized extended private key


### Result

- `public`: _string_ – Public key - 64 symbols hex string


## chacha20

Performs symmetric `chacha20` encryption.

```ts
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
### Parameters
- `data`: _string_ – Source data to be encrypted or decrypted.
<br>Must be encoded with `base64`.
- `key`: _string_ – 256-bit key.
<br>Must be encoded with `hex`.
- `nonce`: _string_ – 96-bit nonce.
<br>Must be encoded with `hex`.


### Result

- `data`: _string_ – Encrypted/decrypted data.
<br>Encoded with `base64`.


## register_signing_box

Register an application implemented signing box.

```ts
type RegisteredSigningBox = {
    handle: SigningBoxHandle
}

function register_signing_box(
    obj: AppSigningBox,
): Promise<RegisteredSigningBox>;
```


### Result

- `handle`: _[SigningBoxHandle](mod_crypto.md#signingboxhandle)_ – Handle of the signing box.


## get_signing_box

Creates a default signing box implementation.

```ts
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
### Parameters
- `public`: _string_ – Public key - 64 symbols hex string
- `secret`: _string_ – Private key - u64 symbols hex string


### Result

- `handle`: _[SigningBoxHandle](mod_crypto.md#signingboxhandle)_ – Handle of the signing box.


## signing_box_get_public_key

Returns public key of signing key pair.

```ts
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
### Parameters
- `handle`: _[SigningBoxHandle](mod_crypto.md#signingboxhandle)_ – Handle of the signing box.


### Result

- `pubkey`: _string_ – Public key of signing box.
<br>Encoded with hex


## signing_box_sign

Returns signed user data.

```ts
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
### Parameters
- `signing_box`: _[SigningBoxHandle](mod_crypto.md#signingboxhandle)_ – Signing Box handle.
- `unsigned`: _string_ – Unsigned user data.
<br>Must be encoded with `base64`.


### Result

- `signature`: _string_ – Data signature.
<br>Encoded with `hex`.


## remove_signing_box

Removes signing box from SDK.

```ts
type RegisteredSigningBox = {
    handle: SigningBoxHandle
}

function remove_signing_box(
    params: RegisteredSigningBox,
): Promise<void>;
```
### Parameters
- `handle`: _[SigningBoxHandle](mod_crypto.md#signingboxhandle)_ – Handle of the signing box.


## register_encryption_box

Register an application implemented encryption box.

```ts
type RegisteredEncryptionBox = {
    handle: EncryptionBoxHandle
}

function register_encryption_box(
    obj: AppEncryptionBox,
): Promise<RegisteredEncryptionBox>;
```


### Result

- `handle`: _[EncryptionBoxHandle](mod_crypto.md#encryptionboxhandle)_ – Handle of the encryption box


## remove_encryption_box

Removes encryption box from SDK

```ts
type RegisteredEncryptionBox = {
    handle: EncryptionBoxHandle
}

function remove_encryption_box(
    params: RegisteredEncryptionBox,
): Promise<void>;
```
### Parameters
- `handle`: _[EncryptionBoxHandle](mod_crypto.md#encryptionboxhandle)_ – Handle of the encryption box


## encryption_box_get_info

Queries info from the given encryption box

```ts
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
### Parameters
- `encryption_box`: _[EncryptionBoxHandle](mod_crypto.md#encryptionboxhandle)_ – Encryption box handle


### Result

- `info`: _[EncryptionBoxInfo](mod_crypto.md#encryptionboxinfo)_ – Encryption box information


## encryption_box_encrypt

Encrypts data using given encryption box Note.

Block cipher algorithms pad data to cipher block size so encrypted data can be longer then original data. Client should store the original data size after encryption and use it after
decryption to retrieve the original data from decrypted data.

```ts
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
### Parameters
- `encryption_box`: _[EncryptionBoxHandle](mod_crypto.md#encryptionboxhandle)_ – Encryption box handle
- `data`: _string_ – Data to be encrypted, encoded in Base64


### Result

- `data`: _string_ – Encrypted data, encoded in Base64.
<br>Padded to cipher block size


## encryption_box_decrypt

Decrypts data using given encryption box Note.

Block cipher algorithms pad data to cipher block size so encrypted data can be longer then original data. Client should store the original data size after encryption and use it after
decryption to retrieve the original data from decrypted data.

```ts
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
### Parameters
- `encryption_box`: _[EncryptionBoxHandle](mod_crypto.md#encryptionboxhandle)_ – Encryption box handle
- `data`: _string_ – Data to be decrypted, encoded in Base64


### Result

- `data`: _string_ – Decrypted data, encoded in Base64.


## create_encryption_box

Creates encryption box with specified algorithm

```ts
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
### Parameters
- `algorithm`: _[EncryptionAlgorithm](mod_crypto.md#encryptionalgorithm)_ – Encryption algorithm specifier including cipher parameters (key, IV, etc)


### Result

- `handle`: _[EncryptionBoxHandle](mod_crypto.md#encryptionboxhandle)_ – Handle of the encryption box


# Types
## CryptoErrorCode
```ts
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

- `InvalidPublicKey = 100`
- `InvalidSecretKey = 101`
- `InvalidKey = 102`
- `InvalidFactorizeChallenge = 106`
- `InvalidBigInt = 107`
- `ScryptFailed = 108`
- `InvalidKeySize = 109`
- `NaclSecretBoxFailed = 110`
- `NaclBoxFailed = 111`
- `NaclSignFailed = 112`
- `Bip39InvalidEntropy = 113`
- `Bip39InvalidPhrase = 114`
- `Bip32InvalidKey = 115`
- `Bip32InvalidDerivePath = 116`
- `Bip39InvalidDictionary = 117`
- `Bip39InvalidWordCount = 118`
- `MnemonicGenerationFailed = 119`
- `MnemonicFromEntropyFailed = 120`
- `SigningBoxNotRegistered = 121`
- `InvalidSignature = 122`
- `EncryptionBoxNotRegistered = 123`
- `InvalidIvSize = 124`
- `UnsupportedCipherMode = 125`
- `CannotCreateCipher = 126`
- `EncryptDataError = 127`
- `DecryptDataError = 128`
- `IvRequired = 129`


## SigningBoxHandle
```ts
type SigningBoxHandle = number
```


## EncryptionBoxHandle
```ts
type EncryptionBoxHandle = number
```


## EncryptionBoxInfo
Encryption box information

```ts
type EncryptionBoxInfo = {
    hdpath?: string,
    algorithm?: string,
    options?: any,
    public?: any
}
```
- `hdpath`?: _string_ – Derivation path, for instance "m/44'/396'/0'/0/0"
- `algorithm`?: _string_ – Cryptographic algorithm, used by this encryption box
- `options`?: _any_ – Options, depends on algorithm and specific encryption box implementation
- `public`?: _any_ – Public information, depends on algorithm


## EncryptionAlgorithm
```ts
type EncryptionAlgorithm = ({
    type: 'AES'
} & AesParams)
```
Depends on value of the  `type` field.

When _type_ is _'AES'_

- `mode`: _[CipherMode](mod_crypto.md#ciphermode)_
- `key`: _string_
- `iv`?: _string_


Variant constructors:

```ts
function encryptionAlgorithmAES(params: AesParams): EncryptionAlgorithm;
```

## CipherMode
```ts
enum CipherMode {
    CBC = "CBC",
    CFB = "CFB",
    CTR = "CTR",
    ECB = "ECB",
    OFB = "OFB"
}
```
One of the following value:

- `CBC = "CBC"`
- `CFB = "CFB"`
- `CTR = "CTR"`
- `ECB = "ECB"`
- `OFB = "OFB"`


## AesParams
```ts
type AesParams = {
    mode: CipherMode,
    key: string,
    iv?: string
}
```
- `mode`: _[CipherMode](mod_crypto.md#ciphermode)_
- `key`: _string_
- `iv`?: _string_


## AesInfo
```ts
type AesInfo = {
    mode: CipherMode,
    iv?: string
}
```
- `mode`: _[CipherMode](mod_crypto.md#ciphermode)_
- `iv`?: _string_


## ParamsOfFactorize
```ts
type ParamsOfFactorize = {
    composite: string
}
```
- `composite`: _string_ – Hexadecimal representation of u64 composite number.


## ResultOfFactorize
```ts
type ResultOfFactorize = {
    factors: string[]
}
```
- `factors`: _string[]_ – Two factors of composite or empty if composite can't be factorized.


## ParamsOfModularPower
```ts
type ParamsOfModularPower = {
    base: string,
    exponent: string,
    modulus: string
}
```
- `base`: _string_ – `base` argument of calculation.
- `exponent`: _string_ – `exponent` argument of calculation.
- `modulus`: _string_ – `modulus` argument of calculation.


## ResultOfModularPower
```ts
type ResultOfModularPower = {
    modular_power: string
}
```
- `modular_power`: _string_ – Result of modular exponentiation


## ParamsOfTonCrc16
```ts
type ParamsOfTonCrc16 = {
    data: string
}
```
- `data`: _string_ – Input data for CRC calculation.
<br>Encoded with `base64`.


## ResultOfTonCrc16
```ts
type ResultOfTonCrc16 = {
    crc: number
}
```
- `crc`: _number_ – Calculated CRC for input data.


## ParamsOfGenerateRandomBytes
```ts
type ParamsOfGenerateRandomBytes = {
    length: number
}
```
- `length`: _number_ – Size of random byte array.


## ResultOfGenerateRandomBytes
```ts
type ResultOfGenerateRandomBytes = {
    bytes: string
}
```
- `bytes`: _string_ – Generated bytes encoded in `base64`.


## ParamsOfConvertPublicKeyToTonSafeFormat
```ts
type ParamsOfConvertPublicKeyToTonSafeFormat = {
    public_key: string
}
```
- `public_key`: _string_ – Public key - 64 symbols hex string


## ResultOfConvertPublicKeyToTonSafeFormat
```ts
type ResultOfConvertPublicKeyToTonSafeFormat = {
    ton_public_key: string
}
```
- `ton_public_key`: _string_ – Public key represented in TON safe format.


## KeyPair
```ts
type KeyPair = {
    public: string,
    secret: string
}
```
- `public`: _string_ – Public key - 64 symbols hex string
- `secret`: _string_ – Private key - u64 symbols hex string


## ParamsOfSign
```ts
type ParamsOfSign = {
    unsigned: string,
    keys: KeyPair
}
```
- `unsigned`: _string_ – Data that must be signed encoded in `base64`.
- `keys`: _[KeyPair](mod_crypto.md#keypair)_ – Sign keys.


## ResultOfSign
```ts
type ResultOfSign = {
    signed: string,
    signature: string
}
```
- `signed`: _string_ – Signed data combined with signature encoded in `base64`.
- `signature`: _string_ – Signature encoded in `hex`.


## ParamsOfVerifySignature
```ts
type ParamsOfVerifySignature = {
    signed: string,
    public: string
}
```
- `signed`: _string_ – Signed data that must be verified encoded in `base64`.
- `public`: _string_ – Signer's public key - 64 symbols hex string


## ResultOfVerifySignature
```ts
type ResultOfVerifySignature = {
    unsigned: string
}
```
- `unsigned`: _string_ – Unsigned data encoded in `base64`.


## ParamsOfHash
```ts
type ParamsOfHash = {
    data: string
}
```
- `data`: _string_ – Input data for hash calculation.
<br>Encoded with `base64`.


## ResultOfHash
```ts
type ResultOfHash = {
    hash: string
}
```
- `hash`: _string_ – Hash of input `data`.
<br>Encoded with 'hex'.


## ParamsOfScrypt
```ts
type ParamsOfScrypt = {
    password: string,
    salt: string,
    log_n: number,
    r: number,
    p: number,
    dk_len: number
}
```
- `password`: _string_ – The password bytes to be hashed. Must be encoded with `base64`.
- `salt`: _string_ – Salt bytes that modify the hash to protect against Rainbow table attacks. Must be encoded with `base64`.
- `log_n`: _number_ – CPU/memory cost parameter
- `r`: _number_ – The block size parameter, which fine-tunes sequential memory read size and performance.
- `p`: _number_ – Parallelization parameter.
- `dk_len`: _number_ – Intended output length in octets of the derived key.


## ResultOfScrypt
```ts
type ResultOfScrypt = {
    key: string
}
```
- `key`: _string_ – Derived key.
<br>Encoded with `hex`.


## ParamsOfNaclSignKeyPairFromSecret
```ts
type ParamsOfNaclSignKeyPairFromSecret = {
    secret: string
}
```
- `secret`: _string_ – Secret key - unprefixed 0-padded to 64 symbols hex string


## ParamsOfNaclSign
```ts
type ParamsOfNaclSign = {
    unsigned: string,
    secret: string
}
```
- `unsigned`: _string_ – Data that must be signed encoded in `base64`.
- `secret`: _string_ – Signer's secret key - unprefixed 0-padded to 128 symbols hex string (concatenation of 64 symbols secret and 64 symbols public keys). See `nacl_sign_keypair_from_secret_key`.


## ResultOfNaclSign
```ts
type ResultOfNaclSign = {
    signed: string
}
```
- `signed`: _string_ – Signed data, encoded in `base64`.


## ParamsOfNaclSignOpen
```ts
type ParamsOfNaclSignOpen = {
    signed: string,
    public: string
}
```
- `signed`: _string_ – Signed data that must be unsigned.
<br>Encoded with `base64`.
- `public`: _string_ – Signer's public key - unprefixed 0-padded to 64 symbols hex string


## ResultOfNaclSignOpen
```ts
type ResultOfNaclSignOpen = {
    unsigned: string
}
```
- `unsigned`: _string_ – Unsigned data, encoded in `base64`.


## ResultOfNaclSignDetached
```ts
type ResultOfNaclSignDetached = {
    signature: string
}
```
- `signature`: _string_ – Signature encoded in `hex`.


## ParamsOfNaclSignDetachedVerify
```ts
type ParamsOfNaclSignDetachedVerify = {
    unsigned: string,
    signature: string,
    public: string
}
```
- `unsigned`: _string_ – Unsigned data that must be verified.
<br>Encoded with `base64`.
- `signature`: _string_ – Signature that must be verified.
<br>Encoded with `hex`.
- `public`: _string_ – Signer's public key - unprefixed 0-padded to 64 symbols hex string.


## ResultOfNaclSignDetachedVerify
```ts
type ResultOfNaclSignDetachedVerify = {
    succeeded: boolean
}
```
- `succeeded`: _boolean_ – `true` if verification succeeded or `false` if it failed


## ParamsOfNaclBoxKeyPairFromSecret
```ts
type ParamsOfNaclBoxKeyPairFromSecret = {
    secret: string
}
```
- `secret`: _string_ – Secret key - unprefixed 0-padded to 64 symbols hex string


## ParamsOfNaclBox
```ts
type ParamsOfNaclBox = {
    decrypted: string,
    nonce: string,
    their_public: string,
    secret: string
}
```
- `decrypted`: _string_ – Data that must be encrypted encoded in `base64`.
- `nonce`: _string_ – Nonce, encoded in `hex`
- `their_public`: _string_ – Receiver's public key - unprefixed 0-padded to 64 symbols hex string
- `secret`: _string_ – Sender's private key - unprefixed 0-padded to 64 symbols hex string


## ResultOfNaclBox
```ts
type ResultOfNaclBox = {
    encrypted: string
}
```
- `encrypted`: _string_ – Encrypted data encoded in `base64`.


## ParamsOfNaclBoxOpen
```ts
type ParamsOfNaclBoxOpen = {
    encrypted: string,
    nonce: string,
    their_public: string,
    secret: string
}
```
- `encrypted`: _string_ – Data that must be decrypted.
<br>Encoded with `base64`.
- `nonce`: _string_
- `their_public`: _string_ – Sender's public key - unprefixed 0-padded to 64 symbols hex string
- `secret`: _string_ – Receiver's private key - unprefixed 0-padded to 64 symbols hex string


## ResultOfNaclBoxOpen
```ts
type ResultOfNaclBoxOpen = {
    decrypted: string
}
```
- `decrypted`: _string_ – Decrypted data encoded in `base64`.


## ParamsOfNaclSecretBox
```ts
type ParamsOfNaclSecretBox = {
    decrypted: string,
    nonce: string,
    key: string
}
```
- `decrypted`: _string_ – Data that must be encrypted.
<br>Encoded with `base64`.
- `nonce`: _string_ – Nonce in `hex`
- `key`: _string_ – Secret key - unprefixed 0-padded to 64 symbols hex string


## ParamsOfNaclSecretBoxOpen
```ts
type ParamsOfNaclSecretBoxOpen = {
    encrypted: string,
    nonce: string,
    key: string
}
```
- `encrypted`: _string_ – Data that must be decrypted.
<br>Encoded with `base64`.
- `nonce`: _string_ – Nonce in `hex`
- `key`: _string_ – Public key - unprefixed 0-padded to 64 symbols hex string


## ParamsOfMnemonicWords
```ts
type ParamsOfMnemonicWords = {
    dictionary?: number
}
```
- `dictionary`?: _number_ – Dictionary identifier


## ResultOfMnemonicWords
```ts
type ResultOfMnemonicWords = {
    words: string
}
```
- `words`: _string_ – The list of mnemonic words


## ParamsOfMnemonicFromRandom
```ts
type ParamsOfMnemonicFromRandom = {
    dictionary?: number,
    word_count?: number
}
```
- `dictionary`?: _number_ – Dictionary identifier
- `word_count`?: _number_ – Mnemonic word count


## ResultOfMnemonicFromRandom
```ts
type ResultOfMnemonicFromRandom = {
    phrase: string
}
```
- `phrase`: _string_ – String of mnemonic words


## ParamsOfMnemonicFromEntropy
```ts
type ParamsOfMnemonicFromEntropy = {
    entropy: string,
    dictionary?: number,
    word_count?: number
}
```
- `entropy`: _string_ – Entropy bytes.
<br>Hex encoded.
- `dictionary`?: _number_ – Dictionary identifier
- `word_count`?: _number_ – Mnemonic word count


## ResultOfMnemonicFromEntropy
```ts
type ResultOfMnemonicFromEntropy = {
    phrase: string
}
```
- `phrase`: _string_ – Phrase


## ParamsOfMnemonicVerify
```ts
type ParamsOfMnemonicVerify = {
    phrase: string,
    dictionary?: number,
    word_count?: number
}
```
- `phrase`: _string_ – Phrase
- `dictionary`?: _number_ – Dictionary identifier
- `word_count`?: _number_ – Word count


## ResultOfMnemonicVerify
```ts
type ResultOfMnemonicVerify = {
    valid: boolean
}
```
- `valid`: _boolean_ – Flag indicating if the mnemonic is valid or not


## ParamsOfMnemonicDeriveSignKeys
```ts
type ParamsOfMnemonicDeriveSignKeys = {
    phrase: string,
    path?: string,
    dictionary?: number,
    word_count?: number
}
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
}
```
- `phrase`: _string_ – String with seed phrase
- `dictionary`?: _number_ – Dictionary identifier
- `word_count`?: _number_ – Mnemonic word count


## ResultOfHDKeyXPrvFromMnemonic
```ts
type ResultOfHDKeyXPrvFromMnemonic = {
    xprv: string
}
```
- `xprv`: _string_ – Serialized extended master private key


## ParamsOfHDKeyDeriveFromXPrv
```ts
type ParamsOfHDKeyDeriveFromXPrv = {
    xprv: string,
    child_index: number,
    hardened: boolean
}
```
- `xprv`: _string_ – Serialized extended private key
- `child_index`: _number_ – Child index (see BIP-0032)
- `hardened`: _boolean_ – Indicates the derivation of hardened/not-hardened key (see BIP-0032)


## ResultOfHDKeyDeriveFromXPrv
```ts
type ResultOfHDKeyDeriveFromXPrv = {
    xprv: string
}
```
- `xprv`: _string_ – Serialized extended private key


## ParamsOfHDKeyDeriveFromXPrvPath
```ts
type ParamsOfHDKeyDeriveFromXPrvPath = {
    xprv: string,
    path: string
}
```
- `xprv`: _string_ – Serialized extended private key
- `path`: _string_ – Derivation path, for instance "m/44'/396'/0'/0/0"


## ResultOfHDKeyDeriveFromXPrvPath
```ts
type ResultOfHDKeyDeriveFromXPrvPath = {
    xprv: string
}
```
- `xprv`: _string_ – Derived serialized extended private key


## ParamsOfHDKeySecretFromXPrv
```ts
type ParamsOfHDKeySecretFromXPrv = {
    xprv: string
}
```
- `xprv`: _string_ – Serialized extended private key


## ResultOfHDKeySecretFromXPrv
```ts
type ResultOfHDKeySecretFromXPrv = {
    secret: string
}
```
- `secret`: _string_ – Private key - 64 symbols hex string


## ParamsOfHDKeyPublicFromXPrv
```ts
type ParamsOfHDKeyPublicFromXPrv = {
    xprv: string
}
```
- `xprv`: _string_ – Serialized extended private key


## ResultOfHDKeyPublicFromXPrv
```ts
type ResultOfHDKeyPublicFromXPrv = {
    public: string
}
```
- `public`: _string_ – Public key - 64 symbols hex string


## ParamsOfChaCha20
```ts
type ParamsOfChaCha20 = {
    data: string,
    key: string,
    nonce: string
}
```
- `data`: _string_ – Source data to be encrypted or decrypted.
<br>Must be encoded with `base64`.
- `key`: _string_ – 256-bit key.
<br>Must be encoded with `hex`.
- `nonce`: _string_ – 96-bit nonce.
<br>Must be encoded with `hex`.


## ResultOfChaCha20
```ts
type ResultOfChaCha20 = {
    data: string
}
```
- `data`: _string_ – Encrypted/decrypted data.
<br>Encoded with `base64`.


## RegisteredSigningBox
```ts
type RegisteredSigningBox = {
    handle: SigningBoxHandle
}
```
- `handle`: _[SigningBoxHandle](mod_crypto.md#signingboxhandle)_ – Handle of the signing box.


## ParamsOfAppSigningBox
Signing box callbacks.

```ts
type ParamsOfAppSigningBox = {
    type: 'GetPublicKey'
} | {
    type: 'Sign'
    unsigned: string
}
```
Depends on value of the  `type` field.

When _type_ is _'GetPublicKey'_

Get signing box public key


When _type_ is _'Sign'_

Sign data


- `unsigned`: _string_ – Data to sign encoded as base64


Variant constructors:

```ts
function paramsOfAppSigningBoxGetPublicKey(): ParamsOfAppSigningBox;
function paramsOfAppSigningBoxSign(unsigned: string): ParamsOfAppSigningBox;
```

## ResultOfAppSigningBox
Returning values from signing box callbacks.

```ts
type ResultOfAppSigningBox = {
    type: 'GetPublicKey'
    public_key: string
} | {
    type: 'Sign'
    signature: string
}
```
Depends on value of the  `type` field.

When _type_ is _'GetPublicKey'_

Result of getting public key


- `public_key`: _string_ – Signing box public key

When _type_ is _'Sign'_

Result of signing data


- `signature`: _string_ – Data signature encoded as hex


Variant constructors:

```ts
function resultOfAppSigningBoxGetPublicKey(public_key: string): ResultOfAppSigningBox;
function resultOfAppSigningBoxSign(signature: string): ResultOfAppSigningBox;
```

## ResultOfSigningBoxGetPublicKey
```ts
type ResultOfSigningBoxGetPublicKey = {
    pubkey: string
}
```
- `pubkey`: _string_ – Public key of signing box.
<br>Encoded with hex


## ParamsOfSigningBoxSign
```ts
type ParamsOfSigningBoxSign = {
    signing_box: SigningBoxHandle,
    unsigned: string
}
```
- `signing_box`: _[SigningBoxHandle](mod_crypto.md#signingboxhandle)_ – Signing Box handle.
- `unsigned`: _string_ – Unsigned user data.
<br>Must be encoded with `base64`.


## ResultOfSigningBoxSign
```ts
type ResultOfSigningBoxSign = {
    signature: string
}
```
- `signature`: _string_ – Data signature.
<br>Encoded with `hex`.


## RegisteredEncryptionBox
```ts
type RegisteredEncryptionBox = {
    handle: EncryptionBoxHandle
}
```
- `handle`: _[EncryptionBoxHandle](mod_crypto.md#encryptionboxhandle)_ – Handle of the encryption box


## ParamsOfAppEncryptionBox
Encryption box callbacks.

```ts
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
Depends on value of the  `type` field.

When _type_ is _'GetInfo'_

Get encryption box info


When _type_ is _'Encrypt'_

Encrypt data


- `data`: _string_ – Data, encoded in Base64

When _type_ is _'Decrypt'_

Decrypt data


- `data`: _string_ – Data, encoded in Base64


Variant constructors:

```ts
function paramsOfAppEncryptionBoxGetInfo(): ParamsOfAppEncryptionBox;
function paramsOfAppEncryptionBoxEncrypt(data: string): ParamsOfAppEncryptionBox;
function paramsOfAppEncryptionBoxDecrypt(data: string): ParamsOfAppEncryptionBox;
```

## ResultOfAppEncryptionBox
Returning values from signing box callbacks.

```ts
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
Depends on value of the  `type` field.

When _type_ is _'GetInfo'_

Result of getting encryption box info


- `info`: _[EncryptionBoxInfo](mod_crypto.md#encryptionboxinfo)_

When _type_ is _'Encrypt'_

Result of encrypting data


- `data`: _string_ – Encrypted data, encoded in Base64

When _type_ is _'Decrypt'_

Result of decrypting data


- `data`: _string_ – Decrypted data, encoded in Base64


Variant constructors:

```ts
function resultOfAppEncryptionBoxGetInfo(info: EncryptionBoxInfo): ResultOfAppEncryptionBox;
function resultOfAppEncryptionBoxEncrypt(data: string): ResultOfAppEncryptionBox;
function resultOfAppEncryptionBoxDecrypt(data: string): ResultOfAppEncryptionBox;
```

## ParamsOfEncryptionBoxGetInfo
```ts
type ParamsOfEncryptionBoxGetInfo = {
    encryption_box: EncryptionBoxHandle
}
```
- `encryption_box`: _[EncryptionBoxHandle](mod_crypto.md#encryptionboxhandle)_ – Encryption box handle


## ResultOfEncryptionBoxGetInfo
```ts
type ResultOfEncryptionBoxGetInfo = {
    info: EncryptionBoxInfo
}
```
- `info`: _[EncryptionBoxInfo](mod_crypto.md#encryptionboxinfo)_ – Encryption box information


## ParamsOfEncryptionBoxEncrypt
```ts
type ParamsOfEncryptionBoxEncrypt = {
    encryption_box: EncryptionBoxHandle,
    data: string
}
```
- `encryption_box`: _[EncryptionBoxHandle](mod_crypto.md#encryptionboxhandle)_ – Encryption box handle
- `data`: _string_ – Data to be encrypted, encoded in Base64


## ResultOfEncryptionBoxEncrypt
```ts
type ResultOfEncryptionBoxEncrypt = {
    data: string
}
```
- `data`: _string_ – Encrypted data, encoded in Base64.
<br>Padded to cipher block size


## ParamsOfEncryptionBoxDecrypt
```ts
type ParamsOfEncryptionBoxDecrypt = {
    encryption_box: EncryptionBoxHandle,
    data: string
}
```
- `encryption_box`: _[EncryptionBoxHandle](mod_crypto.md#encryptionboxhandle)_ – Encryption box handle
- `data`: _string_ – Data to be decrypted, encoded in Base64


## ResultOfEncryptionBoxDecrypt
```ts
type ResultOfEncryptionBoxDecrypt = {
    data: string
}
```
- `data`: _string_ – Decrypted data, encoded in Base64.


## ParamsOfCreateEncryptionBox
```ts
type ParamsOfCreateEncryptionBox = {
    algorithm: EncryptionAlgorithm
}
```
- `algorithm`: _[EncryptionAlgorithm](mod_crypto.md#encryptionalgorithm)_ – Encryption algorithm specifier including cipher parameters (key, IV, etc)


## AppSigningBox

```ts

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

## get_public_key

Get signing box public key

```ts
type ResultOfAppSigningBoxGetPublicKey = {
    public_key: string
}

function get_public_key(): Promise<ResultOfAppSigningBoxGetPublicKey>;
```


### Result

- `public_key`: _string_ – Signing box public key


## sign

Sign data

```ts
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
### Parameters
- `unsigned`: _string_ – Data to sign encoded as base64


### Result

- `signature`: _string_ – Data signature encoded as hex


## AppEncryptionBox

```ts

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

## get_info

Get encryption box info

```ts
type ResultOfAppEncryptionBoxGetInfo = {
    info: EncryptionBoxInfo
}

function get_info(): Promise<ResultOfAppEncryptionBoxGetInfo>;
```


### Result

- `info`: _[EncryptionBoxInfo](mod_crypto.md#encryptionboxinfo)_


## encrypt

Encrypt data

```ts
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
### Parameters
- `data`: _string_ – Data, encoded in Base64


### Result

- `data`: _string_ – Encrypted data, encoded in Base64


## decrypt

Decrypt data

```ts
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
### Parameters
- `data`: _string_ – Data, encoded in Base64


### Result

- `data`: _string_ – Decrypted data, encoded in Base64


