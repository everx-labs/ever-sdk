pragma ton-solidity >=0.47.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
import "https://raw.githubusercontent.com/tonlabs/debots/main/Debot.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/Terminal/Terminal.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/Json/Json.sol";

contract TestDebot15 is Debot {

    using JsonLib for JsonLib.Value;
    using JsonLib for mapping(uint256 => TvmCell);

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
        Json.parse(tvm.functionId(setValue), json);
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
        require(title == "My main account", 105);
    }

    function setValue(bool result, JsonLib.Value obj) public {
        require(result == true, 199);

        optional(JsonLib.Value) val;
        mapping(uint256 => TvmCell) jsonObj = obj.as_object().get();
        
        val = jsonObj.get("name");
        string name = val.get().as_string().get();
        require(name =="Joe",200);

        val = jsonObj.get("age");
        int age = val.get().as_number().get();
        require(age == 73, 202);

        val = jsonObj.get("addrs");
        mapping(uint256 => TvmCell) addrs = val.get().as_object().get();
        
        val = addrs.get("0:1111111111111111111111111111111111111111111111111111111111111111");
        string desc1 = val.get().as_string().get();
        require(desc1 == "My main account", 205);

        for ((uint256 hash, TvmCell cell): addrs) {
            optional(string) nameOpt;
            (val, nameOpt) = JsonLib.decodeObjectValue(cell);
            string desc = val.get().as_string().get();
            //Terminal.print(0, format("Address: {}, Description: {}", nameOpt.get(), desc));
        }
    }

    function getDebotInfo() public functionID(0xDEB) override view returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon
    ) {
        name = "TestDeBot15";
        version = "0.1.0";
        publisher = "TON Labs";
        caption = "TestDeBot15";
        author = "TON Labs";
        support = address(0);
        hello = "TestDeBot15";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ Terminal.ID, Json.ID ];
    }

}
