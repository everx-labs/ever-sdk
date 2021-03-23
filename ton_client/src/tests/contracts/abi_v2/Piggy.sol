pragma solidity >=0.6.0;
pragma AbiHeader time;
pragma AbiHeader expire;

contract Piggy {

    bytes targetGoal;
    uint64 targetAmount;
    uint256 _owner;

    modifier onlyOwner {
        require(msg.pubkey() == _owner, 100);
        tvm.accept();
        _;
    }

    constructor(uint64 amount, bytes memory goal) public {
        _owner = tvm.pubkey();
        targetAmount = amount;
        targetGoal = goal;
    }

    function transfer(address payable to) public view onlyOwner {
        require(address(this).balance > targetAmount, 101);
        tvm.transfer(to, targetAmount, false, 128);
    }

    function getGoal() public view returns (bytes memory) {
        return targetGoal;
    }

    function getTargetAmount() public view returns (uint64) {
        return targetAmount;
    }

    function sendAllMoney(address payable dest_addr) public onlyOwner {
        tvm.accept();
        selfdestruct(dest_addr);
    }
}