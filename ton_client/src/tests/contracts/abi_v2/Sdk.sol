pragma ton-solidity >= 0.40.0;

struct AccData {
    address id;
    TvmCell data;
}

struct EncryptionBoxInfoResult {
    string hdpath;
    string algorithm;
    string options;
    string publicInfo;
}

interface ISdk {
// accounts
function getBalance(uint32 answerId, address addr) external returns (uint128 nanotokens);
function getAccountType(uint32 answerId, address addr) external returns (int8 acc_type);
function getAccountCodeHash(uint32 answerId, address addr) external returns (uint256 code_hash);
function getAccountsDataByHash(uint32 answerId, uint256 codeHash, address gt) external returns (AccData[] accounts);
// encryption
function encrypt(uint32 answerId, uint32 boxHandle, bytes data) external returns (uint32 result, bytes encrypted);
function decrypt(uint32 answerId, uint32 boxHandle, bytes data) external returns (uint32 result, bytes decrypted);
function getEncryptionBoxInfo(uint32 answerId, uint32 boxHandle) external returns (uint32 result, EncryptionBoxInfoResult info);
// signing
function signHash(uint32 answerId, uint32 boxHandle, uint256 hash) external returns (bytes signature);
// crypto utils
function genRandom(uint32 answerId, uint32 length) external returns (bytes buffer);
// string
function substring(uint32 answerId, string str, uint32 start, uint32 count) external returns (string substr);

// [OBSOLETE] 
// Warning: DONT USE FUNCTIONS BELOW IN NEW DEBOTS. They will be removed from interface at any time in future.

// [OBSOLETE]
function mnemonicFromRandom(uint32 answerId, uint32 dict, uint32 wordCount)  external returns (string phrase);
// [OBSOLETE]
function mnemonicVerify(uint32 answerId, string phrase) external returns (bool valid);
// [OBSOLETE]
function mnemonicDeriveSignKeys(uint32 answerId, string phrase, string path) external returns (uint256 pub, uint256 sec);
// [OBSOLETE]
function hdkeyXprvFromMnemonic(uint32 answerId, string phrase) external returns (string xprv);
// [OBSOLETE]
function hdkeyDeriveFromXprv(uint32 answerId, string inXprv, uint32 childIndex, bool hardened) external returns (string xprv);
// [OBSOLETE]
function hdkeyDeriveFromXprvPath(uint32 answerId, string inXprv, string path)external returns (string xprv);
// [OBSOLETE]
function hdkeySecretFromXprv(uint32 answerId, string xprv) external returns (uint256 sec);
// [OBSOLETE]
function hdkeyPublicFromXprv(uint32 answerId, string xprv) external returns (uint256 pub);
// [OBSOLETE]
function naclSignKeypairFromSecretKey (uint32 answerId, uint256 secret)  external returns (uint256 sec, uint256 pub);
// [OBSOLETE]
function naclBox(uint32 answerId, bytes decrypted, bytes nonce, uint256 publicKey, uint256 secretKey) external returns (bytes encrypted);
// [OBSOLETE]
function naclBoxOpen(uint32 answerId, bytes encrypted, bytes nonce, uint256 publicKey, uint256 secretKey) external returns (bytes decrypted);
// [OBSOLETE]
function naclKeypairFromSecret(uint32 answerId, uint256 secret) external returns (uint256 publicKey, uint256 secretKey);
// [OBSOLETE]
function chacha20(uint32 answerId, bytes data, bytes nonce, uint256 key) external returns (bytes output);
}

