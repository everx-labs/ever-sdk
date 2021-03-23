pragma solidity >=0.6.0;
pragma AbiHeader time;
pragma AbiHeader expire;

contract GiverV2 {

    mapping(uint256 => uint64) m_messages;

    modifier acceptOnlyOwner {
        require(msg.pubkey() == tvm.pubkey(), 101);
        tvm.accept(); 
        _;
	}

    /*
     * Publics
     */

    function sendTransaction(address dest, uint128 value, bool bounce) public acceptOnlyOwner {
        tvm.transfer(dest, value, bounce, 3);
        gc();
    }

    function sendAllMoney(address payable dest_addr) public acceptOnlyOwner {
        selfdestruct(dest_addr);
    }

    /*
     * Privates
     */
    
    /// @notice Function with predefined name called after signature check. Used to
    /// implement custom replay protection with parallel access.
    function afterSignatureCheck(TvmSlice body, TvmCell message) private inline 
        returns (TvmSlice) 
    {
        // load and drop message timestamp     
        body.ldu32();
        body.ldu32();
        uint64 expireAt = body.ldu32();
        require(expireAt >= now, 57);
        uint256 msgHash = tvm.hash(message);
        require(!m_messages.exists(msgHash), 102);
        m_messages[msgHash] = expireAt;

        return body;
    }

    /// @notice Allows to delete expired messages from dict.
    function gc() private inline {
        (uint256 msgHash, uint64 expireAt, bool ok) = m_messages.min();
        while (ok) {
            if (expireAt <= now) {
                delete m_messages[msgHash];
            }
            (msgHash, expireAt, ok) = m_messages.next(msgHash);
        }
    }

    /*
     * Get methods
     */
    struct Message {
        uint256 hash;
        uint64 expireAt;
    }
    function getMessages() public view returns (Message[] memory messages) {
       (uint256 msgHash, uint64 expireAt, bool ok) = m_messages.min();
        while (ok) {
            messages.push(Message(msgHash, expireAt));
            (msgHash, expireAt, ok) = m_messages.next(msgHash);
        }
    }
}