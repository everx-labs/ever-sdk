package io.tonlabs.client;

/**
 * Library Constants: API Methods
 * @author Martin Zeitler
**/
@SuppressWarnings(value = "unused")
public class Constants {

    /**
     * Access to TON Labs Development Network
     */
    public static final String GRAPHQL_DEV = "https://net.ton.dev/graphql";

    /**
     * Access to Telegram Test Network
     */
    public static final String GRAPHQL_TEST = "https://testnet.ton.dev/graphql";

    /**
     * Access to Free TON Main Network
     */
    public static final String GRAPHQL_MAIN = "https://main.ton.dev/graphql";

    /**
     * Access to TON OS SE for local testing
     * For Windows, use http://127.0.0.1/graphql or http://localhost/graphql
     * Not applicable for Android.
     */
    public static final String GRAPHQL_LOCAL = "http://0.0.0.0/graphql";

    // Core API
    public static final String CORE_API_VERSION                         = "version";
    public static final String CORE_API_SETUP                           = "setup";

    // Contracts
    public static final String CONTRACTS_RUN_LOCAL_MSG                  = "contracts.run.local.msg";
    public static final String CONTRACTS_RUN_LOCAL                      = "contracts.run.local";
    public static final String CONTRACTS_RUN_FEE                        = "contracts.run.fee";
    public static final String TVM_GET                                  = "tvm.get";
    public static final String CONTRACTS_DEPLOY_ADDRRESS                = "contracts.deploy.address";
    public static final String CONTRACTS_RUN_UNKNOWN_INPUT              = "contracts.run.unknown.input";
    public static final String CONTRACTS_RUN_UNKNOWN_OUTPUT             = "contracts.run.unknown.output";
    public static final String CONTRACTS_ENCODE_UNSIGNED_MESSAGE        = "contracts.deploy.encode_unsigned_message";
    public static final String CONTRACTS_RESOLVE_ERROR                  = "contracts.resolve.error";
    public static final String CONTRACTS_PARSE_MESSAGE                  = "contracts.parse.message";
    public static final String CONTRACTS_RUN_FEE_MSG                    = "contracts.run.fee.msg";
    public static final String CONTRACTS_DEPLOY_DATA                    = "contracts.deploy.data";
    public static final String CONTRACTS_DEPLOY_MESSAGE                 = "contracts.deploy.message";
    public static final String CONTRACTS_ENCODE_MESSAGE_WITH_SIGN       = "contracts.encode_message_with_sign";
    public static final String CONTRACTS_SEND_MESSAGE                   = "contracts.send.message";
    public static final String CONTRACTS_RUN_MESSAGE                    = "contracts.run.message";
    public static final String CONTRACTS_FUNCTION_ID                    = "contracts.function.id";
    public static final String CONTRACTS_PROCESS_MESSAGE                = "contracts.process.message";
    public static final String CONTRACTS_RUN                            = "contracts.run";
    public static final String CONTRACTS_PROCESS_TRANSACTION            = "contracts.process.transaction";
    public static final String CONTRACTS_RUN_OUTPUT                     = "contracts.run.output";
    public static final String CONTRACTS_RUN_ENCOD_UNSIGNED_MESSAGE     = "contracts.run.encode_unsigned_message";
    public static final String CONTRACTS_RUN_BODY                       = "contracts.run.body";
    public static final String CONTRACTS_ADDRESS_CONVERT                = "contracts.address.convert";
    public static final String CONTRACTS_DEPLOY                         = "contracts.deploy";
    public static final String CONTRACTS_WAIT_TRANSACTION               = "contracts.wait.transaction";
    public static final String CONTRACTS_LOAD                           = "contracts.load";
    public static final String CONTRACTS_FIND_SHARD                     = "contracts.find.shard";

    // Crypto
    public static final String CRYPTO_MNEMONIC_DERIVE_SIGN_KEYS         = "crypto.mnemonic.derive.sign.keys";
    public static final String CRYPTO_HDKEY_XPRV_SECRET                 = "crypto.hdkey.xprv.secret";
    public static final String CRYPTO_HDKEY_XPRV_DERIVE_PATH            = "crypto.hdkey.xprv.derive.path";
    public static final String CRYPTO_SHA256                            = "crypto.sha256";
    public static final String CRYPTO_HDKEY_XPRV_DERIVE                 = "crypto.hdkey.xprv.derive";
    public static final String CRYPTO_NACL_SIGN_OPEN                    = "crypto.nacl.sign.open";
    public static final String CRYPTO_NACL_BOX_OPEN                     = "crypto.nacl.box.open";
    public static final String CRYPTO_NACL_SECRET_BOX                   = "crypto.nacl.secret.box";
    public static final String CRYPTO_NACL_SIGN_DETACHED                = "crypto.nacl.sign.detached";
    public static final String CRYPTO_NACL_SIGN_KEYPAIR_FROM_SECRET_KEY = "crypto.nacl.sign.keypair.fromSecretKey";
    public static final String CRYPTO_TON_PUBLIC_KEY_STRING             = "crypto.ton_public_key_string";
    public static final String CRYPTO_NACL_SECRET_BOX_OPEN              = "crypto.nacl.secret.box.open";
    public static final String CRYPTO_NACL_SIGN_KEYPAIR                 = "crypto.nacl.sign.keypair";
    public static final String CRYPTO_NACL_SIGN                         = "crypto.nacl.sign";
    public static final String CRYPTO_MATH_MODULAR_POWER                = "crypto.math.modularPower";
    public static final String CRYPTO_ED25519_KEYPAIR                   = "crypto.ed25519.keypair";
    public static final String CRYPTO_RANDOM_GENERATE_BYTES             = "crypto.random.generateBytes";
    public static final String CRYPTO_NACL_BOX_KEYPAIR                  = "crypto.nacl.box.keypair";
    public static final String CRYPTO_SHA512                            = "crypto.sha512";
    public static final String CRYPTO_MNEMONIC_FROM_RANDOM              = "crypto.mnemonic.from.random";
    public static final String CRYPTO_MATH_FACTORIZE                    = "crypto.math.factorize";
    public static final String CRYPTO_NACL_BOX_                         = "crypto.nacl.box.keypair.fromSecretKey";
    public static final String CRYPTO_MNEMONIC_VERIFY                   = "crypto.mnemonic.verify";
    public static final String CRYPTO_TON_CRC16                         = "crypto.ton_crc16";
    public static final String CRYPTO_MNEMONIC_FROM_ENTROPY             = "crypto.mnemonic.from.entropy";
    public static final String CRYPTO_HDKEY_XPRV_FROM_MNEMONIC          = "crypto.hdkey.xprv.from.mnemonic";
    public static final String CRYPTO_NACL_BOX                          = "crypto.nacl.box";
    public static final String CRYPTO_HDKEY_XPRV_PUBLIC                 = "crypto.hdkey.xprv.public";
    public static final String CRYPTO_SCRYPT                            = "crypto.scrypt";
    public static final String CRYPTO_MNEMONIC_WORDS                    = "crypto.mnemonic.words";

    // Queries
    public static final String QUERIES_UNSUBSCRIBE                      = "queries.unsubscribe";
    public static final String QUERIES_SUBSCRIBE                        = "queries.subscribe";
    public static final String QUERIES_QUERY                            = "queries.query";
    public static final String QUERIES_GET_NEXT                         = "queries.get.next";
    public static final String QUERIES_WAIT_FOR                         = "queries.wait.for";
}
