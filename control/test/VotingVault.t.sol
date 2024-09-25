// SPDX-License-Identifier: MIT
pragma solidity ^0.8.25;

import {Test} from "forge-std/Test.sol";
import {IERC20Metadata} from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {IAccessControl} from "@openzeppelin/contracts/access/IAccessControl.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {MockERC20} from "./mock/MockERC20.sol";

import {VotingVault, MissingRedemption, LockedRedemption} from "../src/VotingVault.sol";

contract VotingVaultTest is Test {
    VotingVault public vault;
    MockERC20 public asset;

    address public delayAdjustor = address(1);
    address public user1 = address(2);
    address public user2 = address(3);

    function setUp() public {
        asset = new MockERC20("Filamock Token", "FILA");

        // Mint some tokens to the users
        // asset.mint(user1, 1000 ether);
        asset.mint(user2, 1000 ether);

        // Deploy the vault
        vm.startPrank(delayAdjustor);
        vault = new VotingVault(IERC20Metadata(address(asset)), delayAdjustor);
        vm.stopPrank();

        // Approve the vault to spend the user's tokens
        vm.startPrank(user1);
        asset.approve(address(vault), type(uint256).max);
        vm.stopPrank();

        vm.startPrank(user2);
        asset.approve(address(vault), type(uint256).max);
        vm.stopPrank();
    }

    function test_Initialize() public {
        assertEq(vault.name(), "Filament Token Vault");
        assertEq(vault.symbol(), "vaFILA");
        assertEq(address(vault.asset()), address(asset));
    }

    function testFuzz_Deposit(uint256 amount_) public {
        vm.assume(amount_ < 100_000_000_000_000 ether);
        asset.mint(user1, amount_);

        vm.startPrank(user1);
        asset.approve(address(vault), amount_);
        vault.deposit(amount_, user1);
        assertEq(vault.balanceOf(user1), amount_);
        assertEq(asset.balanceOf(address(vault)), amount_);
        vm.stopPrank();
    }

    function testFuzz_InitiateRedemption(uint256 amount_) public {
        vm.assume(amount_ < 100_000_000_000_000 ether);
        asset.mint(user1, amount_);

        vm.startPrank(user1);
        asset.approve(address(vault), amount_);
        vault.deposit(amount_, user1);
        assertEq(vault.balanceOf(user1), amount_);

        vault.approve(address(vault), amount_);
        uint256 shares_ = vault.initiateRedemption(amount_, user1, user1);
        assertEq(shares_, amount_);
        assertEq(vault.balanceOf(user1), 0);
        vm.stopPrank();
    }

    function test_FinalizeRedemption(uint256 amount_) public {
        vm.assume(amount_ < 100_000_000_000_000 ether);
        asset.mint(user1, amount_);

        vm.startPrank(user1);
        asset.approve(address(vault), amount_);
        uint256 shares_ = vault.deposit(amount_, user1);
        assertEq(vault.balanceOf(user1), amount_);

        vault.approve(address(vault), shares_);
        vault.initiateRedemption(shares_, user1, user1);
        vm.warp(block.timestamp + vault.redemptionDelay());
        uint256 redeemedAmount = vault.redeem(shares_, user1, user1);
        assertEq(redeemedAmount, shares_);
        assertEq(vault.balanceOf(user1), 0);
        assertEq(asset.balanceOf(user1), amount_);
        vm.stopPrank();
    }

    function testFuzz_SetRedemptionDelay(uint256 newDelay_) public {
        // uint256 newDelay = 2 days;
        vm.prank(delayAdjustor);
        vault.setRedemptionDelay(newDelay_);
        assertEq(vault.redemptionDelay(), newDelay_);
    }

    // function testGetVotingUnits() public {
    //     uint256 amount = 100 ether;
    //     vm.startPrank(user1);
    //     asset.approve(address(vault), amount);
    //     vault.deposit(amount, user1);
    //     assertEq(vault.balanceOf(user1), amount);

    //     uint256 votingUnits = vault._getVotingUnits(user1);
    //     assertEq(votingUnits, vault.convertToAssets(amount));

    //     vm.stopPrank();
    // }

    function test_RevertOnMissingRedemption() public {
        vm.expectRevert(MissingRedemption.selector);
        vault.redeem(100 ether, user1, user1);
    }

    function testFuzz_RevertOnLockedRedemption(uint256 delta_) public {
        vm.assume(delta_ < vault.redemptionDelay());

        uint256 amount = 100 ether;
        asset.mint(user1, amount);

        vm.startPrank(user1);
        asset.approve(address(vault), amount);
        uint256 shares = vault.deposit(amount, user1);
        assertEq(vault.balanceOf(user1), amount);

        vm.warp(block.timestamp + delta_);

        vault.approve(address(vault), shares);
        vault.initiateRedemption(amount, user1, user1);
        vm.expectPartialRevert(LockedRedemption.selector);
        vault.redeem(amount, user1, user1);
        vm.stopPrank();
    }

    function testFuzz_RevertOnInvalidRole(uint256 newDelay_) public {
        vm.expectPartialRevert(IAccessControl.AccessControlUnauthorizedAccount.selector);
        vault.setRedemptionDelay(newDelay_);
    }
}
