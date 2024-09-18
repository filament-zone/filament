// SPDX-License-Identifier: Apache-2.0.
pragma solidity ^0.8.25;

import {IERC20} from "@openzeppelin/contracts/interfaces/IERC20.sol";

// The only purpose of this contract is to receive a token transfer and
// then distribute it to a list of addresses.
contract TokenDistributor {
    address public immutable owner;
    address public immutable token;

    event TokenSent(address indexed recipient, uint256 amount);

    constructor(address _owner, address _token) {
        owner = _owner;
        token = _token;
    }

    function distribute(address[] memory recipients, uint256[] memory amounts) external {
        require(msg.sender == owner, "NOT AUTH");
        require(recipients.length == amounts.length, "MISMATCH");

        for (uint256 i = 0; i < recipients.length; i++) {
            address recipient = recipients[i];
            uint256 amount = amounts[i];

            IERC20(token).transferFrom(address(this), recipient, amount);
            emit TokenSent(recipient, amount);
        }
    }
}
