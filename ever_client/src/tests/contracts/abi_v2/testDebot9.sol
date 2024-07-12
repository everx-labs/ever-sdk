pragma ton-solidity >=0.43.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
import "../Debot.sol";
import "../Terminal.sol";

//
//               -------               ------        -----
// startChain -> |  1  | deployNext -> |  2 | -> ... | N | --
//               -------               ------        -----  |
//                                       ^--- completed ----- 
contract TestDebot9 is Debot {
    uint256 constant CHAIN_LENGTH = 10;
    uint256 static _seqno;
    bool _completed;
    address _sender;

    function getseqno() public view returns(uint256 seqno) {
        seqno = _seqno;
    }

    function getcompleted() public view returns(bool completed) {
        completed = _completed;
    }

    /// @notice Entry point function for DeBot.
    function start() public override {
        optional(uint256) key = tvm.pubkey();
        TestDebot9(address(this)).startChain{
            abiVer: 2,
            extMsg: true,
            sign: true,
            time: 0,
            expire: 0,
            pubkey: key,
            callbackId: tvm.functionId(onSuccess),
            onErrorId: tvm.functionId(onError)
        }();
        
    }

    /// @notice Returns Metadata about DeBot.
    function getDebotInfo() public functionID(0xDEB) override view returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon
    ) {
        name = "TestDeBot9";
        version = "0.1.0";
        publisher = "TON Labs";
        caption = "TestDeBot9";
        author = "TON Labs";
        support = address(0);
        hello = "TestDeBot9";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ Terminal.ID ];
    }

    function onSuccess(uint256 n) public view {
        require(n == _seqno, 101);
        optional(uint256) none;
        address addr = address(tvm.hash(tvm.buildStateInit({
            code: tvm.code(),
            pubkey: tvm.pubkey(),
            varInit: {_seqno: _seqno + CHAIN_LENGTH},
            contr: TestDebot9
        })));
        TestDebot9(addr).getseqno{
            abiVer: 2,
            extMsg: true,
            sign: false,
            time: 0,
            expire: 0,
            pubkey: none,
            callbackId: tvm.functionId(onGetSeqno),
            onErrorId: tvm.functionId(onError)
        }();
    }

    function onError(uint32 sdkError, uint32 exitCode) public pure {
        sdkError; exitCode;
        revert(201);
    }

    function onGetSeqno(uint256 seqno) public view {
        require(seqno == _seqno + CHAIN_LENGTH, 301);
        optional(uint256) none;
        TestDebot9(address(this)).getcompleted{
            abiVer: 2,
            extMsg: true,
            sign: false,
            time: 0,
            expire: 0,
            pubkey: none,
            callbackId: tvm.functionId(onCompleted),
            onErrorId: tvm.functionId(onError)
        }();
    }

    function onCompleted(bool completed) public {
        require(completed, 401);
        Terminal.print(0, "Test passed");
    }
    //
    //  Onchain functions
    //

    function startChain() public returns (uint256 n) {
        tvm.accept();
        _seqno = 0;
        deployNext(_seqno);
        return _seqno;
    }

    function deployNext(uint256 initNo) public {
        _sender = msg.sender;
        if (initNo + CHAIN_LENGTH == _seqno) {
            TestDebot9(_sender).completed{value: 0, flag: 64}();
            return;
        }

        address addr = new TestDebot9 {
            value: 0.1 ton,
            flag: 1,
            code: tvm.code(),
            pubkey: tvm.pubkey(),
            varInit: {_seqno: _seqno + 1}
        }();
        TestDebot9(addr).deployNext{value: address(this).balance - 1 ton}(initNo);
    }

    function completed() public {
        _completed = true;
        if (_sender != address(0)) {
            TestDebot9(_sender).completed{value: 0, flag: 64}();
        }
    }
}