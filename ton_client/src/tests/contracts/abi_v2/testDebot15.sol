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
        Json.parse(tvm.functionId(setValue), json);
    }

    function setValue(bool result, JsonLib.Value obj) public {
        require(result == true, 199);

        optional(JsonLib.Value) val;
        mapping(uint256 => TvmCell) jsonObj = obj.as_object().get();

        val = jsonObj.get("name");
        string name = val.get().as_string().get();
        require(name =="Joe",200);

        string[] expectedTags = ["good", "bad", "ugly"];
        uint i = 0;
        val = jsonObj.get("tags");
        JsonLib.Cell[] elems = val.get().as_array().get();
        for (JsonLib.Cell e: elems) {
            val = JsonLib.decodeArrayValue(e.cell);
            string tag = val.get().as_string().get();
            require(expectedTags[i] == tag, 300);
            i++;
        }

        val = jsonObj.get("age");
        int age = val.get().as_number().get();
        require(age == 73, 202);

        val = jsonObj.get("addrs");
        mapping(uint256 => TvmCell) addrs = val.get().as_object().get();

        val = addrs.get("0:1111111111111111111111111111111111111111111111111111111111111111");
        string desc1 = val.get().as_string().get();
        require(desc1 == "My main account", 205);

        mapping(string => string) expectedAddrs;
        expectedAddrs["0:1111111111111111111111111111111111111111111111111111111111111111"] = "My main account";
        for ((uint256 hash, TvmCell cell): addrs) {
            optional(string) nameOpt;
            (val, nameOpt) = JsonLib.decodeObjectValue(cell);
            string desc = val.get().as_string().get();
            require(expectedAddrs.exists(nameOpt.get()), 206);
            require(expectedAddrs[nameOpt.get()] == desc, 207);
        }

        string json = "{\"value1\":10,\"value2\":11.00,\"value3\":12.01}";
        Json.parse(tvm.functionId(setNumberConvertion), json);
    }

    function setNumberConvertion(bool result, JsonLib.Value obj) public {
        require(result == true, 210);

        optional(JsonLib.Value) val;
        mapping(uint256 => TvmCell) jsonObj = obj.as_object().get();

        val = jsonObj.get("value1");
        int value1 = val.get().as_number().get();
        require(value1 == 10,211);

        val = jsonObj.get("value2");
        string value2 = val.get().as_string().get();
        require(value2 == "11.0", 212);

        val = jsonObj.get("value3");
        string value3 = val.get().as_string().get();
        require(value3 == "12.01", 213);
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
