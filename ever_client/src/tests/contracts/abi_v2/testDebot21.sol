pragma ton-solidity >=0.65.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
// import required DeBot interfaces and basic DeBot contract.
import "https://raw.githubusercontent.com/everx-labs/debots/main/Debot.sol";
import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/Terminal/Terminal.sol";
import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/SigningBoxInput/SigningBoxInput.sol";
import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/Sdk/Sdk.sol";

contract Test21 is Debot {

    uint32 _handle;
    uint256 _hash1 = 0xc4c55326d914c1e1ce12a2f777b276f2bb52ea67ef705c46233c9c7b6e5cfe1e;
    uint256 _hash2 = 0xc55326d914c1e1ce12a2f777b276f2bb52ea67ef705c46233c9c7b6e5cfe1e;
    uint256 _hash3 = 0x1e;

    function start() public override {
        SigningBoxInput.get(tvm.functionId(setBoxHandle), "", [tvm.pubkey()]);
    }

    function setBoxHandle(uint32 handle) public {
        _handle = handle;
        Sdk.signHash(tvm.functionId(setSignature1), _handle, _hash1);
        Sdk.signHash(tvm.functionId(setSignature2), _handle, _hash2);
        Sdk.signHash(tvm.functionId(setSignature3), _handle, _hash3);
        Sdk.signHash(tvm.functionId(setSignature4), _handle, _hash3);
    }

    function setSignature1(bytes signature) public view {
        require(_checksign(_hash1, signature), 201);
    }

    function setSignature2(bytes signature) public view {
        require(_checksign(_hash2, signature), 202);
    }

    function setSignature3(bytes signature) public view {
        require(_checksign(_hash3, signature), 203);
    }

    function setSignature4(bytes signature) public view {
        TvmBuilder b;
        b.store(_hash3);
        require(tvm.checkSign(b.toCell().toSlice(), signature.toSlice(), tvm.pubkey()), 204);
    }

    function _checksign(uint256 hash, bytes signature) private view returns (bool) {
        return tvm.checkSign(hash, signature.toSlice(), tvm.pubkey());
    }

    /*
    *  Implementation of DeBot
    */
    function getDebotInfo() public functionID(0xDEB) override view returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon
    ) {
        name = "TestDeBot21";
        version = "0.1.0";
        publisher = "EverX";
        caption = "TestDeBot21";
        author = "EverX";
        support = address(0);
        hello = "TestDeBot21";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ SigningBoxInput.ID, Terminal.ID ];
    }

}
