// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.25;

import {Test, console} from "forge-std/Test.sol";
import {MockERC20} from "./mock/MockERC20.sol";

import {EarlyExitPenalty, PenaltyTooHigh, PenaltyTooLow} from "../src/EarlyExitPenalty.sol";

import {TimeLockedTokens} from "../src/TimeLockedTokens.sol";

import {LockedTokenGrant, Unauthorized} from "../src/LockedTokenGrant.sol";

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
// import "@openzeppelin/contracts/token/ERC721/IERC721.sol";

contract TimeLockedTokensTest is Test {
    EarlyExitPenalty public eep;

    MockERC20 private token;
    address private stakingContract;

    struct GrantDetails {
        uint256 grantAmount;
        uint256 startTime;
        uint256 endTime;
    }

    address public adjustor;

    function setUp() public {
        address adjustor_ = address(this);
        uint16 penalty = 1000; // 1000 bps or 10%
        uint16 maxPenalty = 2000;
        uint16 minPenalty = 100;
        uint256 startTime = 1717556330;
        uint256 endTime = startTime + 720 days;
        eep = new EarlyExitPenalty(address(this), penalty, maxPenalty, minPenalty, startTime, endTime);

        // penaltyManager = makeAddr("penaltyManager");
        token = new MockERC20("Filamock Token", "FILA");

        stakingContract = address(0);
    }

    function createGrantDetails() public returns (GrantDetails memory) {
        return GrantDetails({grantAmount: 1000, startTime: block.timestamp + 1 days, endTime: block.timestamp + 2 days});
    }

    function testFuzz_SetUp(uint128 start_, uint128 end_) public {
        vm.assume(start_ < end_);
        uint256 grantAmount = 500 ether;
        address recipient = makeAddr("recipient");

        LockedTokenGrant ltg = new LockedTokenGrant(
            address(token), address(stakingContract), address(eep), recipient, grantAmount, start_, end_
        );

        vm.warp(start_);
        assertEq(ltg.unlockedTokens(), 0);

        vm.warp(end_);
        assertEq(ltg.unlockedTokens(), grantAmount);
    }

    function test_TokenRelease_Success() public {
        address recipient = makeAddr("recipient");
        uint256 grantAmount = 500 ether;

        GrantDetails memory grantDetails = createGrantDetails();
        LockedTokenGrant ltg = new LockedTokenGrant(
            address(token),
            address(stakingContract),
            address(eep),
            recipient,
            grantDetails.grantAmount,
            grantDetails.startTime,
            grantDetails.endTime
        );

        token.mint(address(ltg), grantAmount);

        assertEq(token.balanceOf(address(ltg)), grantAmount);
        assertEq(ltg.availableTokens(), 0);

        vm.warp(grantDetails.endTime);
        assertEq(ltg.availableTokens(), grantAmount);

        assertEq(token.balanceOf(address(recipient)), 0);
        vm.prank(recipient);
        ltg.releaseTokens(grantAmount);
        assertEq(token.balanceOf(address(recipient)), grantAmount);
    }

    function test_TokenRelease_Failure_InvalidRequester() public {
        address recipient = makeAddr("recipient");

        GrantDetails memory grantDetails = createGrantDetails();
        LockedTokenGrant ltg = new LockedTokenGrant(
            address(token),
            address(stakingContract),
            address(eep),
            recipient,
            grantDetails.grantAmount,
            grantDetails.startTime,
            grantDetails.endTime
        );

        token.mint(address(ltg), grantDetails.grantAmount);
        vm.warp(grantDetails.endTime);

        vm.prank(makeAddr("notrecipient"));
        vm.expectRevert(Unauthorized.selector);

        ltg.releaseTokens(grantDetails.grantAmount);
    }

    function testFuzz_PartialTokenRelease(uint64 start_, uint64 end_, uint256 grant_) public {
        vm.assume(start_ < end_);
        vm.assume(end_ - start_ > 1 days); // arbitrary constraint
        vm.assume(grant_ < 100_000_000_000_000 ether);
        vm.assume(grant_ > 1 ether);

        uint256 half = (end_ - start_) / 2;

        address recipient = makeAddr("recipient");
        uint256 grantAmount = grant_;

        GrantDetails memory grantDetails = createGrantDetails();
        LockedTokenGrant ltg = new LockedTokenGrant(
            address(token), address(stakingContract), address(eep), recipient, grantAmount, start_, end_
        );

        vm.warp(start_ + half);
        token.mint(address(ltg), grantAmount);

        uint256 firstAvailable = ltg.availableTokens();
        assertApproxEqRel(firstAvailable, grantAmount / 2, 0.01 ether);

        vm.prank(recipient);
        ltg.releaseTokens(firstAvailable);
        assertEq(token.balanceOf(recipient), firstAvailable);

        vm.warp(end_);
        uint256 secondAvailable = ltg.availableTokens();
        assertEq(secondAvailable, grantAmount - firstAvailable);

        vm.prank(recipient);
        ltg.releaseTokens(secondAvailable);
        assertEq(token.balanceOf(recipient), grantAmount);
    }

    function testFuzz_EarlyTokenRelease(uint64 start_, uint64 end_, uint256 grant_) public {
        vm.assume(start_ < end_);
        vm.assume(end_ - start_ > 1 days); // arbitrary constraint
        vm.assume(grant_ < 100_000_000_000_000 ether);
        vm.assume(grant_ > 1 ether);

        uint256 half = (end_ - start_) / 2;

        address recipient = makeAddr("recipient");
        address penaltyRecipient = makeAddr("penaltyRecipient");
        uint256 grantAmount = grant_;

        GrantDetails memory grantDetails = createGrantDetails();
        LockedTokenGrant ltg =
            new LockedTokenGrant(address(token), penaltyRecipient, address(eep), recipient, grantAmount, start_, end_);

        token.mint(address(ltg), grantAmount);
        vm.warp(start_);

        uint256 available = ltg.availableTokens();
        assertEq(available, 0);

        uint256 earlyTokens = ltg.availableEarlyTokens();
        vm.prank(recipient);
        ltg.earlyReleaseTokens();
        assertEq(token.balanceOf(recipient), earlyTokens);
        assertEq(token.balanceOf(address(ltg)), 0);
        assertEq(token.balanceOf(penaltyRecipient), grantAmount - earlyTokens);
    }
}
