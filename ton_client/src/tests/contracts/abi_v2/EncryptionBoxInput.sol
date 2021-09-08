pragma ton-solidity >= 0.45.0;

interface IEncryptionBoxInput {

    function getNaclBox(uint32 answerId, string prompt, bytes nonce, uint256 theirPubkey) external returns (uint32 handle);
    function getNaclSecretBox(uint32 answerId, string prompt, bytes nonce) external returns (uint32 handle);
    function getChaCha20Box(uint32 answerId, string prompt, bytes nonce) external returns (uint32 handle);
    function remove(uint32 answerId, uint32 handle) external returns (bool removed);
    function getSupportedAlgorithms(uint32 answerId) external returns (string[] names);
}

library EncryptionBoxInput {

	uint256 constant ID = 0x5b5f76b54d976d72f1ada3063d1af2e5352edaf1ba86b3b311170d4d81056d61;
	int8 constant DEBOT_WC = -31;

	function getNaclBox(uint32 answerId, string prompt, bytes nonce, uint256 theirPubkey) public pure {
		address addr = address.makeAddrStd(DEBOT_WC, ID);
		IEncryptionBoxInput(addr).getNaclBox(answerId, prompt, nonce, theirPubkey);
	}

    function getNaclSecretBox(uint32 answerId, string prompt, bytes nonce) public pure {
		address addr = address.makeAddrStd(DEBOT_WC, ID);
		IEncryptionBoxInput(addr).getNaclSecretBox(answerId, prompt, nonce);
	}

    function getChaCha20Box(uint32 answerId, string prompt, bytes nonce) public pure {
		address addr = address.makeAddrStd(DEBOT_WC, ID);
		IEncryptionBoxInput(addr).getChaCha20Box(answerId, prompt, nonce);
	}

    function getSupportedAlgorithms(uint32 answerId) public pure {
		address addr = address.makeAddrStd(DEBOT_WC, ID);
		IEncryptionBoxInput(addr).getSupportedAlgorithms(answerId);
	}

    function remove(uint32 answerId, uint32 handle) public pure {
        address addr = address.makeAddrStd(DEBOT_WC, ID);
		IEncryptionBoxInput(addr).remove(answerId, handle);
    }
}

contract EncryptionBoxInputABI is IEncryptionBoxInput {

    function getNaclBox(uint32 answerId, string prompt, bytes nonce, uint256 theirPubkey) external override returns (uint32 handle) {}
    function getNaclSecretBox(uint32 answerId, string prompt, bytes nonce) external override returns (uint32 handle) {}
    function getChaCha20Box(uint32 answerId, string prompt, bytes nonce) external override returns (uint32 handle) {}
    function remove(uint32 answerId, uint32 handle) external override returns (bool removed) {}
    function getSupportedAlgorithms(uint32 answerId) external override returns (string[] names) {}

}
