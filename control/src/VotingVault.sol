// SPDX-License-Identifier: MIT
pragma solidity ^0.8.25;

import {console} from "forge-std/console.sol";

import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {IERC20Metadata} from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {ERC4626} from "@openzeppelin/contracts/token/ERC20/extensions/ERC4626.sol";
import {Votes, EIP712} from "@openzeppelin/contracts/governance/utils/Votes.sol";

// From https://github.com/devanoneth/ERC4626Votes

error Unauthorized();
error PendingRedemption();
error MissingRedemption();
error LockedRedemption(uint256 unlockTime);

contract VotingVault is AccessControl, ERC4626, Votes {
    bytes32 public constant DELAY_ADJUSTOR_ROLE = keccak256("DELAY_ADJUSTOR_ROLE");

    uint256 public redemptionDelay = 14 days;

    struct RedemptionUnlock {
        uint256 timestamp;
        uint256 shares;
        address receiver;
    }

    /// @dev owner to unlock mapping
    mapping(address => RedemptionUnlock) public redemptionUnlocks;

    event redemptionDelayAdjusted(uint256 _old, uint256 _new);

    constructor(IERC20Metadata asset_, address adjustor_)
        ERC4626(asset_)
        ERC20("Filament Token Vault", "vaFILA")
        EIP712("ERC4626Votes", "v1.0")
    {
        _grantRole(DELAY_ADJUSTOR_ROLE, adjustor_);
    }

    /// @dev Redemption/Withdrawal of tokens can only be done after a delay.
    ///      During the delay the smart contract holds the shares and then
    ///      burns them after the delay. There can only be one redemption per
    ///      owner active.
    function initiateRedemption(uint256 shares_, address receiver_, address owner_) public returns (uint256) {
        if (msg.sender != owner_) revert Unauthorized();
        if (redemptionUnlocks[owner_].timestamp != 0) revert PendingRedemption();

        // Taken from the ERC4626 `redeem()` implementation.
        uint256 maxShares = maxRedeem(owner_);
        if (shares_ > maxShares) {
            revert ERC4626.ERC4626ExceededMaxRedeem(owner_, shares_, maxShares);
        }

        // XXX: want to take shares and burn them after delay or burn shares and unlock
        //      underlying after delay? Burning the shares later means owners can still
        //      benefit from value accrueing but also gives them a free option later on,
        //      which is undesirable.
        redemptionUnlocks[owner_] = RedemptionUnlock(block.timestamp + redemptionDelay, shares_, receiver_);
        SafeERC20.safeTransferFrom(this, owner_, address(this), shares_);

        return shares_;
    }

    function redeem(uint256 shares_, address receiver_, address owner_) public override returns (uint256) {
        // XXX: If we fix the receiver and the amount of shares in the Unlock struct
        //      then we can just let anyone finalize the redemption.
        RedemptionUnlock memory unlock = redemptionUnlocks[owner_];
        if (unlock.timestamp == 0) revert MissingRedemption();
        if (unlock.timestamp > block.timestamp) revert LockedRedemption(unlock.timestamp);

        delete redemptionUnlocks[owner_];
        uint256 assets = previewRedeem(unlock.shares);
        // ERC20(this).approve(address(this), unlock.shares);

        // If _asset is ERC777, `transfer` can trigger a reentrancy AFTER the transfer happens through the
        // `tokensReceived` hook. On the other hand, the `tokensToSend` hook, that is triggered before the transfer,
        // calls the vault, which is assumed not malicious.
        //
        // Conclusion: we need to do the transfer after the burn so that any reentrancy would happen after the
        // shares are burned and after the assets are transferred, which is a valid state.
        _burn(address(this), unlock.shares);
        SafeERC20.safeTransfer(ERC20(this.asset()), unlock.receiver, assets);

        emit Withdraw(msg.sender, unlock.receiver, owner_, assets, shares_);

        return unlock.shares;
    }

    function withdraw(uint256 assets_, address receiver_, address owner_) public override returns (uint256) {
        // If we fix the receiver and the amount of shares in the Unlock struct
        // then we can just let anyone finalize the redemption.
        RedemptionUnlock memory unlock = redemptionUnlocks[owner_];
        if (redemptionUnlocks[owner_].timestamp == 0) revert MissingRedemption();
        if (unlock.timestamp < block.timestamp) revert LockedRedemption(unlock.timestamp);

        uint256 assets = previewRedeem(unlock.shares);

        delete redemptionUnlocks[owner_];

        return super.withdraw(assets, unlock.receiver, address(this));
    }

    //////// Permissioned

    /// @dev Set a new redemption delay, which is only applied to new redemption
    ///      requests.
    function setRedemptionDelay(uint256 _delay) public onlyRole(DELAY_ADJUSTOR_ROLE) {
        uint256 old = redemptionDelay;

        redemptionDelay = _delay;
        emit redemptionDelayAdjusted(old, _delay);
    }

    //////// Internal

    /// @dev Adjusts votes when tokens are transferred.
    ///
    /// Emits a {Votes-DelegateVotesChanged} event.
    function _update(address from, address to, uint256 amount) internal virtual override {
        _transferVotingUnits(from, to, amount);
        super._update(from, to, amount);
    }

    /// @dev Returns the underlying asset balance of `account` which can be used by Governor
    function _getVotingUnits(address account) internal view virtual override returns (uint256) {
        return convertToAssets(balanceOf(account));
    }
}
