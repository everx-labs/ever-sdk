pragma ton-solidity >=0.45.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
import "https://raw.githubusercontent.com/tonlabs/debots/main/Debot.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/Terminal/Terminal.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/Sdk/Sdk.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/SigningBoxInput/SigningBoxInput.sol";

import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/libraries/JsonLib.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/Query/Query.sol";

contract TestDebot14 is Debot {

    using JsonLib for JsonLib.Value;
    using JsonLib for mapping(uint256 => TvmCell);

    uint32 m_limit;
    uint m_index;
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
    // Getter
    //

    function getRecipients() public returns (Recipient[] recipients) {
        recipients = m_recipients;
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
        optional(uint256) none;
        this.getRecipients{
            abiVer: 2,
            extMsg: true,
            sign: false,
            time: 0,
            expire: 0,
            pubkey: none,
            callbackId: tvm.functionId(onGetter),
            onErrorId: tvm.functionId(onError)
        }();
    }

    function onGetter(Recipient[] recipients) public {
        m_recipients = recipients;
        m_index = 0;
        m_limit = 3;
        Query.collection(
            tvm.functionId(setQueryResult), 
            QueryCollection.Messages, 
            format("{\"src\":{\"eq\":\"{}\"},\"msg_type\":{\"eq\":0}}", address(this)),
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
            mapping(uint256 => TvmCell) jsonObj;
            optional(JsonLib.Value) jsonv;
            for (JsonLib.Value obj: objects) {
                jsonObj = obj.as_object().get();
                jsonv = jsonObj.get("value");
                string balanceStr = jsonv.get().as_string().get();
                (uint balance, bool ok) = stoi(balanceStr);
                
                require(ok, 103);
                jsonv = jsonObj.get("dst");
                string dstStr = jsonv.get().as_string().get();
                address dst = stringToAddress(dstStr);

                require(m_recipients.length == 5, 300);
                Recipient rec = m_recipients[m_index];
                require(rec.dst == dst, 101);
                require(rec.val == uint128(balance), 102);

                m_index += 1;
            }
            
            jsonObj = objects[objects.length - 1].as_object().get();
            m_limit = 50;
            /*
            optional(JsonLib.Value) createdLT = jsonObj.get("created_lt");
            uint32 lastLT = uint32(createdLT.get().as_number().get());
            Query.collection(
                tvm.functionId(setQueryResult),
                QueryCollection.Messages,
                format("{\"src\":{\"eq\":\"{}\"},\"msg_type\":{\"eq\":0},\"created_lt\":{\"gt\":{}}}", address(this), lastLT),
                "created_lt value dst",
                m_limit,
                QueryOrderBy("created_lt", SortDirection.Ascending)
            );
            */
        } else {
            Terminal.print(0, "Done.");
        }
    }

    function stringToAddress(string str) private returns (address) {
        require(str.byteLength() >= 66, 201);
        optional(uint32) semicolon =  1; //str[1].find(byte(':'));
        require(semicolon.hasValue(), 202);
        string wcPart = str.substr(0, semicolon.get());
        string addrPart = str.substr(semicolon.get() + 1);
        (uint wc, bool ok) = stoi(wcPart);
        require(ok, 203);
        (uint addr, bool ok2) = stoi(format("0x{}", addrPart));
        require(ok2, 204);
        return address.makeAddrStd(int8(wc), addr);
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
