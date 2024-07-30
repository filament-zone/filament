// SPDX-License-Identifier: Apache-2.0.
pragma solidity ^0.8.25;

error Unauthorized();
error PenaltyTooLow();
error PenaltyTooHigh();

contract EarlyExitPenalty {
    /// @dev Address authorized to adjust penalty
    address public adjustor;

    /// @dev Penalty in bips
    uint16 public penalty = 1000;

    /// @dev Maximum permissible penalty
    uint16 public immutable maxPenalty = 20000;

    /// @dev Minimum permissible penalty
    uint16 public immutable minPenalty = 0;

    event PenaltyChanged(uint16 oldPenalty, uint16 newPenalty);

    event AdjustorChanged(address oldAdjustor, address newAdjustor);

    constructor(address adjustor_, uint16 penalty_, uint16 maxPenalty_, uint16 minPenalty_) {
        adjustor = adjustor_;
        penalty = penalty_;
        maxPenalty = maxPenalty_;
        minPenalty = minPenalty_;
    }

    function computePenalty(uint256 amt_) external view returns (uint256) {
        return amt_ / 10000 * uint256(penalty);
    }

    // Permissioned functions

    /// @notice Set the early exit penalty.
    ///
    /// @param newPenalty new penalty in bips
    function setPenalty(uint16 newPenalty) external {
        if (msg.sender != adjustor) revert Unauthorized();
        if (newPenalty < minPenalty) revert PenaltyTooLow();
        if (newPenalty > maxPenalty) revert PenaltyTooHigh();

        uint16 old = penalty;
        penalty = newPenalty;
        emit PenaltyChanged(old, newPenalty);
    }

    /// @notice Set the address which can modify the early exit penalty.
    ///
    /// @param newAdjustor newly authorized address to change penalty
    function setAdjustor(address newAdjustor) external {
        if (msg.sender != adjustor) revert Unauthorized();
        require(newAdjustor != address(0), "INVALID_ADJUSTOR_ADDRESS");

        address old = adjustor;
        adjustor = newAdjustor;
        emit AdjustorChanged(old, newAdjustor);
    }
}
