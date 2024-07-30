// SPDX-License-Identifier: MIT
pragma solidity ^0.8.25;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {IERC20Metadata} from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {ERC4626} from "@openzeppelin/contracts/token/ERC20/extensions/ERC4626.sol";
import {Votes, EIP712} from "@openzeppelin/contracts/governance/utils/Votes.sol";

// From https://github.com/devanoneth/ERC4626Votes

contract VotingVault is ERC4626, Votes {
    constructor(IERC20Metadata _asset)
        ERC4626(_asset)
        ERC20("Filament Token Vault", "vaFILA")
        EIP712("ERC4626Votes", "v1.0")
    {}

    /**
     * @dev Adjusts votes when tokens are transferred.
     *
     * Emits a {Votes-DelegateVotesChanged} event.
     */
    function _update(address from, address to, uint256 amount) internal virtual override {
        _transferVotingUnits(from, to, amount);
        super._update(from, to, amount);
    }

    /**
     * @dev Returns the underlying asset balance of `account` which can be used by Governor
     */
    function _getVotingUnits(address account) internal view virtual override returns (uint256) {
        return convertToAssets(balanceOf(account));
    }
}
