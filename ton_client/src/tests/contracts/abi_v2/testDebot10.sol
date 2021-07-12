pragma ton-solidity >=0.45.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
import "https://raw.githubusercontent.com/tonlabs/debots/main/Debot.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/Terminal/Terminal.sol";
import "../EncryptionBoxInput.sol";
import "../Sdk.sol";

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
        Terminal.print(0, format("Encryption Box Handle: {}", handle));
        m_boxHandle = handle;
        
        m_openedData = "Data to encrypt";
        Sdk.encrypt(tvm.functionId(setEncryptionResult), m_boxHandle, bytes(m_openedData));
        Sdk.encrypt(tvm.functionId(setEncryptionError), 100, bytes(m_openedData));
    }

    function setEncryptionError(uint32 result, bytes encrypted) public pure {
        require(result != 0, 406);
        encrypted;
    }

    function setEncryptionResult(uint32 result, bytes encrypted) public {
        require(result == 0, 402);
        Sdk.decrypt(tvm.functionId(setDecryptionResult), m_boxHandle, encrypted);
        Sdk.decrypt(tvm.functionId(setDecryptionError), 100, encrypted);
    }

    function setDecryptionError(uint32 result, bytes decrypted) public pure {
        require(result != 0, 405);
        decrypted;
    }

    function setDecryptionResult(uint32 result, bytes decrypted) public {
        require(result == 0, 403);
        require(tvm.hash(decrypted) == tvm.hash(bytes(m_openedData)), 400);
        EncryptionBoxInput.remove(tvm.functionId(setRemoveResult), m_boxHandle);
    }

    function setRemoveResult(bool removed) public {
        require(removed, 401);
        Terminal.print(0, "Test passed");
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