library Sdk {

    uint256 constant ID = 0x8fc6454f90072c9f1f6d3313ae1608f64f4a0660c6ae9f42c68b6a79e2a1bc4b;
    int8 constant DEBOT_WC = -31;

    function getBalance(uint32 answerId, address addr) public pure {
        address a = address.makeAddrStd(DEBOT_WC, ID);
        ISdk(a).getBalance(answerId, addr);
    }
    function getAccountType(uint32 answerId, address addr) public pure {
        address a = address.makeAddrStd(DEBOT_WC, ID);
        ISdk(a).getAccountType(answerId, addr);
    }
    function getAccountCodeHash(uint32 answerId, address addr) public pure {
        address a = address.makeAddrStd(DEBOT_WC, ID);
        ISdk(a).getAccountCodeHash(answerId, addr);
    }
    function getAccountsDataByHash(uint32 answerId, uint256 codeHash, address gt) public pure {
        address addr = address.makeAddrStd(DEBOT_WC, ID);
        ISdk(addr).getAccountsDataByHash(answerId, codeHash, gt);
    }

    function encrypt(uint32 answerId, uint32 boxHandle, bytes data) public pure {
        address addr = address.makeAddrStd(DEBOT_WC, ID);
        ISdk(addr).encrypt(answerId, boxHandle, data);
    }
    function decrypt(uint32 answerId, uint32 boxHandle, bytes data) public pure {
        address addr = address.makeAddrStd(DEBOT_WC, ID);
        ISdk(addr).decrypt(answerId, boxHandle, data);
    }
    function getEncryptionBoxInfo(uint32 answerId, uint32 boxHandle) public pure {
        address addr = address.makeAddrStd(DEBOT_WC, ID);
        ISdk(addr).getEncryptionBoxInfo(answerId, boxHandle);
    }
    function signHash(uint32 answerId, uint32 boxHandle, uint256 hash) public pure {
        address addr = address.makeAddrStd(DEBOT_WC, ID);
        ISdk(addr).signHash(answerId, boxHandle, hash);
    }
    function genRandom(uint32 answerId, uint32 length) public pure {
        address addr = address.makeAddrStd(DEBOT_WC, ID);
        ISdk(addr).genRandom(answerId, length);
    }

    function mnemonicFromRandom(uint32 answerId, uint32 dict, uint32 wordCount) public pure {
        address addr = address.makeAddrStd(DEBOT_WC, ID);
        ISdk(addr).mnemonicFromRandom(answerId, dict, wordCount);
    }
    function mnemonicVerify(uint32 answerId, string phrase) public pure {
        address addr = address.makeAddrStd(DEBOT_WC, ID);
        ISdk(addr).mnemonicVerify(answerId, phrase);
    }
    function mnemonicDeriveSignKeys(uint32 answerId, string phrase, string path) public pure {
        address addr = address.makeAddrStd(DEBOT_WC, ID);
        ISdk(addr).mnemonicDeriveSignKeys(answerId, phrase, path);
    }

    //hdkey
    function hdkeyXprvFromMnemonic(uint32 answerId, string phrase)public pure {
        address addr = address.makeAddrStd(DEBOT_WC, ID);
        ISdk(addr).hdkeyXprvFromMnemonic(answerId, phrase);
    }
    function hdkeyDeriveFromXprv(uint32 answerId, string inXprv, uint32 childIndex, bool hardened) public pure {
        address addr = address.makeAddrStd(DEBOT_WC, ID);
        ISdk(addr).hdkeyDeriveFromXprv(answerId, inXprv, childIndex, hardened);
    }
    function hdkeyDeriveFromXprvPath(uint32 answerId, string inXprv, string path) public pure {
        address addr = address.makeAddrStd(DEBOT_WC, ID);
        ISdk(addr).hdkeyDeriveFromXprvPath(answerId, inXprv, path);
    }
    function hdkeySecretFromXprv(uint32 answerId, string xprv) public pure {
        address addr = address.makeAddrStd(DEBOT_WC, ID);
        ISdk(addr).hdkeySecretFromXprv(answerId, xprv);
    }
    function hdkeyPublicFromXprv(uint32 answerId, string xprv) public pure {
        address addr = address.makeAddrStd(DEBOT_WC, ID);
        ISdk(addr).hdkeyPublicFromXprv(answerId, xprv);
    }
    function naclSignKeypairFromSecretKey(uint32 answerId, uint256 secret) public pure {
        address addr = address.makeAddrStd(DEBOT_WC, ID);
        ISdk(addr).naclSignKeypairFromSecretKey(answerId, secret);
    }

    function substring(uint32 answerId, string str, uint32 start, uint32 count) public pure {
        address addr = address.makeAddrStd(DEBOT_WC, ID);
        ISdk(addr).substring(answerId, str, start, count);
    }

    function naclBox(uint32 answerId, bytes decrypted, bytes nonce, uint256 publicKey, uint256 secretKey) public pure {
        address addr = address.makeAddrStd(DEBOT_WC, ID);
        ISdk(addr).naclBox(answerId, decrypted, nonce, publicKey, secretKey);
    }
    function naclBoxOpen(uint32 answerId, bytes decrypted, bytes nonce, uint256 publicKey, uint256 secretKey) public pure {
        address addr = address.makeAddrStd(DEBOT_WC, ID);
        ISdk(addr).naclBoxOpen(answerId, decrypted, nonce, publicKey, secretKey);
    }
    function naclKeypairFromSecret(uint32 answerId, uint256 secret) public pure {
        address addr = address.makeAddrStd(DEBOT_WC, ID);
        ISdk(addr).naclKeypairFromSecret(answerId, secret);
    }
    function chacha20(uint32 answerId, bytes data, bytes nonce, uint256 key) public pure {
        address addr = address.makeAddrStd(DEBOT_WC, ID);
        ISdk(addr).chacha20(answerId, data, nonce, key);
    }
}

