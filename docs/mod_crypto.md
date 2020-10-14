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

[hdkey_derive_from_xprv_path](#hdkey_derive_from_xprv_path) – Derives the exented private key from the specified key and path

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

function factorize(
    params: ParamsOfFactorize,
    responseHandler: ResponseHandler | null,
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

function modularPower(
    params: ParamsOfModularPower,
    responseHandler: ResponseHandler | null,
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

function tonCrc16(
    params: ParamsOfTonCrc16,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfTonCrc16>;

```
### Parameters
- `data`: _string_ –  Input data for CRC calculation. Encoded with `base64`.
### Result

- `crc`: _number_ –  Calculated CRC for input data.


## generate_random_bytes

```ts

function generateRandomBytes(
    params: ParamsOfGenerateRandomBytes,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfGenerateRandomBytes>;

```
### Parameters
- `length`: _number_ –  Size of random byte array.
### Result

- `bytes`: _string_ –  Generated bytes, encoded with `base64`.


## convert_public_key_to_ton_safe_format

 Converts public key to ton safe_format

```ts

function convertPublicKeyToTonSafeFormat(
    params: ParamsOfConvertPublicKeyToTonSafeFormat,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfConvertPublicKeyToTonSafeFormat>;

```
### Parameters
- `public_key`: _string_ –  Public key.
### Result

- `ton_public_key`: _string_ –  Public key represented in TON safe format.


## generate_random_sign_keys

 Generates random ed25519 key pair.

```ts

function generateRandomSignKeys(
    responseHandler: ResponseHandler | null,
): Promise<KeyPair>;

```
### Result

- `public`: _string_ –  Public key. Encoded with `hex`.
- `secret`: _string_ –  Private key. Encoded with `hex`.


## sign

 Signs a data using the provided keys.

```ts

function sign(
    params: ParamsOfSign,
    responseHandler: ResponseHandler | null,
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

function verifySignature(
    params: ParamsOfVerifySignature,
    responseHandler: ResponseHandler | null,
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

function sha256(
    params: ParamsOfHash,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfHash>;

```
### Parameters
- `data`: _string_ –  Input data for hash calculation. Encoded with `base64`.
### Result

- `hash`: _string_ –  Hex-encoded hash of input `data`.


## sha512

Calculates SHA512 hash of the specified data.

```ts

function sha512(
    params: ParamsOfHash,
    responseHandler: ResponseHandler | null,
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

function scrypt(
    params: ParamsOfScrypt,
    responseHandler: ResponseHandler | null,
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

function naclSignKeypairFromSecretKey(
    params: ParamsOfNaclSignKeyPairFromSecret,
    responseHandler: ResponseHandler | null,
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

function naclSign(
    params: ParamsOfNaclSign,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfNaclSign>;

```
### Parameters
- `unsigned`: _string_ –  Data that must be signed. Encoded with `base64`.
- `secret`: _string_ –  Signer's secret key.
### Result

- `signed`: _string_ –  Signed data, encoded with `base64`.


## nacl_sign_open

```ts

function naclSignOpen(
    params: ParamsOfNaclSignOpen,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfNaclSignOpen>;

```
### Parameters
- `signed`: _string_ –  Signed data that must be unsigned. Encoded with `base64`.
- `public`: _string_ –  Signer's public key.
### Result

- `unsigned`: _string_ –  Unsigned data, encoded with `base64`.


## nacl_sign_detached

```ts

function naclSignDetached(
    params: ParamsOfNaclSign,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfNaclSignDetached>;

```
### Parameters
- `unsigned`: _string_ –  Data that must be signed. Encoded with `base64`.
- `secret`: _string_ –  Signer's secret key.
### Result

- `signature`: _string_ –  Hex encoded sign.


## nacl_box_keypair

```ts

function naclBoxKeypair(
    responseHandler: ResponseHandler | null,
): Promise<KeyPair>;

```
### Result

- `public`: _string_ –  Public key. Encoded with `hex`.
- `secret`: _string_ –  Private key. Encoded with `hex`.


## nacl_box_keypair_from_secret_key

```ts

function naclBoxKeypairFromSecretKey(
    params: ParamsOfNaclBoxKeyPairFromSecret,
    responseHandler: ResponseHandler | null,
): Promise<KeyPair>;

```
### Parameters
- `secret`: _string_ –  Hex encoded secret key.
### Result

- `public`: _string_ –  Public key. Encoded with `hex`.
- `secret`: _string_ –  Private key. Encoded with `hex`.


## nacl_box

```ts

function naclBox(
    params: ParamsOfNaclBox,
    responseHandler: ResponseHandler | null,
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

function naclBoxOpen(
    params: ParamsOfNaclBoxOpen,
    responseHandler: ResponseHandler | null,
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

function naclSecretBox(
    params: ParamsOfNaclSecretBox,
    responseHandler: ResponseHandler | null,
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

function naclSecretBoxOpen(
    params: ParamsOfNaclSecretBoxOpen,
    responseHandler: ResponseHandler | null,
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

function mnemonicWords(
    params: ParamsOfMnemonicWords,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfMnemonicWords>;

```
### Parameters
- `dictionary`?: _number_ –  dictionary identifier
### Result

- `words`: _string_ –  the list of mnemonic words


## mnemonic_from_random

 Generates a random mnemnonic from the specified dictionary and word count

```ts

function mnemonicFromRandom(
    params: ParamsOfMnemonicFromRandom,
    responseHandler: ResponseHandler | null,
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

function mnemonicFromEntropy(
    params: ParamsOfMnemonicFromEntropy,
    responseHandler: ResponseHandler | null,
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

function mnemonicVerify(
    params: ParamsOfMnemonicVerify,
    responseHandler: ResponseHandler | null,
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

function mnemonicDeriveSignKeys(
    params: ParamsOfMnemonicDeriveSignKeys,
    responseHandler: ResponseHandler | null,
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

function hdkeyXprvFromMnemonic(
    params: ParamsOfHDKeyXPrvFromMnemonic,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfHDKeyXPrvFromMnemonic>;

```
### Parameters
- `phrase`: _string_ – string with seed phrase
### Result

- `xprv`: _string_ –  serialized extended master private key


## hdkey_derive_from_xprv

 Returns derived extended private key derived from the specified extended private key and child index

```ts

function hdkeyDeriveFromXprv(
    params: ParamsOfHDKeyDeriveFromXPrv,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfHDKeyDeriveFromXPrv>;

```
### Parameters
- `xprv`: _string_ –  serialized extended private key
- `child_index`: _number_ –  child index (see BIP-0032)
- `hardened`: _boolean_ –  indicates the derivation of hardened/not-hardened key (see BIP-0032)
### Result

- `xprv`: _string_ –  serialized extended private key


## hdkey_derive_from_xprv_path

 Derives the exented private key from the specified key and path

```ts

function hdkeyDeriveFromXprvPath(
    params: ParamsOfHDKeyDeriveFromXPrvPath,
    responseHandler: ResponseHandler | null,
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

function hdkeySecretFromXprv(
    params: ParamsOfHDKeySecretFromXPrv,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfHDKeySecretFromXPrv>;

```
### Parameters
- `xprv`: _string_ –  serialized extended private key
### Result

- `secret`: _string_ –  private key


## hdkey_public_from_xprv

 Extracts the public key from the serialized extended private key

```ts

function hdkeyPublicFromXprv(
    params: ParamsOfHDKeyPublicFromXPrv,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfHDKeyPublicFromXPrv>;

```
### Parameters
- `xprv`: _string_ –  serialized extended private key
### Result

- `public`: _string_ –  public key


# Types
## SigningBoxHandle

- ``: _number_


## ParamsOfFactorize

- `composite`: _string_ –  Hexadecimal representation of u64 composite number.


## ResultOfFactorize

- `factors`: _string[]_ –  Two factors of composite or empty if composite can't be factorized.


## ParamsOfModularPower

- `base`: _string_ –  `base` argument of calculation.
- `exponent`: _string_ –  `exponent` argument of calculation.
- `modulus`: _string_ –  `modulus` argument of calculation.


## ResultOfModularPower

- `modular_power`: _string_ –  result of modular exponentiation


## ParamsOfTonCrc16

- `data`: _string_ –  Input data for CRC calculation. Encoded with `base64`.


## ResultOfTonCrc16

- `crc`: _number_ –  Calculated CRC for input data.


## ParamsOfGenerateRandomBytes

- `length`: _number_ –  Size of random byte array.


## ResultOfGenerateRandomBytes

- `bytes`: _string_ –  Generated bytes, encoded with `base64`.


## ParamsOfConvertPublicKeyToTonSafeFormat

- `public_key`: _string_ –  Public key.


## ResultOfConvertPublicKeyToTonSafeFormat

- `ton_public_key`: _string_ –  Public key represented in TON safe format.


## KeyPair

- `public`: _string_ –  Public key. Encoded with `hex`.
- `secret`: _string_ –  Private key. Encoded with `hex`.


## ParamsOfSign

- `unsigned`: _string_ –  Data that must be signed.
- `keys`: _[KeyPair](mod_crypto.md#KeyPair)_ –  Sign keys.


## ResultOfSign

- `signed`: _string_ –  Signed data combined with signature. Encoded with `base64`.
- `signature`: _string_ –  Signature. Encoded with `base64`.


## ParamsOfVerifySignature

- `signed`: _string_ –  Signed data that must be verified.
- `public`: _string_ –  Signer's public key.


## ResultOfVerifySignature

- `unsigned`: _string_ –  Unsigned data.


## ParamsOfHash

- `data`: _string_ –  Input data for hash calculation. Encoded with `base64`.


## ResultOfHash

- `hash`: _string_ –  Hex-encoded hash of input `data`.


## ParamsOfScrypt

- `password`: _string_ –  The password bytes to be hashed.
- `salt`: _string_ –  A salt bytes that modifies the hash to protect against Rainbow table attacks.
- `log_n`: _number_ –  CPU/memory cost parameter
- `r`: _number_ –  The block size parameter, which fine-tunes sequential memory read size and performance.
- `p`: _number_ –  Parallelization parameter.
- `dk_len`: _number_ –  Intended output length in octets of the derived key.


## ResultOfScrypt

- `key`: _string_ –  Derived key. Encoded with `hex`.


## ParamsOfNaclSignKeyPairFromSecret

- `secret`: _string_ –  secret key


## ParamsOfNaclSign

- `unsigned`: _string_ –  Data that must be signed. Encoded with `base64`.
- `secret`: _string_ –  Signer's secret key.


## ResultOfNaclSign

- `signed`: _string_ –  Signed data, encoded with `base64`.


## ParamsOfNaclSignOpen

- `signed`: _string_ –  Signed data that must be unsigned. Encoded with `base64`.
- `public`: _string_ –  Signer's public key.


## ResultOfNaclSignOpen

- `unsigned`: _string_ –  Unsigned data, encoded with `base64`.


## ResultOfNaclSignDetached

- `signature`: _string_ –  Hex encoded sign.


## ParamsOfNaclBoxKeyPairFromSecret

- `secret`: _string_ –  Hex encoded secret key.


## ParamsOfNaclBox

- `decrypted`: _string_ –  Data that must be encrypted. Encoded with `base64`.
- `nonce`: _string_
- `their_public`: _string_
- `secret`: _string_


## ResultOfNaclBox

- `encrypted`: _string_ –  Encrypted data. Encoded with `base64`.


## ParamsOfNaclBoxOpen

- `encrypted`: _string_ –  Data that must be decrypted. Encoded with `base64`.
- `nonce`: _string_
- `their_public`: _string_
- `secret`: _string_


## ResultOfNaclBoxOpen

- `decrypted`: _string_ –  Decrypted data. Encoded with `base64`.


## ParamsOfNaclSecretBox

- `decrypted`: _string_ –  Data that must be encrypted. Encoded with `base64`.
- `nonce`: _string_
- `key`: _string_


## ParamsOfNaclSecretBoxOpen

- `encrypted`: _string_ –  Data that must be decrypted. Encoded with `base64`.
- `nonce`: _string_
- `key`: _string_


## ParamsOfMnemonicWords

- `dictionary`?: _number_ –  dictionary identifier


## ResultOfMnemonicWords

- `words`: _string_ –  the list of mnemonic words


## ParamsOfMnemonicFromRandom

- `dictionary`?: _number_ –  dictionary identifier
- `word_count`?: _number_ –  mnemonic word count


## ResultOfMnemonicFromRandom

- `phrase`: _string_ –  string of mnemonic words


## ParamsOfMnemonicFromEntropy

- `entropy`: _string_
- `dictionary`?: _number_
- `word_count`?: _number_


## ResultOfMnemonicFromEntropy

- `phrase`: _string_


## ParamsOfMnemonicVerify

- `phrase`: _string_ –  phrase
- `dictionary`?: _number_ –  dictionary identifier
- `word_count`?: _number_ –  word count


## ResultOfMnemonicVerify

- `valid`: _boolean_ –  flag indicating the mnemonic is valid or not


## ParamsOfMnemonicDeriveSignKeys

- `phrase`: _string_ –  phrase
- `path`?: _string_ –  derivation path, for instance "m/44'/396'/0'/0/0"
- `dictionary`?: _number_ –  dictionary identifier
- `word_count`?: _number_ –  word count


## ParamsOfHDKeyXPrvFromMnemonic

- `phrase`: _string_ – string with seed phrase


## ResultOfHDKeyXPrvFromMnemonic

- `xprv`: _string_ –  serialized extended master private key


## ParamsOfHDKeyDeriveFromXPrv

- `xprv`: _string_ –  serialized extended private key
- `child_index`: _number_ –  child index (see BIP-0032)
- `hardened`: _boolean_ –  indicates the derivation of hardened/not-hardened key (see BIP-0032)


## ResultOfHDKeyDeriveFromXPrv

- `xprv`: _string_ –  serialized extended private key


## ParamsOfHDKeyDeriveFromXPrvPath

- `xprv`: _string_ –  serialized extended private key
- `path`: _string_ –  derivation path, for instance "m/44'/396'/0'/0/0"


## ResultOfHDKeyDeriveFromXPrvPath

- `xprv`: _string_ –  derived serialized extended private key


## ParamsOfHDKeySecretFromXPrv

- `xprv`: _string_ –  serialized extended private key


## ResultOfHDKeySecretFromXPrv

- `secret`: _string_ –  private key


## ParamsOfHDKeyPublicFromXPrv

- `xprv`: _string_ –  serialized extended private key


## ResultOfHDKeyPublicFromXPrv

- `public`: _string_ –  public key


