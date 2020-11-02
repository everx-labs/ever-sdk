package io.tonlabs.client;

/**
 * Library Constants
 * @author Martin Zeitler
 **/
@SuppressWarnings(value = "unused")
public class Constants {

    /**
     * Access to TON Labs Development Network
     */
    public static final String GRAPHQL_DEV = "net.ton.dev";

    /**
     * Access to Telegram Test Network
     */
    public static final String GRAPHQL_TEST = "testnet.ton.dev";

    /**
     * Access to Free TON Main Network
     */
    public static final String GRAPHQL_MAIN = "main.ton.dev";

    /**
     * Table Name: Messages
     */
    public static final String TABLE_NAME_MESSAGES = "messages";

    /**
     * Table Name: Contracts
     */
    public static final String TABLE_NAME_CONTRACTS = "accounts";

    /**
     * Table Name: Blocks
     */
    public static final String TABLE_NAME_BLOCKS = "blocks";

    /**
     * Table Name: Transactions
     */
    public static final String TABLE_NAME_TRANSACTIONS = "transactions";

    /**
     * DEFAULT_RETRIES_COUNT
     */
    public static final int DEFAULT_RETRIES_COUNT = 5;

    /**
     * DEFAULT_EXPIRATION_TIMEOUT
     */
    public static final long DEFAULT_EXPIRATION_TIMEOUT = 40000;

    /**
     * DEFAULT_PROCESSING_TIMEOUT
     */
    public static final long DEFAULT_PROCESSING_TIMEOUT = -1;

    /**
     * DEFAULT_TIMEOUT_GROW_FACTOR
     */
    public static final float DEFAULT_TIMEOUT_GROW_FACTOR = 1.5f;

    /**
     * DEFAULT_WAIT_TIMEOUT
     */
    public static final long DEFAULT_WAIT_TIMEOUT = 40000;

    /**
     * MASTERCHAIN_ID
     */
    public static final int MASTERCHAIN_ID = -1;
}
