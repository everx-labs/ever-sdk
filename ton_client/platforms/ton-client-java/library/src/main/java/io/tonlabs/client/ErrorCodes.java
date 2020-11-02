package io.tonlabs.client;

/**
 * Error Constants
 * @see <a href="https://docs.ton.dev/86757ecb2/p/3874d1-error-reference">Error Reference</a>
 * @author Martin Zeitler
 **/
@SuppressWarnings(value = "unused")
public class ErrorCodes {

    /** An unknown method name passed to SDK JSON API. */
    public static final int ERROR_UNKNOWN_METHOD = 1;

    /** Invalid format of the Parameters JSON. */
    public static final int ERRO_INVALID_JSON_PARAM_FORMAT = 2;

    /** Invalid context handle. */
    public static final int ERROR_INVALID_CONTEXT_HANDLE = 3;

    /** Config initialization failed. */
    public static final int ERROR_1000 = 1000;

    /** Failed to send node request. */
    public static final int ERROR_1001 = 1001;

    /** Function run local failed: account does not exist. */
    public static final int ERROR_1002 = 1002;

    /** Operation timeout. */
    public static final int ERROR_1003 = 1003;

    /** Internal error. */
    public static final int ERROR_1004 = 1004;

    /** Query failed. */
    public static final int ERROR_1005 = 1005;

    /** Message expired. */
    public static final int ERROR_1006 = 1006;

    /** Server doesn't support aggregations. */
    public static final int ERROR_1007 = 1007;

    /** Invalid CONS structure. Each CONS item must consist of two elements. */
    public static final int ERROR_1008 = 1008;

    /** Address required for run local. You haven't specified contract code or data so address is required to load missing parts from network. */
    public static final int ERROR_1009 = 1009;

    /** No blocks produced during timeout. */
    public static final int ERROR_1010 = 1010;

    /** Existing block transaction not found. */
    public static final int ERROR_1011 = 1011;

    /** Transaction was not produced during the messageProcessingTimeout - only for contracts that do not support pragra expire. */
    public static final int ERROR_1012 = 1012;

    /** Your local clock is out of sync with the server time. It is a critical condition for sending messages to the blockchain. Please sync you clock with the internet time. */
    public static final int ERROR_1013 = 1013;

    /** Account does not exist. You have to prepaid this account to have a positive balance on them and then deploy a contract code for this account. */
    public static final int ERROR_1014 = 1014;

    /** Account with address exists but haven't a contract code yet. You have to ensure that an account has an enough balance for deploying a contract code and then deploy a contract code for this account. */
    public static final int ERROR_1015 = 1015;

    /** Account with address has too low balance. You have to send some value to account balance from other contract (e.g. Wallet contract). */
    public static final int ERROR_1016 = 1016;

    /** Account was frozen due storage phase. */
    public static final int ERROR_1017 = 1017;

    /** Invalid public key format or size. All keys must be provided in hex-encoded strings optionally prefixed with 'x' or 'X' or '0x' or '0X'. */
    public static final int ERROR_2001 = 2001;

    /** Invalid secret key format or size. All keys must be provided in hex-encoded strings optionally prefixed with 'x' or 'X' or '0x' or '0X'. */
    public static final int ERROR_2002 = 2002;

    /** Invalid key format or size. All keys must be provided in hex-encoded strings optionally prefixed with 'x' or 'X' or '0x' or '0X'. */
    public static final int ERROR_2003 = 2003;

    /** Invalid address format or size. All addresses must be provided in hex-encoded strings optionally prefixed with 'x' or 'X' or '0x' or '0X'. */
    public static final int ERROR_2004 = 2004;

    /** Invalid user data format. All user data must be provided in proper hex-encoded strings. */
    public static final int ERROR_2005 = 2005;

    /** Invalid user data format. All user data must be provided in proper base64-encoded strings. */
    public static final int ERROR_2006 = 2006;

    /** Invalid factorize challenge. */
    public static final int ERROR_2007 = 2007;

    /** Invalid big int. */
    public static final int ERROR_2008 = 2008;

    /** Input data for conversion function is missing. Expected are one of { text: "..." }, { hex: "..." } or { base64: "..."} */
    public static final int ERROR_2009 = 2009;

