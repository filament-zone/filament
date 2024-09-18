// SPDX-License-Identifier: Apache-2.0.
pragma solidity ^0.8.25;

import {Math} from "@openzeppelin/contracts/utils/math/Math.sol";

import {EarlyExitPenalty} from "./EarlyExitPenalty.sol";

/**
 * This contract provides the number of unlocked tokens,
 *   and indicates if the grant has fully unlocked.
 */
abstract contract TimeLockedTokens {
    EarlyExitPenalty immutable penaltyManager;

    // The grant start time.
    // Grant can be deployed with startTime in the past or in the future.
    // The range of allowed past/future spread is defined in {CommonConstants}.
    // and validated in the constructor.
    uint256 public immutable startTime;

    // The grant end time.
    uint256 public immutable endTime;

    // The lockup time.
    uint256 public immutable lockupTime;

    // The amount of tokens in the locked grant.
    uint256 public immutable grantAmount;

    constructor(address penalty_, uint256 grantAmount_, uint256 startTime_, uint256 endTime_) {
        penaltyManager = EarlyExitPenalty(penalty_);
        grantAmount = grantAmount_;
        startTime = startTime_;
        endTime = endTime_;
        lockupTime = endTime - startTime;
    }

    /*
      Indicates whether the grant has fully unlocked.
    */
    function isGrantFullyUnlocked() public view returns (bool) {
        return block.timestamp >= endTime;
    }

    /// @notice Compute the early exit penalty for this grant at the current point
    ///         in time. This takes into account that there is no penalty for
    ///         unlocked tokens.
    function earlyExitPenalty(uint256 amount_) public view returns (uint256) {
        if (amount_ < this.unlockedTokens()) return 0;
        return penaltyManager.computePenalty(amount_ - this.unlockedTokens());
    }

    /*
      The number of locked tokens that were unlocked so far.
    */
    function unlockedTokens() public view returns (uint256) {
        if (block.timestamp <= startTime) return 0;

        uint256 cappedElapsedTime = Math.min(elapsedTime(), lockupTime);
        return (grantAmount * cappedElapsedTime) / lockupTime;
    }

    /*
      Returns the time passed (in seconds) since grant start time.
      Returns 0 if start time is in the future.
    */
    function elapsedTime() public view returns (uint256) {
        return block.timestamp > startTime ? block.timestamp - startTime : 0;
    }
}
