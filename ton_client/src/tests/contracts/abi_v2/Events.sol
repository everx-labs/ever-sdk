pragma solidity >=0.5.0;
pragma AbiHeader time;
pragma AbiHeader pubkey;

contract Events {


    modifier OnlyOwner {
        require(msg.pubkey() == tvm.pubkey(), 100);
        tvm.accept();
        _;
    }
    event EventThrown(uint256 id);

    function emitValue(uint256 id) public OnlyOwner {
    	tvm.accept();
        emit EventThrown(id);
    }

    function returnValue(uint256 id) public OnlyOwner returns (uint256) {
    	tvm.accept();
        emit EventThrown(id);
        return id;
    }

    function sendAllMoney(address payable dest_addr) public OnlyOwner {
		selfdestruct(dest_addr);
	}
}