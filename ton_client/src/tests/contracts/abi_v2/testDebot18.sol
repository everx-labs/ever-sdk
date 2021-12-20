pragma ton-solidity >=0.40.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
import "https://raw.githubusercontent.com/tonlabs/debots/main/Debot.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/Query/Query.sol";

contract TestDebot18 is Debot {

    using JsonLib for JsonLib.Value;
    using JsonLib for mapping(uint256 => TvmCell);

    TvmCell m_sendMsg;

    function start() public override {
        Query.query(
            tvm.functionId(queryResult),
            "query($my_var: String!) { accounts(filter: {id: {eq: $my_var }}) {code_hash}}",
            format("{\"my_var\" : \"{}\"}",address(this))
        );
    }

    function queryResult(QueryStatus status, JsonLib.Value object) public {
        if(status == QueryStatus.Success) {
            mapping(uint256 => TvmCell) jsonObj;
            optional(JsonLib.Value) jsonv;

            jsonObj = object.as_object().get();
            jsonv = jsonObj.get("data");
            jsonObj = jsonv.get().as_object().get();
            jsonv = jsonObj.get("accounts");
            JsonLib.Cell[] array = jsonv.get().as_array().get();
            require(array.length == 1, 102);
            for (JsonLib.Cell e: array) {
                optional(JsonLib.Value) json = JsonLib.decodeArrayValue(e.cell);
                mapping(uint256 => TvmCell) obj = json.get().as_object().get();
                json = obj.get("code_hash");
                string val = json.get().as_string().get();
                string codeHash = format("{:064x}",tvm.hash(tvm.code()));
                require(val == codeHash, 101);
            }

            string str = format("{}",address(this));
            string query = "query { accounts(filter: {id: {eq:\"";
            query.append(str);
            query.append("\"}}) {code_hash}}");
            Query.query(
                tvm.functionId(queryResultWithoutVar),
                query,
                ""
            );
        } else {
            require(false, 102);
        }
    }

    function queryResultWithoutVar(QueryStatus status, JsonLib.Value object) public {
        if(status == QueryStatus.Success) {
            mapping(uint256 => TvmCell) jsonObj;
            optional(JsonLib.Value) jsonv;

            jsonObj = object.as_object().get();
            jsonv = jsonObj.get("data");
            jsonObj = jsonv.get().as_object().get();
            jsonv = jsonObj.get("accounts");
            JsonLib.Cell[] array = jsonv.get().as_array().get();
            require(array.length == 1, 102);
            for (JsonLib.Cell e: array) {
                optional(JsonLib.Value) json = JsonLib.decodeArrayValue(e.cell);
                mapping(uint256 => TvmCell) obj = json.get().as_object().get();
                json = obj.get("code_hash");
                string val = json.get().as_string().get();
                string codeHash = format("{:064x}",tvm.hash(tvm.code()));
                require(val == codeHash, 103);
            }

            string str = format("{}",address(this));
            string query = "query { accounts(filter: {id: {eq:\"";
            query.append(str);
            query.append("\"}}) {code_hash}}");
            Query.query(
                tvm.functionId(queryResultWithWrongVar),
                query,
                "abc"
            );
        } else {
            require(false, 104);
        }
    }

    function queryResultWithWrongVar(QueryStatus status, JsonLib.Value object) public {
        if(status != QueryStatus.VariablesError) {
            require(false, 105);
        }
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ Query.ID ];
    }

    function getDebotInfo() public functionID(0xDEB) view override returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon) {
        name = "TestDeBot18";
        version = "0.1.0";
        publisher = "TON Labs";
        caption = "TestDeBot18";
        author = "TON Labs";
        support = address(0);
        hello = "TestDeBot18";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }
}
