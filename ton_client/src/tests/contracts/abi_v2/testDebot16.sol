pragma ton-solidity >=0.51.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
import "https://raw.githubusercontent.com/tonlabs/debots/main/Debot.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/Base64/Base64.sol";

contract TestDebot16 is Debot {

    /// @notice Entry point function for DeBot.
    function start() public override {
        Base64.encode(tvm.functionId(decode), "12345");
    }


    function decode(string base64) public {
        Base64.decode(tvm.functionId(validate), base64);
    }

    function validate(bytes data) public {
        require(data == bytes("12345"));
    }

    function getDebotInfo() public functionID(0xDEB) override view returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon
    ) {
        name = "TestDeBot16";
        version = "0.1.0";
        publisher = "TON Labs";
        caption = "TestDeBot16";
        author = "TON Labs";
        support = address(0);
        hello = "TestDeBot16";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        
    }

}