contract SdkABI is ISdk {
// account info
function getBalance(uint32 answerId, address addr) external override returns (uint128 nanotokens) {}
function getAccountType(uint32 answerId, address addr) external override returns (int8 acc_type) {}
function getAccountCodeHash(uint32 answerId, address addr) external override returns (uint256 code_hash) {}
function getAccountsDataByHash(uint32 answerId, uint256 codeHash, address gt) external override returns (AccData[] accounts) {}
// encryption
function encrypt(uint32 answerId, uint32 boxHandle, bytes data) external override returns (uint32 result, bytes encrypted) {}
function decrypt(uint32 answerId, uint32 boxHandle, bytes data) external override returns (uint32 result, bytes decrypted) {}
function getEncryptionBoxInfo(uint32 answerId, uint32 boxHandle) external override returns (uint32 result, EncryptionBoxInfoResult info) {}
// signing
function signHash(uint32 answerId, uint32 boxHandle, uint256 hash) external override returns (bytes signature) {}
// crypto utils
function genRandom(uint32 answerId, uint32 length) external override returns (bytes buffer) {}
// string
function substring(uint32 answerId, string str, uint32 start, uint32 count) external override returns (string substr) {}

// [OBSOLETE]

function mnemonicFromRandom(uint32 answerId, uint32 dict, uint32 wordCount)  external override returns (string phrase) {}
function mnemonicVerify(uint32 answerId, string phrase) external override returns (bool valid) {}
function mnemonicDeriveSignKeys(uint32 answerId, string phrase, string path) external override returns (uint256 pub, uint256 sec) {}

function hdkeyXprvFromMnemonic(uint32 answerId, string phrase) external override returns (string xprv) {}
function hdkeyDeriveFromXprv(uint32 answerId, string inXprv, uint32 childIndex, bool hardened) external override returns (string xprv) {}
function hdkeyDeriveFromXprvPath(uint32 answerId, string inXprv, string path)external override returns (string xprv) {}
function hdkeySecretFromXprv(uint32 answerId, string xprv) external override returns (uint256 sec) {}
function hdkeyPublicFromXprv(uint32 answerId, string xprv) external override returns (uint256 pub) {}
function naclSignKeypairFromSecretKey (uint32 answerId, uint256 secret)  external override returns (uint256 sec, uint256 pub) {}

function naclBox(uint32 answerId, bytes decrypted, bytes nonce, uint256 publicKey, uint256 secretKey) external override returns (bytes encrypted) {}
function naclBoxOpen(uint32 answerId, bytes encrypted, bytes nonce, uint256 publicKey, uint256 secretKey) external override returns (bytes decrypted) {}
function naclKeypairFromSecret(uint32 answerId, uint256 secret) external override returns (uint256 publicKey, uint256 secretKey) {}
function chacha20(uint32 answerId, bytes data, bytes nonce, uint256 key) external override returns (bytes output) {}

}
