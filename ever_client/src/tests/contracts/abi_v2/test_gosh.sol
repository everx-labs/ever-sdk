pragma ton-solidity >=0.5.0;

contract TestAddress {
	
	modifier alwaysAccept {
		tvm.accept(); _;
	}
	
	function f() public pure alwaysAccept {
		string textA;
		string textB;
		string diff = gosh.diff(textA, textB);
		{
			string newTextB = gosh.applyPatch(textA, diff);
			require(newTextB == textB);
		}
		{
			optional(string) newTextB = gosh.applyPatchQ(textA, diff);
			require(newTextB.get() == textB);
		}
	}

	function fZip() public pure alwaysAccept {
		string textA = "12edsegfr";
		bytes cell = gosh.zip(textA);
		string s = gosh.unzip(cell);
		require(s == textA);
	}

	function f2() public pure alwaysAccept {
		bytes textA = gosh.zip("lalalala");
		bytes textB = gosh.zip("lalalala\nlalalala\n");
		bytes diff = gosh.zipDiff(textA, textB);
		{
			bytes newTextB = gosh.applyZipPatch(textA, diff);
			require(newTextB == textB);
		}
		{
			optional(bytes) newTextB = gosh.applyZipPatchQ(textA, diff);
			require(newTextB.get() == textB);
		}
	}

	function ff() public pure alwaysAccept returns (uint64) {
		return tx.storageFee;
	}

	function fff() public pure alwaysAccept {
		bytes textA;
		bytes textB;
		bytes patch;
		optional(bytes) optTextB;

		textB = gosh.applyBinPatch(textA, patch);
		optTextB = gosh.applyBinPatchQ(textA, patch);

		textB = gosh.applyZipBinPatch(textA, patch);
		optTextB = gosh.applyZipBinPatchQ(textA, patch);
	}
}
