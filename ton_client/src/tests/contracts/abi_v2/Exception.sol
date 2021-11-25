pragma ton-solidity >=0.35.0;

contract Exception {
    function fail() public {
        require(false, 1111, "This is long error message (just for testing purposes). If you see this error, you can be sure that this contract works as expected.");
    }

    function failAfterAccept() public {
        tvm.accept();
        require(false, 1111, "This is long error message (just for testing purposes). If you see this error, you can be sure that this contract works as expected.");
    }
}