pragma ton-solidity >= 0.38.0;
pragma AbiHeader expire;
import "./Wallet.sol";
/// @title wallet with transfer delegation to so called subscription contract
/// @author Tonlabs
contract SubscrWallet is Wallet {
    /*
     *  Storage
     */
    address subscription;

    /*
     Exception codes:
      100 - message sender is not a wallet owner.
      101 - limit is overrun.
      102 - invalid transfer value.
      103 - destination address is zero.
     */

    modifier checkOwnerAndAccept override {
        address subscrAddress = subscription;
		require(tvm.pubkey() == msg.pubkey() ||
            (subscrAddress != address(0) && msg.sender == subscrAddress), 100);
        tvm.accept();
        _;
	}

    /*
       For subscription contract
     */
    function setSubscriptionAccount(address addr) public {
        require(msg.pubkey() == tvm.pubkey(), 100);
        tvm.accept();
        subscription = addr;
    }

    /*
     * Get methods
     */

    function getSubscriptionAccount() public view returns (address) {
        return subscription;
    }

    receive() external virtual override {}
}