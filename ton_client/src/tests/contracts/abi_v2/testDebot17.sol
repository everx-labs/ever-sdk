pragma ton-solidity >=0.40.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
import "https://raw.githubusercontent.com/tonlabs/debots/main/Debot.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/Query/Query.sol";

interface IMsg {

    function sendWithKeypair(uint32 answerId, TvmCell message, uint256 pub, uint256 sec) external;
    function sendAsync(TvmCell message) external returns(uint256 id);
}

library Msg {

	uint256 constant ID = 0x475a5d1729acee4601c2a8cb67240e4da5316cc90a116e1b181d905e79401c51;
	int8 constant DEBOT_WC = -31;

	function sendWithKeypair(uint32 answerId, TvmCell message, uint256 pub, uint256 sec) public pure {
		address addr = address.makeAddrStd(DEBOT_WC, ID);
		IMsg(addr).sendWithKeypair(answerId, message, pub, sec);
	}

	function sendAsync(TvmCell message) public pure {
		address addr = address.makeAddrStd(DEBOT_WC, ID);
		IMsg(addr).sendAsync(message);
	}

}

contract TestDebot17 is Debot {

    uint8 m_value;
    uint8 m_rand;
    TvmCell m_sendMsg;

    //onchanin function
    function setValue(uint8 value) public {
        require(tvm.pubkey() == msg.pubkey(),101);
        tvm.accept();
        m_value = value;
    }

    //onchanin function
    function getValue() public view returns(uint8 value) {
        value = m_value;
    }

    function start() public override {
        m_rand = 201;
        optional(uint256) none;
        m_sendMsg = tvm.buildExtMsg({
            abiVer: 2,
            dest: address(this),
            callbackId: tvm.functionId(onSuccess),
            onErrorId: tvm.functionId(onError),
            time: 0,
            expire: 0,
            sign: true,
            pubkey: none,
            call: {TestDebot17.setValue, m_rand}
        });
        Msg.sendAsync(m_sendMsg);
    }

    function onError(uint32 sdkError, uint32 exitCode) public {
        require(false,102);
    }

    function onSuccess(uint256 id) public {
        Query.waitForCollection(
            tvm.functionId(waitTransactionResult),
            QueryCollection.Transactions,
            format("{\"in_msg\": {\"eq\": \"{:064x}\"}}",id),
            "id",
            40000
        );
    }

    function waitTransactionResult(QueryStatus status, JsonLib.Value object) public {
        if(status == QueryStatus.Success) {
            optional(uint256) none;
            TestDebot17(address(this)).getValue{
                abiVer: 2,
                extMsg: true,
                callbackId: tvm.functionId(checkValue),
                onErrorId: tvm.functionId(onError),
                time: 0,
                expire: 0,
                sign: false,
                pubkey: none
            }();
        } else {
            require(false,103);
        }
    }

    function checkValue(uint8 value) public {
        require(value==m_rand,104);
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ Msg.ID, Query.ID ];
    }

    function getDebotInfo() public functionID(0xDEB) view override returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon) {
        name = "TestDeBot17";
        version = "0.1.0";
        publisher = "TON Labs";
        caption = "TestDeBot17";
        author = "TON Labs";
        support = address(0);
        hello = "TestDeBot17";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }
}