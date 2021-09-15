pragma ton-solidity >=0.47.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
import "https://raw.githubusercontent.com/tonlabs/debots/main/Debot.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/Terminal/Terminal.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/SigningBoxInput/SigningBoxInput.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/Sdk/Sdk.sol";

contract testDebot is Debot {

    /// @notice Entry point function for DeBot.
    function start() public override {
        uint256[] hints = [tvm.pubkey()];
        SigningBoxInput.get(tvm.functionId(checkHandleInfo), "Enter signing keys", hints);
    }

    function checkHandleInfo(uint32 handle) public {
        Sdk.getSigningBoxInfo(tvm.functionId(setHandleInfo), handle);
    }

    function setHandleInfo(uint32 result, uint256 key) public view {
        require(result == 0, 101);
        require(tvm.pubkey() == key, 102);
    }

    function getDebotInfo() public functionID(0xDEB) override view returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon
    ) {
        name = "TestDeBot12";
        version = "0.1.0";
        publisher = "TON Labs";
        caption = "TestDeBot12";
        author = "TON Labs";
        support = address(0);
        hello = "TestDeBot12";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ Terminal.ID, SigningBoxInput.ID ];
    }

}