pragma ton-solidity >= 0.38.0;
pragma AbiHeader time;
pragma AbiHeader expire;

contract GiverV2 {
     uint8 constant MAX_CLEANUP_MSGS = 30;

 
    // mapping to store hashes of inbound messages;
     mapping(uint => uint32) messages;

    modifier checkOwnerAndAccept  {
        require(msg.pubkey() == tvm.pubkey(), 101);
        tvm.accept(); 
        _;
	}

    /*
     * Publics
     */

    function sendTransaction(address dest, uint128 value, bool bounce) public  checkOwnerAndAccept  {
        dest.transfer(value, bounce, 3);
        gc();
    }

    function sendAllMoney(address dest_addr) public checkOwnerAndAccept  {
        selfdestruct(dest_addr);
    }

    /*
     * Privates
     */
    
    /// @notice Function with predefined name called after signature check. Used to
    /// implement custom replay protection with parallel access.
    function afterSignatureCheck(TvmSlice body, TvmCell message) private inline returns (TvmSlice) {
        // Via TvmSlice methods we read header fields from the message body

        body.decode(uint64); // The first 64 bits contain timestamp which is usually used to differentiate messages.
        uint32 expireAt = body.decode(uint32);

        require(expireAt >= now, 101);   // Check that message is not expired.

        // Runtime function tvm.hash() allows to calculate the hash of the message.
        uint hash = tvm.hash(message);

        // Check that the message is unique.
        require(!messages.exists(hash), 102);

        // Save the hash of the message in  the state variable.
        messages[hash] = expireAt;

        // After reading message headers this function must return the rest of the body slice.
        return body;
    }


    /// @notice Allows to delete expired messages from dict.
    function gc() private {
        optional(uint256, uint32) res = messages.min();
        uint8 counter = 0;
        while (res.hasValue() && counter < MAX_CLEANUP_MSGS) {
            (uint256 msgHash, uint32 expireAt) = res.get();
            if (expireAt < now) {
                delete messages[msgHash];
            }
            counter++;
            res = messages.next(msgHash);
        }
    }
}