pragma ever-solidity ^0.66.0;
pragma AbiHeader time;
pragma AbiHeader expire;
pragma AbiHeader pubkey;

contract Events {


    modifier OnlyOwner {
        require(msg.pubkey() == tvm.pubkey(), 100);
        tvm.accept();
        _;
    }
    event EventThrown(uint256 id);

    function emitValue(uint256 id) public pure {
    	tvm.accept();
        emit EventThrown(id);
    }

    function returnValue(uint256 id) public pure returns (uint256) {
    	tvm.accept();
        emit EventThrown(id);
        return id;
    }

    function sendAllMoney(address dest_addr) public OnlyOwner {
		selfdestruct(dest_addr);
	}
}
