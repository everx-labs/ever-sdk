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

    struct Response {
        DataItem[] Result;
        string Status;
        string TestValue2;
        int32[] Numbers;
    }

    struct DataItem {
        string Provider;
        string Name;
        uint64 Number;
        string Special_Name;
        string Url;
        ProductData[] Product;
    }

    struct ProductData {
        string Currency;
        string MinValueStr;
        string MaxValueStr;
    }

    /// @notice Entry point function for DeBot.
    function start() public override {
        string json = "{\"name\":\"Joe\",\"tags\":[\"good\",\"bad\",\"ugly\"],\"age\":73,\"numbers\":[1,2,3],\"addrs\":{\"0:1111111111111111111111111111111111111111111111111111111111111111\":\"My main account\"}}";
        Json.deserialize(tvm.functionId(setResult), json);
        string json2 = "{\
\"Result\":[{\
    \"Provider\":\"PROVIDER\",\
    \"Name\":\"This is a name\",\
    \"Number\":123,\
    \"Special-Name\":\"Name with hyphen\",\
    \"Url\":\"https://this.is.url/logo/l.png\",\
    \"Product\":[{\
        \"Currency\":\"TON\",\
        \"MinValue\":2.00,\
        \"MinValueStr\":\"2.00\",\
        \"MaxValue\":461.00,\
        \"MaxValueStr\":\"461.00\"\
    }]\
}],\
\"Status\":\"success\",\
\"TestValue1\": 9.200000000,\
\"TestValue2\":\"9.300000000\",\
\"Numbers\":[1, 2, 3],\
\"Floats\":[1.1, 2.1, 3.1]\
}";
        Json.deserialize(tvm.functionId(setResult2), json2);
    }

    function assert_eq_s(string s1, string s2, uint code) private pure {
        require(tvm.hash(bytes(s1)) == tvm.hash(bytes(s2)), code);
    }

    function assert_eq_arr(int32[] a1, int32[] a2, uint code) private pure {
        require(a1.length == a2.length, code);
        for (uint i = 0; i < a1.length; i++) {
            require(a1[i] == a2[i], code);
        }
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

    function setResult2(bool result, Response obj) public pure {
        require(result == true, 99);
        assert_eq_s(obj.Status, "success", 101);
        assert_eq_s(obj.TestValue2, "9.300000000", 102);
        assert_eq_arr(obj.Numbers, [int32(1),2,3], 103);
        assert_eq_s(obj.Result[0].Provider, "PROVIDER", 104);
        assert_eq_s(obj.Result[0].Name, "This is a name", 105);
        assert_eq_s(obj.Result[0].Special_Name, "Name with hyphen", 109);
        require(obj.Result[0].Number == 123, 106);
        assert_eq_s(obj.Result[0].Product[0].MinValueStr, "2.00", 107);
        assert_eq_s(obj.Result[0].Product[0].MaxValueStr, "461.00", 108);
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ Terminal.ID, Json.ID ];
    }

    function getDebotInfo() public functionID(0xDEB) view override returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon) {
        name = "Test DeBot 7";
        version = "0.1.0";
        publisher = "TON Labs";
        caption = "Test for Json interface";
        author = "TON Labs";
        support = address(0);
        hello = "Test DeBot 7";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }


}