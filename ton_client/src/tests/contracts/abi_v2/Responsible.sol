pragma ton-solidity >=0.58.0;

contract Responsible {
    function sum(uint32 a, uint32 b) pure external responsible returns (uint32) {
        return a + b;
    }
}
