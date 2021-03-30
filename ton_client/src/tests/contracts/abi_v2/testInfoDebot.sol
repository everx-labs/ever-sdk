pragma ton-solidity >=0.35.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
// import required DeBot interfaces and basic DeBot contract.
import "../Debot.sol";
import "../Terminal.sol";

contract HelloDebot is Debot {

    /// @notice Entry point function for DeBot.
    function start() public override {
        // print string to user.
        Terminal.print(0, "Hello, World!");
        // input string from user and define callback that receives entered string.
        Terminal.input(tvm.functionId(setUserInput), "How is it going?", false);
    }

    function setUserInput(string value) public {
        // TODO: continue DeBot logic here...
        Terminal.print(0, format("You entered \"{}\"", value));
    }

    // @notice Define DeBot version and title here.
    function getVersion() public override returns (string name, uint24 semver) {
        (name, semver) = ("HelloWorld DeBot", _version(0,1,0));
    }

    function _version(uint24 major, uint24 minor, uint24 fix) private pure inline returns (uint24) {
        return (major << 16) | (minor << 8) | (fix);
    }

}