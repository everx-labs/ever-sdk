pragma ton-solidity >=0.40.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
import "../Debot.sol";
import "../Terminal.sol";
import "../Json.sol";

contract TestDebot7 is Debot {
    struct Info{
        string name;
        string[] tags;
        uint8 age;
        uint8[] numbers;
        mapping (address => string) addrs;
    }

    /// @notice Entry point function for DeBot.
    function start() public override {
        string json = "{\"name\":\"Joe\",\"tags\":[\"good\",\"bad\",\"ugly\"],\"age\":73,\"numbers\":[1,2,3],\"addrs\":{\"0:1111111111111111111111111111111111111111111111111111111111111111\":\"My main account\"}}";
        Json.deserialize(tvm.functionId(setResult), json);
    }

    function setResult(bool result, Info obj) public pure {
        uint8[] numbers = [1,2,3];
        string[] tags = ["good", "bad", "ugly"];
        require(result == true, 99);
        require(obj.name =="Joe", 100);
        for(uint i = 0; i < tags.length; i++) {
            require(tvm.hash(bytes(tags[i])) == tvm.hash(bytes(obj.tags[i])), (i << 4) & 1);
        }
	    require(obj.age == 73, 102);
        for(uint i = 0; i < numbers.length; i++) {
            require(obj.numbers[i]==numbers[i], 103);
        }
        address testAddr = address.makeAddrStd(0, 0x1111111111111111111111111111111111111111111111111111111111111111);
        optional(string) titleOpt = obj.addrs.fetch(testAddr);
        require(titleOpt.hasValue(), 104);
        string title = titleOpt.get();
        require(tvm.hash(bytes(title)) == tvm.hash(bytes("My main account")), 105);
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ Terminal.ID, Json.ID ];
    }

    function getDebotInfo() public functionID(0xDEB) view override returns(
        string name, string version, string publisher, string key, string author,
        address support, string hello, string language, string dabi, bytes icon) {
        name = "Test DeBot 7";
        version = "0.1.0";
        publisher = "TON Labs";
        key = "Test for Json interface";
        author = "TON Labs";
        support = address(0);
        hello = "Test DeBot 7";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }

    function getVersion() public override returns (string name, uint24 semver) {
        name = "Test DeBot 7";
        semver = 1 << 8;
    }

}