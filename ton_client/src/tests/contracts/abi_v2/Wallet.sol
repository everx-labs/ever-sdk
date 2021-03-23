pragma solidity >=0.6.0;
pragma AbiHeader expire;

/// @title Simple wallet
/// @author Tonlabs
contract Wallet {
    /*
     *  Storage
     */

    /*
     Exception codes:
      100 - message sender is not a wallet owner.
     */

    modifier checkOwnerAndAccept virtual {
		require(msg.pubkey() == tvm.pubkey(), 100);
        tvm.accept();
        _;
	}

    /*
     * Public functions
     */

    /// @dev Allows to transfer grams to destination account.
    /// @param dest Transfer target address.
    /// @param value Nanograms value to transfer.
    /// @param bounce Flag that enables bounce message in case of target contract error.
    function sendTransaction(address dest, uint128 value, bool bounce) public checkOwnerAndAccept virtual {
        require(value > 0 && value < address(this).balance, 101);
        tvm.transfer(dest, value, bounce, 3);
    }

    receive() external payable virtual {}
}