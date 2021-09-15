pragma ton-solidity >=0.45.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
import "https://raw.githubusercontent.com/tonlabs/debots/main/Debot.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/Terminal/Terminal.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/JsonDeserialize/Json.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/Hex/Hex.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/EncryptionBoxInput/EncryptionBoxInput.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/Sdk/Sdk.sol";

contract ExampleContract is Debot {

    uint32 m_boxHandle;

    string m_openedData;

    uint256 m_public = 0x5f88dfedff20eef951ff26f4e5a88526929b195af091791a45cc8c6cb14961e4;
    uint256 m_private= 0xdcfe6f38f47ffbea6a2a51981c33ed655f7b849706544838101ea789ac7845ea;
    uint256 m_myTestPubkey = 0xb7cb10668eb106f91293014f6f47657f2f6b1b47332b4c865a874905271e95b3;
    string m_nonce;

    function start() public override {
        m_nonce = "abcdefghijklmnopqrstuvwx";
        uint256 theirPubkey = m_public;
        EncryptionBoxInput.getNaclBox(
            tvm.functionId(setEncryptionBox), 
            "Choose encryption keys", 
            bytes(m_nonce),
            theirPubkey
        );
    }

    function setEncryptionBox(uint32 handle) public {
        Sdk.getEncryptionBoxInfo(tvm.functionId(printInfoResult), handle);
    }

    function printInfoResult(uint32 result, EncryptionBoxInfoResult info) public {
        require(result == 0, 220);
        require(tvm.hash(info.hdpath) == tvm.hash("m/44'/396'/0'/0/1"), 201);
        require(tvm.hash(info.algorithm) == tvm.hash("NaclBox"), 202);
        Json.deserialize(tvm.functionId(setOptions), info.options);
        Json.deserialize(tvm.functionId(setPublicInfo), info.publicInfo);
    }

    struct Options {
        string nonce;
        uint256 theirPubkey;
    }
    function setOptions(bool result, Options obj) public {
        require(result, 203);
        require(obj.theirPubkey == m_public, 205);
        Hex.decode(tvm.functionId(setNonce), obj.nonce);
    }

    struct Info {
        uint256 key;
    }
    function setPublicInfo(bool result, Info obj) public {
        require(result, 206);
        require(obj.key == m_myTestPubkey, 207);
    }

    function setNonce(bytes data) public {
        require(tvm.hash(data) == tvm.hash(m_nonce), 204);
    }

    function getDebotInfo() public functionID(0xDEB) override view returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon
    ) {
        name = "TestDeBot11";
        version = "0.1.0";
        publisher = "TON Labs";
        caption = "TestDeBot11";
        author = "TON Labs";
        support = address(0);
        hello = "TestDeBot11";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ Terminal.ID, EncryptionBoxInput.ID ];
    }

}
