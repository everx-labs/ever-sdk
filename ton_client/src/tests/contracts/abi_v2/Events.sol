pragma ton-solidity >= 0.38.0;
pragma AbiHeader time;
pragma AbiHeader expire;
pragma AbiHeader pubkey;

contract Events {
    modifier checkOwnerAndAccept  {
        require(msg.pubkey() == tvm.pubkey(), 100);
        tvm.accept();
        _;
    }
    event EventThrown(uint256 id);

    function emitValue(uint256 id) public pure checkOwnerAndAccept {
        emit EventThrown(id);
    }

    function returnValue(uint256 id) public pure checkOwnerAndAccept returns (uint256) {
        emit EventThrown(id);
        return id;
    }

    function sendAllMoney(address dest_addr) public checkOwnerAndAccept  {
		selfdestruct(dest_addr);
	}
}