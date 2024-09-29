// SPDX-License-Identifier: Apache-2.0.
pragma solidity ^0.8.25;

error Unauthorized();
error InvalidEndTime();
error InvalidAdjustor();
error PenaltyTooLow();
error PenaltyTooHigh();

/// @notice This contract computes the exit penalty which gets applied to grants.
///         The penalty decreases linearly over `penaltyTime` from `penalty` to
///         zero.
contract EarlyExitPenalty {
    /// @dev Address authorized to adjust penalty
    address public adjustor;

    /// @dev Penalty in bps
    uint16 public penalty = 1000;

    /// @dev Maximum permissible penalty in bps
    uint16 public immutable maxPenalty = 10000;

    /// @dev Minimum permissible penalty in bps
    uint16 public immutable minPenalty = 0;

    /// @dev Time for penalty to drop off
    uint256 public penaltyTime;

    /// @dev Start time for the penalty to drop off
    uint256 public immutable startTime;

    // @dev The time by which the penalty will be zero
    uint256 public endTime;

    event PenaltyChanged(uint16 oldPenalty, uint16 newPenalty);

    event EndTimeChanged(uint256 oldEndTime, uint256 newEndTime);

    event AdjustorChanged(address oldAdjustor, address newAdjustor);

    constructor(
        address adjustor_,
        uint16 penalty_,
        uint16 maxPenalty_,
        uint16 minPenalty_,
        uint256 startTime_,
        uint256 endTime_
    ) {
        adjustor = adjustor_;
        penalty = penalty_;
        maxPenalty = maxPenalty_;
        minPenalty = minPenalty_;

        penaltyTime = endTime_ - startTime_;
        startTime = startTime_;
        endTime = endTime_;
    }

    function elapsedTime() public view returns (uint256) {
        if (block.timestamp >= endTime) return penaltyTime;
        return block.timestamp > startTime ? block.timestamp - startTime : 0;
    }

    /// @dev Compute the penalty for a given amount.
    ///
    /// @param amt_ amount to compute penalty for
    function computePenalty(uint256 amt_) external view returns (uint256) {
        // Risk overflow rather than no penalty
        uint256 elapsed_ = elapsedTime();
        uint256 penalty_ = penalty - (penalty * elapsed_) / penaltyTime;
        return amt_ * uint256(penalty_) / 10000;
    }

    // Permissioned functions

    /// @notice Set the early exit penalty.
    ///
    /// @param newPenalty_ new penalty in bps
    function setPenalty(uint16 newPenalty_) external {
        if (msg.sender != adjustor) revert Unauthorized();
        if (newPenalty_ < minPenalty) revert PenaltyTooLow();
        if (newPenalty_ > maxPenalty) revert PenaltyTooHigh();

        uint16 old = penalty;
        penalty = newPenalty_;

        emit PenaltyChanged(old, newPenalty_);
    }

    /// XXX: set endtime
    function setEndTime(uint256 newEndTime_) external {
        if (msg.sender != adjustor) revert Unauthorized();
        if (newEndTime_ <= startTime) revert InvalidEndTime();

        uint256 old = endTime;
        endTime = newEndTime_;
        penaltyTime = endTime - startTime;

        emit EndTimeChanged(old, newEndTime_);
    }

    /// @notice Set the address which can modify the early exit penalty.
    ///
    /// @param newAdjustor_ newly authorized address to change penalty
    function setAdjustor(address newAdjustor_) external {
        if (msg.sender != adjustor) revert Unauthorized();
        if (newAdjustor_ == address(0)) revert InvalidAdjustor();

        address old = adjustor;
        adjustor = newAdjustor_;

        emit AdjustorChanged(old, newAdjustor_);
    }
}
