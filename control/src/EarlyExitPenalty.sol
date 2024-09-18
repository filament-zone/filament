// SPDX-License-Identifier: Apache-2.0.
pragma solidity ^0.8.25;

error Unauthorized();
error InvalidAdjustor();
error PenaltyTooLow();
error PenaltyTooHigh();

contract EarlyExitPenalty {
    /// @dev Address authorized to adjust penalty
    address public adjustor;

    /// @dev Penalty in bps
    uint16 public penalty = 1000;

    /// @dev Maximum permissible penalty in bps
    uint16 public immutable maxPenalty = 2000;

    /// @dev Minimum permissible penalty in bps
    uint16 public immutable minPenalty = 0;

    event PenaltyChanged(uint16 oldPenalty, uint16 newPenalty);

    event AdjustorChanged(address oldAdjustor, address newAdjustor);

    constructor(address adjustor_, uint16 penalty_, uint16 maxPenalty_, uint16 minPenalty_) {
        adjustor = adjustor_;
        penalty = penalty_;
        maxPenalty = maxPenalty_;
        minPenalty = minPenalty_;
    }

    /// @dev Compute the penalty for a given amount.
    ///
    /// @param amt_ amount to compute penalty for
    function computePenalty(uint256 amt_) external view returns (uint256) {
        // Risk overflow rather than no penalty
        return amt_ * uint256(penalty) / 10000;
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
