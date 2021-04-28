pragma ton-solidity >=0.40.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
import "../Debot.sol";
import "../Terminal.sol";
import "../Network.sol";
import "../Json.sol";

contract TestDebot8 is Debot {

    struct Response {
        Provider[] Result;
        string Status;
    }

    struct Provider {
        uint32 ProviderCode;
        string CountryIso;
        string Name;
        string LogoUrl;
    }

    struct Args {
        string name1;
    }
    struct Headers {
        string Accept;
        string Host;
        string X_Amzn_Trace_Id;
    }
    struct GetResponse {
        Args args;
        Headers headers;
        string origin;
        string url;
    }

    function start() public override {
        string[] headers;
        string url = "http://ptsv2.com/t/qajss-1618330246/post";
        headers.push("Content-Type: application/x-www-form-urlencoded");
        string body = "key1=value1";
        Network.post(tvm.functionId(setResponse), url, headers, body);
        url = "https://httpbin.org/get?name1=value1";
        string[] headers2;
        Network.get(tvm.functionId(setGetResponse), url, headers2);
    }

    function setResponse(int32 statusCode, string[] retHeaders, string content) public {
        retHeaders = retHeaders;
        require(statusCode == 200, 199);
        Json.deserialize(tvm.functionId(setResult), content);
    }

    function setGetResponse(int32 statusCode, string[] retHeaders, string content) public {
        retHeaders = retHeaders;
        require(statusCode == 200, 199);
        Json.deserialize(tvm.functionId(setResult2), content);
    }

    function setResult(bool result, Response obj) public pure {
        require(result == true, 200);
        require(tvm.hash(bytes(obj.Status)) == tvm.hash(bytes("success")), 201);
        require(obj.Result[0].ProviderCode == 678, 202);
        require(tvm.hash(bytes(obj.Result[0].Name)) == tvm.hash(bytes("Wonderland")), 203);
        require(tvm.hash(bytes(obj.Result[0].CountryIso)) == tvm.hash(bytes("WDL")), 204);
        require(tvm.hash(bytes(obj.Result[0].LogoUrl)) == tvm.hash(bytes("http://path.to.logo/url")), 205);
    }

    function setResult2(bool result, GetResponse obj) public pure {
        require(result == true, 300);
        require(tvm.hash(bytes(obj.args.name1)) == tvm.hash(bytes("value1")), 301);
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ Terminal.ID, Network.ID, Json.ID ];
    }

    function getDebotInfo() public functionID(0xDEB) view override returns(
        string name, string version, string publisher, string key, string author,
        address support, string hello, string language, string dabi, bytes icon) {
        name = "Test DeBot 8";
        version = "0.1.0";
        publisher = "TON Labs";
        string iface = "Network";
        key = format("Test for {} interface", iface);
        author = "TON Labs";
        support = address(0);
        hello = "Test DeBot 8";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }
}