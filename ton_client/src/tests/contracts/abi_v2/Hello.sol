pragma solidity >=0.5.0;
pragma AbiHeader time;
pragma AbiHeader expire;

contract HelloTON {
    uint32 timestamp;

    // Modifier that allows public function to accept all external calls.
    modifier alwaysAccept {
        tvm.accept();
        _;
    }
    modifier onlyOwner {
        require(msg.pubkey() == tvm.pubkey(), 100);
        tvm.accept();
        _;
    }

    constructor() public {
        tvm.accept();
        timestamp = uint32(now);
    }
    //Function setting set value to state variable timestamp
    function touch() public alwaysAccept {
        timestamp = uint32(now);
    }
    //Function returns value of state variable timestamp
    function sayHello() public view returns (uint32) {
        return timestamp;
    }
    //Due to the modifier onlyOwner function sendAllMoney can be called only by the owner of the contract.
    //Function sendAllMoney send all contract's money to dest_addr.
    function sendAllMoney(address payable dest_addr) public onlyOwner {
        dest_addr.transfer(100000, false, 128|32);
    }
}
