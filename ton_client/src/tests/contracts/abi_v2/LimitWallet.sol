pragma ton-solidity >= 0.38.0;
pragma AbiHeader expire;
import "./SubscrWallet.sol";

contract LimitWallet is SubscrWallet {
    //There are 2 kind of m_limits: single operation limit and periodic limit.
    //Single operation limit has period = 0.
    struct Limit {
        //limit value in nanograms
        uint256 value;
        //limit period in days, can be 0 for single operation limit
        uint32 period;
        //single operation or arbitrary
        uint8 ltype;
        //sum of spent nanograms in limit period
        uint256 spent;
        //start of limit period in seconds
        uint32 start;
    }

    uint8 MAX_LIMIT_COUNT = 5;
    uint32 SECONDS_IN_DAY = 86400;
    uint32 MAX_LIMIT_PERIOD = 365;

    mapping (uint64 => Limit) m_limits;
    uint8 m_limitCount = 0;
    uint64 m_limitseqNo = 0;

    /*
     Exception codes:
      100 - message sender is not a wallet owner.
      101 - limit is overrun.
      102 - invalid transfer value.
      104 - invalid limit period.
      105 - maximum number of m_limits is reached.
     */

    modifier OnlyOwner {
        require(msg.pubkey() == tvm.pubkey(), 100);
        _;
    }

    /// @dev Allows to transfer grams to destination account.
    /// @param dest Transfer target address.
    /// @param value Nanograms value to transfer.
    /// @param bounce Flag that enables bounce message in case of target contract error.
    function sendTransaction(address dest, uint128 value, bool bounce)  public  override  OnlyOwner  {
        require(_checkLimits(value) == false, 101);
        dest.transfer(value, bounce, 3);
    }


    /*
    m_limits API
    */

    /// @dev Allow to create single operation limit.
    /// @param value Limit value in nanograms.
    /// @return Limit id.
    function createOperationLimit(uint256 value) public OnlyOwner returns (uint256) {
        _validate(value, 0, 0);
        tvm.accept();
        return _newLimit(value, 0, 0);
    }

    /// @dev Allow to create limit for defined period.
    /// @param value Limit value in nanograms.
    /// @param period Period in days.
    /// @return Limit id.
    function createArbitraryLimit(uint256 value, uint32 period) public OnlyOwner returns (uint64) {
        _validate(value, period, 1);
        tvm.accept();
        return _newLimit(value, period, 1);
    }

    /// @dev Allow to request for changes in limit parameters.
    /// @param limitId limit id.
    /// @param value Limit value in nanograms.
    /// @param period Period in days.
    function changeLimit(uint64 limitId, uint256 value, uint32 period) public OnlyOwner {
        optional(Limit) m_limit = m_limits.fetch(limitId);
        require(m_limit.hasValue(), 103);
        (Limit lim) = m_limit.get();
        _validate(value, period, lim.ltype);
        tvm.accept();
        lim.value = value;
        lim.period = period;
        m_limits[limitId] = lim;
    }

    /// @dev Allow to delete limit
    /// @param limitId Limit id
    function deleteLimit(uint64 limitId) public OnlyOwner {
        tvm.accept();
        optional(Limit) m_limit = m_limits.fetch(limitId);
        if (m_limit.hasValue()) {
            delete m_limits[limitId];
            m_limitCount -= 1;
        }
    }

    /*
     Get methods
    */

    function getLimit(uint64 limitId) public view returns (Limit) {
        return m_limits[limitId];
    }

    function getLimitCount() public view returns (uint64) {
        return m_limitCount;
    }

    /// @dev Allow to query all limits.
    /// @return limits - array of limits.
    function getLimits() public view returns (Limit[] limits) {
        mapping(uint64 => Limit) tmpLimits = m_limits;
        optional (uint64, Limit) tmpLimit = tmpLimits.min();
        while(tmpLimit.hasValue()) {
            (uint64 id, Limit lim) = tmpLimit.get();
            limits.push(lim);
            tmpLimit = tmpLimits.next(id);
        }
    }

    /*
     Internal methods
    */

    /// @dev Checks that all of m_limits allow transfering defined value.
    /// @param value Value of nanograms in transaction.
    /// @return true if limit is overrun.
    /// if returns 0 then some of the m_limits is exceded.
    function _checkLimits(uint256 value) private inline returns (bool) {
        bool overrun = false;
        optional(uint64, Limit) m_limit = m_limits.min();
        while(m_limit.hasValue()) {
            (uint64 id, Limit limit) = m_limit.get();
            uint32 endTime = limit.start + limit.period * SECONDS_IN_DAY;
            uint32 nowTime = uint32(now);
            if (nowTime > endTime) {
                //reset period
                limit.start = nowTime;
                limit.spent = 0;
            }

            limit.spent += value;
            if (limit.spent > limit.value) {
                //abort transaction cause limit is overrun
                overrun = true;
                break;
            }
            m_limits[id] = limit;
            m_limit = m_limits.next(id);
        }
        return overrun;
    }
   
    function _validate(uint256 value, uint32 period, uint8 ltype) private view inline {
        require(value > 0, 102);
        if (ltype == 1) {
            require(period > 0 && period <= MAX_LIMIT_PERIOD, 104);
        } else {
            require(period == 0, 104);
        }
        require(m_limitCount < MAX_LIMIT_COUNT, 105);
    }

    function _newLimit(uint256 value, uint32 period, uint8 ltype) internal returns (uint64) {
        uint64 limitId = m_limitseqNo;
        m_limitseqNo += 1;
        m_limits[limitId] = Limit(value, period, ltype, 0, 0);
        m_limitCount += 1;
        return limitId;
    }

    receive() external override {}
}