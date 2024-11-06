// SPDX-License-Identifier: Apache-2.0.
pragma solidity ^0.8.16;

import {Pausable} from "@openzeppelin/contracts/utils/Pausable.sol";

error Unauthorized();
error InvalidManager();
error InvalidSecurityCouncil();

contract BaseSecurity is Pausable {
    /// @notice The manager can
    address public manager;

    /// @notice The security council can pause this contract, after which the manager
    ///         is allowed to perform security critical operations
    address public securityCouncil;

    event ManagerChanged(address indexed oldManager, address indexed newManager);
    event SecurityCouncilChanged(address indexed oldManager, address indexed newManager);
    event ContractPaused();

    constructor(address manager_, address securityCouncil_) {
        manager = manager_;
        securityCouncil = securityCouncil_;
    }

    function pause() external {
        if (msg.sender != securityCouncil) revert Unauthorized();

        _pause();

        emit ContractPaused();
    }

    function unpause() external {
        if (msg.sender != securityCouncil) revert Unauthorized();

        _unpause();

        emit ContractPaused();
    }

    function setManager(address manager_) external {
        if (msg.sender != manager) revert Unauthorized();
        if (manager_ == address(0)) revert InvalidManager();

        address old = manager;
        manager = manager_;

        emit ManagerChanged(old, manager_);
    }

    function setSecurityCouncil(address council_) external {
        if (msg.sender != securityCouncil) revert Unauthorized();
        if (council_ == address(0)) revert InvalidSecurityCouncil();

        address old = securityCouncil;
        securityCouncil = council_;

        emit SecurityCouncilChanged(old, council_);
    }
}
