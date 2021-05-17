pragma ton-solidity >=0.35.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
// import required DeBot interfaces and basic DeBot contract.
import "../Debot.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/Terminal/Terminal.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/SigningBoxInput/SigningBoxInput.sol";

contract testDebot is Debot {

    /// @notice Entry point function for DeBot.
    function start() public override {
        uint256[] hints;
        SigningBoxInput.get(tvm.functionId(callSend), "Enter signing keys", hints);
    }

    function callSend(uint32 handle) public view {
        optional(uint256) pubkey = tvm.pubkey();
        optional(uint32) sbHandle = handle;
        this.send1{
            abiVer: 2,
            extMsg: true,
            sign: true,
            time: 0,
            expire: 0,
            pubkey: pubkey,
            signBoxHandle: sbHandle,
            callbackId: tvm.functionId(onSuccess1),
            onErrorId: tvm.functionId(onError1)
        }(2.2 ton, 3.5 ton);

        sbHandle = 0;
        this.send1{
            abiVer: 2,
            extMsg: true,
            sign: true,
            time: 0,
            expire: 0,
            pubkey: pubkey,
            signBoxHandle: sbHandle,
            callbackId: tvm.functionId(onSuccess1),
            onErrorId: tvm.functionId(onError4)
        }(2.2 ton, 3.5 ton);

        this.send2{
            abiVer: 2,
            extMsg: true,
            sign: true,
            time: 0,
            expire: 0,
            pubkey: pubkey,
            callbackId: tvm.functionId(onSuccess2),
            onErrorId: tvm.functionId(onError2)
        }();

        this.send3{
            abiVer: 2,
            extMsg: true,
            sign: true,
            time: 0,
            expire: 0,
            pubkey: pubkey,
            callbackId: tvm.functionId(onSuccess3),
            onErrorId: tvm.functionId(onError3)
        }();
    }

    function onSuccess1() public {
        Terminal.print(0, "Send1 succeeded");
    }

    function onSuccess2() public pure {
        require(false, 201);
    }

    function onSuccess3() public pure {
        require(false, 304);
    }

    function onError1(uint32 sdkError, uint32 exitCode) public pure {
        sdkError = sdkError;
        exitCode = exitCode;
        require(false, 200);
    }

    function onError2(uint32 sdkError, uint32 exitCode) public {
        require(sdkError == 812, 300);
        require(exitCode == 0, 301);
        Terminal.print(0, "Send2 rejected");
    }

    function onError3(uint32 sdkError, uint32 exitCode) public pure {
        require(sdkError == 414, 102);
        require(exitCode == 303, 103);
    }

    function onError4(uint32 sdkError, uint32 exitCode) public pure {
        require(sdkError == 810, 401);
        require(exitCode == 0, 402);
    }

    function send3() public view {
        require(false, 303);
    }

    function send2() public view {
        require(msg.pubkey() == tvm.pubkey(), 101);
        tvm.accept();
        address(this).transfer(10 ton, true, 1);
    }

    function send1(uint64 value1, uint64 value2) public view {
        require(msg.pubkey() == tvm.pubkey(), 101);
        tvm.accept();
        address(this).transfer(value1, true, 1);
        address addr = address.makeAddrStd(0, 0);
        addr.transfer(value2, false, 1);
    }

    /// @notice Returns Metadata about DeBot.
    function getDebotInfo() public functionID(0xDEB) override view returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon
    ) {
        name = "testDebot6";
        version = "0.1.0";
        publisher = "TON Labs";
        caption = "Test for approve callback and signing handle";
        author = "TON Labs";
        support = address(0);
        hello = "testDebot6";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ Terminal.ID, SigningBoxInput.ID ];
    }

}