pragma ton-solidity >=0.47.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
import "https://raw.githubusercontent.com/tonlabs/debots/main/Debot.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/SigningBoxInput/SigningBoxInput.sol";

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

contract TestDebot19 is Debot {
//check exitCode for transaction
//Expected Result
//first call sync error sdk=414 exitcode=101
//second call sync error sdk=0 exitcode=102
//third call sync success
//fouth call async error sdk=414 exitcode=101
//fifth call async error sdk=0 exitcode=102
//sixth call async success

    uint8 m_value;
    uint32 m_sbHandle;

    //onchanin function
    function onchainCall(uint8 value) public {
        require(value > 10, 101);
        tvm.accept();
        tvm.commit();
        require(value > 20, 102);
        m_value = value;
    }

    //onchanin function
    function getValue() public view returns(uint8 value) {
        value = m_value;
    }

    //start DeBot
    function start() public override {
        SigningBoxInput.get(tvm.functionId(setBoxHandle), "", [tvm.pubkey()]);
    }

    function setBoxHandle(uint32 handle) public {
        m_sbHandle = handle;
        this.firstCall();
    }

    function firstCall() public {
        this.onchainCall{
            callbackId: tvm.functionId(onFirstCallSuccess),
            onErrorId: tvm.functionId(onFirstCallError),
            abiVer: 2,
            extMsg: true,
            sign: true,
            time: 0,
            expire: 0,
            pubkey: 0,
            signBoxHandle: m_sbHandle
        }(5);
    }

    function onFirstCallError(uint32 sdkError, uint32 exitCode) public {
        require(sdkError==414, 312);
        require(exitCode==101, 313);
        this.secondCall();
    }

    function onFirstCallSuccess() public {
        require(false,311);
    }

    function secondCall() public {
        this.onchainCall{
            callbackId: tvm.functionId(onSecondCallSuccess),
            onErrorId: tvm.functionId(onSecondCallError),
            abiVer: 2,
            extMsg: true,
            sign: true,
            time: 0,
            expire: 0,
            pubkey: 0,
            signBoxHandle: m_sbHandle
        }(15);
    }

    function onSecondCallError(uint32 sdkError, uint32 exitCode) public {
        require(sdkError==0, 322);
        require(exitCode==102, 323);
        this.thirdCall();
    }

    function onSecondCallSuccess() public {
        require(false,321);
    }

    function thirdCall() public {
        this.onchainCall{
            callbackId: tvm.functionId(onThirdCallSuccess),
            onErrorId: tvm.functionId(onThirdCallError),
            abiVer: 2,
            extMsg: true,
            sign: true,
            time: 0,
            expire: 0,
            pubkey: 0,
            signBoxHandle: m_sbHandle
        }(25);
    }

    function onThirdCallError(uint32 sdkError, uint32 exitCode) public {
        require(false,331);
    }

    function onThirdCallSuccess() public {
        this.fourthCall();
    }

    function fourthCall() public {
        TvmCell sendMsg = tvm.buildExtMsg({
            abiVer: 2,
            dest: address(this),
            callbackId: tvm.functionId(onFourthCallSuccess),
            onErrorId: tvm.functionId(onFourthCallError),
            time: 0,
            expire: 0,
            sign: true,
            pubkey: 0,
            call: {TestDebot19.onchainCall, 5}
        });
        Msg.sendAsync(sendMsg);
    }

    function onFourthCallError(uint32 sdkError, uint32 exitCode) public {
        require(sdkError==414, 342);
        require(exitCode==101, 343);
        this.fifthCall();
    }

    function onFourthCallSuccess(uint256 id) public {
        require(false,341);
    }

    function fifthCall() public {
        TvmCell sendMsg = tvm.buildExtMsg({
            abiVer: 2,
            dest: address(this),
            callbackId: tvm.functionId(onFifthCallSuccess),
            onErrorId: tvm.functionId(onFifthCallError),
            time: 0,
            expire: 0,
            sign: true,
            pubkey: 0,
            call: {TestDebot19.onchainCall, 15}
        });
        Msg.sendAsync(sendMsg);
    }

    function onFifthCallError(uint32 sdkError, uint32 exitCode) public {
        require(sdkError==0, 352);
        require(exitCode==102, 353);
        this.sixthCall();
    }

    function onFifthCallSuccess(uint256 id) public {
        require(false,351);
    }

    function sixthCall() public {
        TvmCell sendMsg = tvm.buildExtMsg({
            abiVer: 2,
            dest: address(this),
            callbackId: tvm.functionId(onSixthCallSuccess),
            onErrorId: tvm.functionId(onSixthCallError),
            time: 0,
            expire: 0,
            sign: true,
            pubkey: 0,
            call: {TestDebot19.onchainCall, 25}
        });
        Msg.sendAsync(sendMsg);
    }

    function onSixthCallError(uint32 sdkError, uint32 exitCode) public {
        require(false,361);
    }

    function onSixthCallSuccess(uint256 id) public {
        require(id!=0,362);
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ Msg.ID, SigningBoxInput.ID ];
    }

    function getDebotInfo() public functionID(0xDEB) view override returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon) {
        name = "TestDeBot19";
        version = "0.1.0";
        publisher = "TON Labs";
        caption = "TestDeBot19";
        author = "TON Labs";
        support = address(0);
        hello = "TestDeBot19";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }
}