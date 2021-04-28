pragma ton-solidity >=0.40.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
import "../Debot.sol";
import "../Sdk.sol";
import "../Base64.sol";
import "../Hex.sol";
import "../Terminal.sol";
import "../SigningBoxInput.sol";

contract testDebot3 is Debot {

    /*
        Storage
    */

    uint32 m_rnd1;
    uint32 m_rnd2;

     /*
     *  Overrided Debot functions
     */

     // Entry point for new debots
    function start() override public {
        runAll();
    }

    /// @notice Returns Metadata about DeBot.
    function getDebotInfo() public functionID(0xDEB) override view returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon
    ) {
        name = "TestSdk";
        version = "0.4.0";
        publisher = "TON Labs";
        caption = "Test for SDK interface";
        author = "TON Labs";
        support = address(0);
        hello = "Hello, I'm a test.";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ Terminal.ID ];
    }

    function runAll() public {
        testMnemonic(0);
        testString(0);
        testMnemonicDeriveSignKeys(0);
        testhdkeyXprv(0);
        testAccount();
        testRandom();
        testNaclbox();
        testKeysFromSecret();
        testHexEncode();
        testBase64Encode();
        testSign();
    }

    function testhdkeyXprv(uint32 index) public {
        index = index;
        string ph = "blood genius security pen scissors tissue coil wish devote silk minimum remind";
        Sdk.hdkeyXprvFromMnemonic(tvm.functionId(hdkeyXprvFromMnemonicRes),ph);
    }

    function hdkeyXprvFromMnemonicRes(string xprv) public{
        require(xprv=="xprv9s21ZrQH143K4bDg2b9x7Y81Qw5LcfJ6D6YStM2rZVZP1DcYvmadfUXmV8wFb77Jp6s88VykAvdFAey3b5B1ykToXVJo5nnG26VVupdDxS3",130);
        Sdk.hdkeyDeriveFromXprv(tvm.functionId(hdkeyDeriveFromXprvRes),xprv,1,true);
    }

     function hdkeyDeriveFromXprvRes(string xprv) public{
        require(xprv=="xprv9vDCWeesvU6UAq8cHqF3Pxyrju8oWRw9czjRrGbHpRWhUrp36MxX2WbGKuQdtKB24Z7bsThjfexsnWFwpjU81WKhGE9KuT7RJmJ2yDb6TeJ",131);
        string path = "m/44'/396'/0'/0/1";
        Sdk.hdkeyDeriveFromXprvPath(tvm.functionId(hdkeyDeriveFromXprvPathRes),xprv,path);
    }

    function hdkeyDeriveFromXprvPathRes(string xprv) public{
        require(xprv=="xprvA5Hr4SR7z6JmHGYJmDJefc8PETc7JSEV7iQGmr1AxycsddzVvHeZPpomy4ygLDiUbUu7322yTba9JxomxPM3TNH4TVHx6ZDysE6WX3X5Ym9",132);
        Sdk.hdkeySecretFromXprv(tvm.functionId(hdkeySecretFromXprvRes),xprv);
    }

    function hdkeySecretFromXprvRes(uint256 sec) public{
        require(sec==0xa2087ef167360868a5153cd84182221388a79a26fe5a8557da0430b354dcc096,133);
        Sdk.naclSignKeypairFromSecretKey(tvm.functionId(naclSignKeypairFromSecretKeyRes),sec);

    }
    function naclSignKeypairFromSecretKeyRes(uint256 sec, uint256 pub) public{
       require(pub==0x33d6e80fb461e6b1f7592970112a13f398428dcecc18431fead5ade906bce304,134);
       require(sec==0xa2087ef167360868a5153cd84182221388a79a26fe5a8557da0430b354dcc096,135);
       Terminal.print(0, "test hdkeyXprv passed");
    }

    function testMnemonicDeriveSignKeys(uint32 index) public {
        index = index;
        string ph = "blood genius security pen scissors tissue coil wish devote silk minimum remind";
        string path = "m/44'/396'/0'/0/1";
        Sdk.mnemonicDeriveSignKeys(tvm.functionId(checkMnemonicDeriveSignKeys),ph,path);
    }

    function checkMnemonicDeriveSignKeys(uint256 pub, uint256 sec) public {
        require(pub == 0x07c31538a8371ced8ec6bd37c1b6dde86cf7246495e5ce538d58256c4f73dc5f,128);
        require(sec == 0xbc6464a6003bcb94659a03a1e705dd3d857bd270fb35acfd9bd460136e33ae39,129);
        Terminal.print(0, "test mnemonicDeriveSignKeys passed");
    }


    function testMnemonic(uint32 index) public {
        index = index;
        Sdk.mnemonicFromRandom(tvm.functionId(genMnemonic),1,12);
    }

    function genMnemonic(string phrase) public {
        Sdk.mnemonicVerify(tvm.functionId(verifyPhrase),phrase);
    }

    function verifyPhrase(bool valid) public {
        require(valid, 125);
        Terminal.print(0, "test mnemonic passed");
    }

    function testString(uint32 index) public {
        index = index;
        Sdk.substring(tvm.functionId(testCut1),"one two three",0,3);
        Sdk.substring(tvm.functionId(testCut2),"one two three",4,3);
    }

    function testCut1(string substr) public {
        require(substr=="one", 126);
        Terminal.print(0, "test substring1 passed");
    }

    function testCut2(string substr) public {
        require(substr=="two", 127);
        Terminal.print(0, "test substring2 passed");
    }

    function testAccount() public {
        Sdk.getBalance(tvm.functionId(setBalance), address(this));
        Sdk.getBalance(tvm.functionId(setBalance2), address.makeAddrStd(0, 1));
    }

    function setBalance(uint128 nanotokens) public {
        require(nanotokens > 0, 130);
        Sdk.getAccountType(tvm.functionId(setAccountType), address(this));
        Sdk.getAccountType(tvm.functionId(setAccountType2), address.makeAddrStd(0, 1));
    }

    function setBalance2(uint128 nanotokens) public pure {
        require(nanotokens == 0, 131);
    }

    function setAccountType(int8 acc_type) public {
        require(acc_type == 1, 132);
        Sdk.getAccountCodeHash(tvm.functionId(setCodeHash), address(this));
        Sdk.getAccountCodeHash(tvm.functionId(setCodeHash2), address.makeAddrStd(0, 1));
    }

    function setAccountType2(int8 acc_type) public pure {
        require(acc_type == -1, 133);
    }

    function setCodeHash(uint256 code_hash) public {
        require(code_hash != 0, 134);
        Terminal.print(0, "test account passed");
    }

    function setCodeHash2(uint256 code_hash) public pure {
        require(code_hash == 0, 135);
    }

    function testRandom() public {
        Sdk.genRandom(tvm.functionId(setRandom1), 32);
        Sdk.genRandom(tvm.functionId(setRandom2), 32);
    }

    function setRandom1(bytes buffer) public {
        m_rnd1 = buffer.toSlice().decode(uint32);
    }

    function setRandom2(bytes buffer) public {
        m_rnd2 = buffer.toSlice().decode(uint32);
        require(m_rnd1 != m_rnd2, 140);
        Terminal.print(0, "test genRandom passed");
    }

    function testNaclbox() public {
     /*
    const pub1 = "47F7C36DC0896FD2C020BB21F1605A3C161BF4EC5C5A23E5B5A111288CE59F19";
    const sec1 = "4DD67EBD0431CEC53C60499775423695C4407A2CFABB6E7569046C1E7445A89C";

    const pub2 = "552bc62d6e65294c0dea6c9143163dfcca6e1da4ea8411241e949925d07a0204";
    const sec2 = "6817601a9b60fc4c449d3dc72e30600bddc74eeb4ad00e8d4d223f4b6dd94f3c";
    */
        string dec = "hello";
        bytes nonce = bytes("000000000000000000000001");
        uint256 tp = 0x552bc62d6e65294c0dea6c9143163dfcca6e1da4ea8411241e949925d07a0204;
        uint256 s = 0x4DD67EBD0431CEC53C60499775423695C4407A2CFABB6E7569046C1E7445A89C;
        Sdk.naclBox(tvm.functionId(getEnc),dec,nonce,tp,s);
    }

    function getEnc(bytes encrypted)public {
        require(encrypted.length==21,151);
        Terminal.print(0,"test naclbox passed");
        bytes nonce = bytes("000000000000000000000001");
        uint256 tp = 0x47F7C36DC0896FD2C020BB21F1605A3C161BF4EC5C5A23E5B5A111288CE59F19;
        uint256 s = 0x6817601a9b60fc4c449d3dc72e30600bddc74eeb4ad00e8d4d223f4b6dd94f3c;
        Sdk.naclBoxOpen(tvm.functionId(getDec),encrypted,nonce,tp,s);
    }

    function getDec(bytes decrypted)public {
        require(string(decrypted)=="hello",152);
        Terminal.print(0,"test naclboxopen passed");
    }

    function testKeysFromSecret() public {
        uint256 sec = 0x4dd67ebd0431cec53c60499775423695c4407a2cfabb6e7569046c1e7445a89c;
        Sdk.naclKeypairFromSecret(tvm.functionId(getPair),sec);
    }

    function getPair(uint256 publicKey, uint256 secretKey)public {
        require(publicKey==0x47f7c36dc0896fd2c020bb21f1605a3c161bf4ec5c5a23e5b5a111288ce59f19,153);
        require(secretKey==0x4dd67ebd0431cec53c60499775423695c4407a2cfabb6e7569046c1e7445a89c,154);
        Terminal.print(0,"test naclKeypairFromSecret passed");
    }

    function testHexEncode() public {
        bytes data = bytes("hello");
        Hex.encode(tvm.functionId(hexEnc),data);
    }
    function hexEnc(string hexstr) public {
        require(hexstr=="68656c6c6f",130);
        Terminal.print(tvm.functionId(testHexDecode),"test hex encode passed");
    }
    function testHexDecode() public {
        string hexstr = "68656c6c6f";
        Hex.decode(tvm.functionId(hexDec),hexstr);
    }
    function hexDec(bytes data) public {
        require(string(data)=="hello",131);
        Terminal.print(0,"test hex decode passed");
    }

    function testBase64Encode() public {
        bytes data = bytes("hello");
        Base64.encode(tvm.functionId(base64Enc),data);
    }
    function base64Enc(string base64) public {
        require(base64=="aGVsbG8=",132);
        Terminal.print(tvm.functionId(testBase64Decode),"test base64 encode passed");
    }
    function testBase64Decode() public {
        string base64 = "aGVsbG8=";
        Base64.decode(tvm.functionId(base64Dec),base64);
    }
    function base64Dec(bytes data) public {
        require(string(data)=="hello",133);
        Terminal.print(0,"test base64 decode passed");
    }

    //
    // Sign functions
    //

    function testSign() public {
        uint256[] possibleKeys;
        SigningBoxInput.get(tvm.functionId(setSigningBox), "Enter key:", possibleKeys);
    }

    function setSigningBox(uint32 handle) public {
        uint256 hash = sha256("test sign string");
        Sdk.signHash(tvm.functionId(setSignature), handle, hash);
    }

    function setSignature(bytes signature) public {
        require(signature.length == 64, 200);
        Terminal.print(0,"test sign hash passed");
        uint256 hash = sha256("test sign string");
        require(tvm.checkSign(hash, signature.toSlice(), tvm.pubkey()), 201);
    }
}
