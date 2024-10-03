// SPDX-License-Identifier: Apache-2.0.
pragma solidity ^0.8.16;

import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

error Unauthorized();
error InvalidAdjustor();

/// @notice The DelegateRegistry is a simple contract that holds all the valid
///         delegates which can be chosen when staking.
contract DelegateRegistry {
    using EnumerableSet for EnumerableSet.AddressSet;

    address public adjustor;

    address[] public delegates;
    mapping(address => bool) public delegateSet;

    event DelegateSetChanged();

    event AdjustorChanged(address oldAdjustor, address newAdjustor);

    constructor(address adjustor_) {
        adjustor = adjustor_;
    }

    function isDelegate(address who_) external view returns (bool) {
        return delegateSet[who_];
    }

    function allDelegates() external view returns (address[] memory) {
        return delegates;
    }

    function size() external view returns (uint256) {
        return delegates.length;
    }

    function setDelegates(address[] memory newDelegates_) external {
        if (msg.sender != adjustor) revert Unauthorized();

        for (uint256 i = 0; i < delegates.length; i++) {
            delete delegateSet[delegates[i]];
        }
        delete delegates;

        for (uint256 i = 0; i < newDelegates_.length; i++) {
            if (delegateSet[newDelegates_[i]]) continue;
            delegates.push(newDelegates_[i]);
            delegateSet[newDelegates_[i]] = true;
        }

        emit DelegateSetChanged();
    }

    /// @dev Clear the delegate set. This function is just a safety measurement in
    ///      case the delegate list gets too long to do clear/add in `setDelegates`.
    function clearDelegates() external {
        if (msg.sender != adjustor) revert Unauthorized();

        for (uint256 i = 0; i < delegates.length; i++) {
            delete delegateSet[delegates[i]];
        }
        delete delegates;
    }

    /// @notice Set the address which can modify the delegate set.
    ///
    /// @param newAdjustor_ newly authorized address to change delegate set
    function setAdjustor(address newAdjustor_) external {
        if (msg.sender != adjustor) revert Unauthorized();
        if (newAdjustor_ == address(0)) revert InvalidAdjustor();

        address old = adjustor;
        adjustor = newAdjustor_;
        emit AdjustorChanged(old, newAdjustor_);
    }
}
