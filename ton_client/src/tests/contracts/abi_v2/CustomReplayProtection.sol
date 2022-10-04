pragma ton-solidity ^0.65.0;
pragma AbiHeader time;
pragma AbiHeader pubkey;
pragma AbiHeader expire;

import "testDebot20.sol";

contract RollingIdsTest
 {
    uint64 m_timestamp;
    uint32 m_expire;

    function answer(address dst) public view externalMsg {
        tvm.accept();
        ARecieverDebot(dst).headerCalback(m_timestamp,m_expire);
    }

    function afterSignatureCheck(TvmSlice body, TvmCell message) private inline returns (TvmSlice) {
        //msgId = timestamp
        message;
        (uint64 msgId,uint32 exp) = body.decode(uint64, uint32);
        require(msgId>m_timestamp,302);
        m_timestamp = msgId;
        m_expire = exp;
        return body;
    }

    function getData() external view returns (uint64,uint32) {
        return (m_timestamp,m_expire);
    }
}
