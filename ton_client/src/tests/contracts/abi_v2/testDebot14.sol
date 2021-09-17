pragma ton-solidity >=0.45.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
import "https://raw.githubusercontent.com/tonlabs/debots/main/Debot.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/Terminal/Terminal.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/Sdk/Sdk.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/SigningBoxInput/SigningBoxInput.sol";

import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/query/libraries/JsonLib.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/query/Query/Query.sol";

contract TestDebot14 is Debot {

    using JsonLib for JsonLib.Value;
    using JsonLib for mapping(uint256 => JsonLib.Value);

    uint32 m_limit;
    struct Recipient {
        address dst;
        uint128 val;
    }
    
    Recipient[] m_recipients;

    //
    // Onchain functions
    //

    function scatter(uint8 count) public {
        tvm.accept();
        rnd.shuffle();
        uint128 val = 0.1 ton;
        address dst;
        for (uint8 i = 0; i < count - 2; i++) {
            dst = address(rnd.next());
            dst.transfer(val, false, 3);
            m_recipients.push(Recipient(dst, val));
            val += 0.1 ton;

        }

        dst = address(this); val = 0.15 ton;
        dst.transfer(val, true, 3);
        m_recipients.push(Recipient(dst, val));

        val = 0.25 ton;
        dst.transfer(val, true, 3);
        m_recipients.push(Recipient(dst, val));
    }
    //
    // DeBot functions
    //


    function start() public override {
        SigningBoxInput.get(tvm.functionId(setBoxHandle), "", [tvm.pubkey()]);
    }
    function setBoxHandle(uint32 handle) public {
        
        this.scatter{
            abiVer: 2,
            extMsg: true,
            sign: true,
            time: 0,
            expire: 0,
            pubkey: 0,
            callbackId: tvm.functionId(onSuccess),
            onErrorId: tvm.functionId(onError),
            signBoxHandle: handle
        }(5);

    }

    function onError(uint32 sdkError, uint32 exitCode) public {
        revert(200);
    }

    function onSuccess() public {
        m_limit = 3;
        Query.collection(
            tvm.functionId(setQueryResult), 
            QueryCollection.Messages, 
            format("{ src: { eq: \"{}\" } msg_type: { eq: 0 } }", address(this)),
            "created_lt value dst",
            m_limit,
            QueryOrderBy("created_lt", SortDirection.Ascending)
        );
    }

    function setQueryResult(QueryStatus status, JsonLib.Value[] objects) public {
        if (status != QueryStatus.Success) {
            Terminal.print(tvm.functionId(Debot.start), "Messages query failed.");
            return;
        }

        if (objects.length != 0) {
            mapping(uint256 => JsonLib.Value) jsonObj;
            for (JsonLib.Value obj: objects) {
                jsonObj = obj.as_object().get();
                optional(JsonLib.Value) balance = jsonObj.get("value");
                optional(JsonLib.Value) dst = jsonObj.get("dst");
                Terminal.print(0, format(
                    "Sent {:t} tons to address {}", 
                    balance.get().as_number().get(), 
                    dst.get().as_string().get()
                ));
            }
            
            jsonObj = objects[objects.length - 1].as_object().get();
            m_limit = 50;
            optional(JsonLib.Value) createdLT = jsonObj.get("created_lt");
            uint32 lastLT = uint32(createdLT.get().as_number().get());
            Query.collection(
                tvm.functionId(setQueryResult),
                QueryCollection.Messages,
                format("{ src: { eq: \"{}\" } msg_type: { eq: 0 } created_lt: {gt: {} } }", address(this), lastLT),
                "created_lt value dst",
                m_limit,
                QueryOrderBy("created_lt", SortDirection.Ascending)
            );
        } else {
            Terminal.print(0, "Done.");
        }
    }

    function getDebotInfo() public functionID(0xDEB) override view returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon
    ) {
        name = "TestDeBot14";
        version = "0.1.0";
        publisher = "TON Labs";
        caption = "TestDeBot14";
        author = "TON Labs";
        support = address(0);
        hello = "TestDeBot14";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ Terminal.ID, Query.ID ];
    }

}
