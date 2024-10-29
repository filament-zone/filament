// SPDX-License-Identifier: Apache-2.0.
pragma solidity ^0.8.16;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SignedMath} from "@openzeppelin/contracts/utils/math/SignedMath.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

import {BaseSecurity} from "./BaseSecurity.sol";
import {TreasuryWindow} from "./TreasuryWindow.sol";

error BondAlreadyLocked();
error BondSlashed();
error BondExpired();
error BondNotFound(uint256 campaign);
error CampaignerInsufficientFunds();
error UnauthorizedRelayer();

contract Paymaster is BaseSecurity {
    using SafeERC20 for IERC20;

    struct Bond {
        uint256 amount;
        uint256 target;
        bool slashed;
        bool locked;
        bool expired;
    }

    // struct Funds {
    //     uint256 fila;
    //     uint256 usd;
    // }

    IERC20 public immutable fila;
    IERC20 public immutable usd;

    address public relayer;
    TreasuryWindow public window;

    mapping(address => mapping(uint256 => Bond)) bonds;
    mapping(address => uint256) availableFunds;

    event BondLocked(uint256 indexed campaignId, uint256 amount);
    event BondToppedUp(uint256 indexed campaignId, uint256 supplied, uint256 target, uint256 amount);
    event FilaDeposited(address from, uint256 amount);
    event UsdDeposited(address from, uint256 amountUsd, uint256 amountFila);
    event FilaWithdrawn(address from, uint256 amount);

    constructor(
        address fila_,
        address usd_,
        address window_,
        address relayer_,
        address manager_,
        address securityCouncil_
    ) BaseSecurity(manager_, securityCouncil_) {
        fila = IERC20(fila_);
        usd = IERC20(usd_);
        window = TreasuryWindow(window_);
        relayer = relayer_;
    }

    /// @notice Deposit FILA tokens for the given `campaigner_` which can then be
    ///         used to bond for campaigns.
    /// @param amount_ Amount of FILA to deposit
    /// @param campaigner_ The campaigner to get credited for deposited tokens
    function depositFila(uint256 amount_, address campaigner_) external whenNotPaused {
        // XXX: do we care who deposits?

        fila.safeTransferFrom(msg.sender, address(this), amount_);
        availableFunds[campaigner_] += amount_;

        emit FilaDeposited(campaigner_, amount_);
    }

    /// @notice Use USD tokens to deposit. These USD tokens will be automatically
    ///         converted to FILA via the treasury window.
    function depositUsd(uint256 amount_, uint256 minOut_, address campaigner_) external whenNotPaused {
        usd.safeTransferFrom(msg.sender, address(this), amount_);
        usd.approve(address(window), amount_);

        uint256 out_ = window.getFila(amount_, minOut_, address(this));

        availableFunds[campaigner_] += out_;
        emit UsdDeposited(campaigner_, amount_, out_);
    }

    /// @notice
    function topUpBond(address campaigner_, uint256 campaign_, uint256 amount_) external whenNotPaused {
        Bond storage bond_ = bonds[msg.sender][campaign_];
        if (bond_.slashed) revert BondSlashed();

        fila.safeTransferFrom(msg.sender, address(this), amount_);
        bond_.amount += amount_;

        emit BondToppedUp(campaign_, amount_, bond_.target, bond_.amount);
    }

    function topUpBondUsd(address campaigner_, uint256 campaign_, uint256 amount_, uint256 minOut_)
        external
        whenNotPaused
    {
        Bond storage bond_ = bonds[msg.sender][campaign_];
        if (bond_.slashed) revert BondSlashed();

        usd.safeTransferFrom(msg.sender, address(this), amount_);
        usd.approve(address(window), amount_);

        uint256 out_ = window.getFila(amount_, minOut_, address(this));
        bond_.amount += out_;

        emit BondToppedUp(campaign_, out_, bond_.target, bond_.amount);
    }

    function pay() external whenNotPaused {}

    ///// Permissioned

    function lockBond(address campaigner_, uint256 campaign_, uint256 amount_) external whenNotPaused {
        if (msg.sender != relayer) revert UnauthorizedRelayer();

        Bond storage bond_ = bonds[campaigner_][campaign_];
        if (bond_.locked) revert BondAlreadyLocked();

        bond_.locked = true;
    }

    function setBondTarget(address campaigner_, uint256 campaign_, uint256 target_) external whenNotPaused {
        if (msg.sender != relayer) revert UnauthorizedRelayer();

        Bond storage bond_ = bonds[campaigner_][campaign_];
        if (bond_.slashed) revert BondSlashed();
        if (bond_.expired) revert BondExpired();

        if (bond_.target == target_ && bond_.amount == bond_.target) return;
        if (bond_.target != target_ && bond_.amount == target_) {
            bond_.target = target_;
            return;
        }

        // XXX: if we don't want to automatically adjust the amount then most of the
        //      code below can be removed.
        // int256 targetDelta_ = bond_.target - target_;
        // int256 amountDelta_ = bond_.amount - bond_.target;
        // int256 delta_ = targetDelta_ - amountDelta_;
        // uint256 abs_ = SignedMath.abs(delta_);

        // if (delta_ < 0) {
        //     // Bond target > amount
        //     uint256 toTransfer_ = availableFunds[campaigner_] < abs_ ? availableFunds[campaigner_] : abs_;
        //     availableFunds[campaigner_] -= toTransfer_;
        //     bond_.amount += toTransfer_;
        //     bond_.target = target_; // target might still not be reached
        // } else {
        //     // Bond target < amount
        //     // delta > 0, because delta == 0 is covered by (bond.target != target_ && bond.amount == target_)
        //     availableFunds[campaigner_] += abs_;
        //     bond_.amount -= abs_;
        //     bond_.target = target_;
        // }
    }
}
