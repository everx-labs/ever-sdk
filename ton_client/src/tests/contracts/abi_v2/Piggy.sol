pragma ton-solidity >= 0.38.0;
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

    constructor(uint64 amount, bytes goal) public {
        _owner = tvm.pubkey();
        targetAmount = amount;
        targetGoal = goal;
    }

    function transfer(address to) public view onlyOwner {
        require(address(this).balance > targetAmount, 101);
        to.transfer(targetAmount, false, 128);
    }

    function getGoal() public view returns (bytes) {
        return targetGoal;
    }

    function getTargetAmount() public view returns (uint64) {
        return targetAmount;
    }

    function sendAllMoney(address dest_addr) public onlyOwner {
        selfdestruct(dest_addr);
    }
}