    /** Output data for conversion function can not be encoded to utf8. */
    public static final int ERROR_2010 = 2010;

    /** The provided scrypt has failed meaning there's a typo in the message. */
    public static final int ERROR_2011 = 2011;

    /** Invalid size key. 512-bit keys are 64 symbols long, 256-bit keys are 32 symbols long and 192-bit keys are 24 symbols long. */
    public static final int ERROR_2012 = 2012;

    /** Secretbox failed */
    public static final int ERROR_2013 = 2013;

    /** nacl.box failed  */
    public static final int ERROR_2014 = 2014;

    /** nacl.sign failed */
    public static final int ERROR_2015 = 2015;

    /** Invalid bip39 entropy */
    public static final int ERROR_2016 = 2016;

    /** Invalid bip39 phrase. */
    public static final int ERROR_2017 = 2017;

    /** Invalid bip32 key. */
    public static final int ERROR_2018 = 2018;

    /** Invalid bip32 derive path. */
    public static final int ERROR_2019 = 2019;

    /** Keystore handle is invalid or was removed. */
    public static final int ERROR_2020 = 2020;

    /** Either Key or Keystore Handle must be specified. */
    public static final int ERROR_2021 = 2021;

    /** Invalid mnemonic dictionary. */
    public static final int ERROR_2022 = 2022;

    /** Invalid mnemonic word count. */
    public static final int ERROR_2023 = 2023;

    /** Generating mnemonic phrase failed. */
    public static final int ERROR_2024 = 2024;

    /** Generating mnemonics from entropy failed. */
    public static final int ERROR_2025 = 2025;

    /** Load contract failed. */
    public static final int ERROR_3001 = 3001;

    /** Invalid contract image. */
    public static final int ERROR_3002 = 3002;

    /** Image creation failed. */
    public static final int ERROR_3003 = 3003;

    /** Deploy failed: transaction missing. */
    public static final int ERROR_3004 = 3004;

    /** Decode run output failed. */
    public static final int ERROR_3005 = 3005;

    /** Decode run input failed. */
    public static final int ERROR_3006 = 3006;

    /** Contract execution failed. */
    public static final int ERROR_3007 = 3007;

    /** Contract load failed. */
    public static final int ERROR_3008 = 3008;

    /** Transaction missing. */
    public static final int ERROR_3009 = 3009;

    /** Send message failed. */
    public static final int ERROR_3010 = 3010;

    /** Create deploy message failed. */
    public static final int ERROR_3011 = 3011;

    /** Create run message failed. */
    public static final int ERROR_3012 = 3012;

    /** Create send tokens message failed. */
    public static final int ERROR_3013 = 3013;

    /** Encoding message with sign failed. */
    public static final int ERROR_3014 = 3014;

    /** Deploy failed: transaction aborted. */
    public static final int ERROR_3015 = 3015;

    /** Run body creation failed. */
    public static final int ERROR_3016 = 3016;

    /** Get function ID failed. */
    public static final int ERROR_3017 = 3017;

    /** Local run failed. */
    public static final int ERROR_3018 = 3018;

    /** Address conversion failed. */
    public static final int ERROR_3019 = 3019;

    /** Invalid Bag of Cells. */
    public static final int ERROR_3020 = 3020;

    /** Load messages failed. */
    public static final int ERROR_3021 = 3021;

    /** Can not serialize message. */
    public static final int ERROR_3022 = 3022;

    /** Process message failed. */
    public static final int ERROR_3023 = 3023;

    /** Load contract failed: Account exists but code is not deployed. */
    public static final int ERROR_3024 = 3024;

    /** This error means that local execution of the contract failed. See the contract error in data.exit_code field . Look for the description below - among the compute phase errors. Ask the contract developer for more info. */
    public static final int ERROR_3025 = 3025;

    /** Query failed. */
    public static final int ERROR_4001 = 4001;

    /** Queries Subscribe failed. */
    public static final int ERROR_4002 = 4002;

    /** Queries `WaitFor` failed. */
    public static final int ERROR_4003 = 4003;

    /** Queries Get next failed. */
    public static final int ERROR_4004 = 4004;

    /** Reserved for wallet errors. */
    public static final int ERROR_5000 = 5000;
}
