pragma ton-solidity >=0.47.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
// import required DeBot interfaces and basic DeBot contract.
import "https://raw.githubusercontent.com/tonlabs/debots/main/Debot.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/SigningBoxInput/SigningBoxInput.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/Terminal/Terminal.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/Query/Query.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/Sdk/Sdk.sol";

struct AbiHeader {
	optional(uint64) timestamp;
	optional(uint32) expire;
	optional(uint256) pubkey;
}
interface IMsg {
    function sendWithKeypair(uint32 answerId, TvmCell message, uint256 pub, uint256 sec) external;
    function sendAsync(TvmCell message) external returns(uint256 id);
	function sendWithHeader(TvmCell message, AbiHeader header) external returns(uint256 id);
}
library Msg {
	uint256 constant ID = 0x475a5d1729acee4601c2a8cb67240e4da5316cc90a116e1b181d905e79401c51;
	int8 constant DEBOT_WC = -31;

	function sendWithKeypair(uint32 answerId, TvmCell message, uint256 pub, uint256 sec) public {
		address addr = address.makeAddrStd(DEBOT_WC, ID);
		IMsg(addr).sendWithKeypair(answerId, message, pub, sec);
	}

	function sendAsync(TvmCell message) public {
		address addr = address.makeAddrStd(DEBOT_WC, ID);
		IMsg(addr).sendAsync(message);
	}

	function sendWithHeader(TvmCell message, AbiHeader header) public {
		address addr = address.makeAddrStd(DEBOT_WC, ID);
		IMsg(addr).sendWithHeader(message, header);
	}
}

abstract contract ARollingId {
    function answer(address dst) public {}
}

abstract contract ARecieverDebot {
    function headerCalback(uint64 timestamp, uint32 expire) virtual public {}
}

contract MsgTestDebot is Debot, ARecieverDebot {

    using JsonLib for JsonLib.Value;
    using JsonLib for mapping(uint256 => TvmCell);

    address m_rollingId;
    TvmCell m_sendMsg;
    uint64 m_timestamp;
    uint32 m_expire;
    uint64 m_sendTimestamp;
    uint32 m_sendExpire;
    uint32 m_handle;

    function getData() external view returns (uint64,uint32) {
        return (m_timestamp,m_expire);
    }

    function setRollingId(address a) public {
        require(msg.pubkey() == tvm.pubkey(), 101);
        tvm.accept();
        m_rollingId = a;
    }

    function headerCalback(uint64 timestamp, uint32 expire) override public {
        tvm.accept();
        m_timestamp = timestamp;
        m_expire = expire;
    }

    function start() public override {
        Sdk.genRandom(tvm.functionId(setRandom), 2);
    }

    function setRandom(bytes buffer) public {
        uint16 r = buffer.toSlice().decode(uint16);
        m_sendTimestamp = r+500;
        m_sendExpire = r+100;
        SigningBoxInput.get(tvm.functionId(setBoxHandle), "", [tvm.pubkey()]);
    }

    function setBoxHandle(uint32 handle) public {
        m_handle = handle;
        m_sendMsg = tvm.buildExtMsg({
            dest: m_rollingId,
            callbackId: tvm.functionId(transferSuccess),
            onErrorId: tvm.functionId(transferError),
            time: 0,
            expire: 0,
            sign: true,
            pubkey: null,
            signBoxHandle: m_handle,
            call: {ARollingId.answer, address(this)}
        });
        AbiHeader header = AbiHeader({
            timestamp: m_sendTimestamp,
            expire: m_sendExpire,
            pubkey: null
        });
        Msg.sendWithHeader(m_sendMsg, header);
    }


    function transferError(uint32 sdkError, uint32 exitCode) public pure {
        Terminal.print(0, format("Transaction failed. Sdk error = {}, Error code = {}\nDo you want to retry?", sdkError, exitCode));
        require(false, 108);
    }

    function transferSuccess(uint256 id) public pure {
        id;
        //wait for contract call our debot
        Query.waitForCollection(
            tvm.functionId(waitTransactionResult),
            QueryCollection.Transactions,
            format("{\"account_addr\":{\"eq\": \"{}\"},\"now\":{\"ge\": {}}}",address(this),now),
            "id now",
            60000
        );
    }

    function printQueryStatus(QueryStatus status) public pure {
        if (status == QueryStatus.FilterError)
            require(false, 102);
        else if (status == QueryStatus.NetworkError)
            require(false, 103);
        else if (status == QueryStatus.UnknownError)
            require(false, 104);
    }

    function waitTransactionResult(QueryStatus status, JsonLib.Value object) public pure {
        object;
        if (status == QueryStatus.Success) {
            MsgTestDebot(address(this)).getData{
                callbackId: getMsgDetails,
                onErrorId: onGetMethodError,
                time: 0,
                expire: 0,
                sign: false,
                pubkey: null
            }().extMsg;
        } else {
            printQueryStatus(status);
        }
    }

    function getMsgDetails(uint64 value0,uint32 value1) public view {
        require(value0==m_sendTimestamp, 105);
        require(value1==m_sendExpire, 106);
    }

    function onGetMethodError(uint32 sdkError, uint32 exitCode) public pure {
        Terminal.print(0, format("Get method error. Sdk error = {}, Error code = {}",sdkError, exitCode));
        require(false, 107);
    }

    /*
    *  Implementation of DeBot
    */
    function getDebotInfo() public functionID(0xDEB) override view returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon
    ) {
        name = "TestDeBot20";
        version = "0.1.0";
        publisher = "EverX";
        caption = "TestDeBot20";
        author = "EverX";
        support = address(0);
        hello = "TestDeBot20";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ SigningBoxInput.ID, Terminal.ID ];
    }
}
