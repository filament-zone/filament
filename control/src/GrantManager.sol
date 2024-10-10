// SPDX-License-Identifier: Apache-2.0.
pragma solidity ^0.8.25;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {LockedTokenGrant} from "./LockedTokenGrant.sol";

contract GrantManager {
    address public immutable token;

    address public controller;

    address public immutable stakingContract;

    // XXX: do we want to hardcode this one?
    address public immutable penaltyManager;

    event TokensGranted(address indexed recipient, uint256 amount, uint256 start, uint256 end);

    event ControllerChanged(address oldController, address indexed newController);

    constructor(address controller_, address token_, address stakingContract_, address penaltyManager_) {
        controller = controller_;
        token = token_;
        stakingContract = stakingContract_;
        penaltyManager = penaltyManager_;
    }

    // Permissioned functions

    function giveGrant(address recipient_, address clawManager_, uint256 amount_, uint256 start_, uint256 end_)
        external
    {
        require(msg.sender == controller, "UNAUTHORIZED");

        LockedTokenGrant grant = new LockedTokenGrant(
            token, clawManager_, stakingContract, penaltyManager, recipient_, amount_, start_, end_
        );

        IERC20(token).transfer(address(grant), amount_);

        emit TokensGranted(recipient_, amount_, start_, end_);
    }

    function setController(address new_) external {
        require(msg.sender == controller, "UNAUTHORIZED");
        require(new_ != address(0), "INVALID_ADDRESS");

        address old = controller;
        controller = new_;

        emit ControllerChanged(old, new_);
    }
}
