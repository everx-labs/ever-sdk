pragma ton-solidity >=0.45.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
import "https://raw.githubusercontent.com/tonlabs/debots/main/Debot.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/Terminal/Terminal.sol";
import "EncryptionBoxInput.sol";
import "Sdk.sol";

contract ExampleContract is Debot {

    uint32 m_boxHandle;
    string m_openedData;
    uint m_currentBoxNum;
    uint256 m_public = 0x5f88dfedff20eef951ff26f4e5a88526929b195af091791a45cc8c6cb14961e4;
    uint256 m_private= 0xdcfe6f38f47ffbea6a2a51981c33ed655f7b849706544838101ea789ac7845ea;
    string nonce = "abcdefghijklmnopqrstuvwx";
    
    function start() public override {
        m_currentBoxNum = 0;
        Terminal.print(0,"printit ffs!");
        _nextBox();
        EncryptionBoxInput.getSupportedAlgorithms(tvm.functionId(setAlgorithms));
    }

    function setAlgorithms(string[] names) public {
        Terminal.print(0, "Supported algorithms:");
        for (string name: names) {
            Terminal.print(0, format("{}", name));
        }
        _nextBox();
    }

    function getNaclSecretBox() public {
        EncryptionBoxInput.getNaclSecretBox(
            tvm.functionId(callGetInfo),
            "Choose encryption keys", 
            bytes(nonce)
        );
    }

    function callGetInfo(uint32 handle) public {
        Sdk.getEncryptionBoxInfo(tvm.functionId(printInfoResult), handle);
    }

    function setBox(uint32 handle) public {
        Terminal.print(0, format("Handle created: {}", handle));
        m_boxHandle = handle;
        m_openedData = "Data to encrypt";
        Sdk.encrypt(tvm.functionId(setEncryptionResult), m_boxHandle, m_openedData);
    }

    function printInfoResult(uint32 result, EncryptionBoxInfoResult info) public {
        Terminal.print(0, format("INFO WAS {} {} {} {}", info.hdpath, info.algorithm, info.options, info.public_info));
    }

    function setEncryptionResult(uint32 result, bytes encrypted) public {
        if (result != 0) {
            Terminal.print(tvm.functionId(Debot.start), format("Failed to encrypt: error {}", result));
            return;
        }
        Terminal.print(0, "Encrypt succeeded");
        Sdk.decrypt(tvm.functionId(setDecryptionResult), m_boxHandle, encrypted);
    }

    function setDecryptionResult(uint32 result, bytes decrypted) public {
        if (result != 0) {
            Terminal.print(tvm.functionId(Debot.start), format("Failed to encrypt: error {}", result));
            return;
        }
        require(tvm.hash(decrypted) == tvm.hash(bytes(m_openedData)), 400, "invalid decrypted data");
        Terminal.print(0, "Decrypt succeeded");
        EncryptionBoxInput.remove(tvm.functionId(setRemoveResult), m_boxHandle);
    }

    function setRemoveResult(bool removed) public {
        require(removed, 401);
        Terminal.print(0, "Handle removed");
        m_boxHandle = 0;
        _nextBox();
    }

    function _nextBox() private {
        Terminal.print(
            tvm.functionId(getNaclSecretBox), 
            format("Creating NaclSecretBox with parameters:\nnonce: {}", nonce)
        );
    }

    function getDebotInfo() public functionID(0xDEB) override view returns(
        string name, string version, string publisher, string key, string author,
        address support, string hello, string language, string dabi, bytes icon
    ) {
        name = "EncryptionBoxInput example DeBot";
        version = "0.1.0";
        publisher = "TON Labs";
        key = "How to use EncryptionBoxInput interface";
        author = "TON Labs";
        support = address(0);
        hello = "Hello, i am an example DeBot.";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ Terminal.ID, EncryptionBoxInput.ID ];
    }

}
