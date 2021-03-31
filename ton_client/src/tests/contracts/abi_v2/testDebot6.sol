pragma ton-solidity >=0.35.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
// import required DeBot interfaces and basic DeBot contract.
import "../Debot.sol";
import "../Terminal.sol";

contract testDebot is Debot {

    /// @notice Entry point function for DeBot.
    function start() public override {
        optional(uint256) pubkey = tvm.pubkey();
        this.send1{
            abiVer: 2,
            extMsg: true,
            sign: true,
            time: 0,
            expire: 0,
            pubkey: pubkey,
            callbackId: tvm.functionId(onSuccess1),
            onErrorId: tvm.functionId(onError1)
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

    // @notice Define DeBot version and title here.
    function getVersion() public override returns (string name, uint24 semver) {
        (name, semver) = ("Test DeBot 6 for testing approve callback", _version(0,1,0));
    }

    function _version(uint24 major, uint24 minor, uint24 fix) private pure inline returns (uint24) {
        return (major << 16) | (minor << 8) | (fix);
    }

}