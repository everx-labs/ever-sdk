pragma ton-solidity >=0.45.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
import "https://raw.githubusercontent.com/tonlabs/debots/main/Debot.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/Terminal/Terminal.sol";
import "./EncryptionBoxInput.sol";
import "./Sdk.sol";

contract ExampleContract is Debot {

    uint32 m_boxHandle;

    string m_openedData;

    uint256 m_public = 0x5f88dfedff20eef951ff26f4e5a88526929b195af091791a45cc8c6cb14961e4;
    uint256 m_private= 0xdcfe6f38f47ffbea6a2a51981c33ed655f7b849706544838101ea789ac7845ea;

    function start() public override {
        string nonce = "abcdefghijklmnopqrstuvwx";
        uint256 theirPubkey = m_public;
        EncryptionBoxInput.getNaclBox(
            tvm.functionId(setEncryptionBox), 
            "Choose encryption keys", 
            bytes(nonce),
            theirPubkey
        );
    }

    function setEncryptionBox(uint32 handle) public {
        Sdk.getEncryptionBoxInfo(tvm.functionId(printInfoResult), handle);
    }

    function printInfoResult(uint32 result, EncryptionBoxInfoResult info) public {
        Terminal.print(0, format("{} {}", info.hdpath, info.algorithm));//, info.options, info.public_info));
    }

    function getDebotInfo() public functionID(0xDEB) override view returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon
    ) {
        name = "TestDeBot10";
        version = "0.1.0";
        publisher = "TON Labs";
        caption = "TestDeBot10";
        author = "TON Labs";
        support = address(0);
        hello = "TestDeBot10";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ Terminal.ID, EncryptionBoxInput.ID ];
    }

}
