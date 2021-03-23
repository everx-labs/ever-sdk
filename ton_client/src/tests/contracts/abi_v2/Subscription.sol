pragma solidity >=0.5.0;
pragma AbiHeader time;
pragma AbiHeader expire;

contract SubscriptionContract {

    address mywallet;
    mapping (uint256 => Payment) subscriptions;
    uint256 _owner;

    uint8 constant STATUS_ACTIVE   = 1;
    uint8 constant STATUS_EXECUTED = 2;


    struct Payment {
        uint256 pubkey;
        address payable to;
        uint64 value;
        uint32 period;
        uint32 start;
        uint8 status;
    }

    modifier onlyOwner {
        require(msg.pubkey() == tvm.pubkey(), 100);        
        _;
    }
    constructor(address wallet) public {
        mywallet = wallet;
    }

    function getWallet() public view returns (address) {
        return mywallet;
    }

    function getSubscription(uint256 subscriptionId) public view returns (Payment memory) {
        return subscriptions[subscriptionId];
    }

    function subscribe(
        uint256 subscriptionId,
        uint256 pubkey,
        address payable to,
        uint64 value,
        uint32 period) public onlyOwner {
        require(subscriptionId != 0 &&
            value > 0 &&
            period > 0, 101);
        tvm.accept();
        subscriptions[subscriptionId] = Payment(pubkey, to, value, period, 0, STATUS_ACTIVE);
    }

    function cancel(uint256 subscriptionId) public onlyOwner {
        require(subscriptions[subscriptionId].status != 0, 101);
        tvm.accept();
        delete subscriptions[subscriptionId];
    }

    function executeSubscription(uint256 subscriptionId) public {
        Payment storage subscr = subscriptions[subscriptionId];
        require(msg.pubkey() == subscr.pubkey, 102);
        require(subscr.status != 0, 101);
        if (now > (subscr.start + subscr.period)) {
            subscr.start = uint32(now);
        } else {
            require(subscr.status != STATUS_EXECUTED, 103);
        }
        tvm.accept();

        subscr.to.transfer(subscr.value);
        subscr.status = STATUS_EXECUTED;

        subscriptions[subscriptionId] = subscr;
    }
}