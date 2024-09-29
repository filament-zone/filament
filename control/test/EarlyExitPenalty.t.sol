// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.25;

import {Test, console} from "forge-std/Test.sol";
import {
    EarlyExitPenalty,
    InvalidAdjustor,
    InvalidEndTime,
    PenaltyTooHigh,
    PenaltyTooLow,
    Unauthorized
} from "../src/EarlyExitPenalty.sol";

contract EarlyExitPenaltyTest is Test {
    EarlyExitPenalty public eep;

    function setUp() public {
        address adjustor_ = address(this);
        uint16 penalty = 9000; // 9000 bps or 90%
        uint16 maxPenalty = 10000;
        uint16 minPenalty = 100;
        uint256 startTime = 1717556330;
        uint256 endTime = startTime + 730 days;
        eep = new EarlyExitPenalty(adjustor_, penalty, maxPenalty, minPenalty, startTime, endTime);
    }

    function testFuzz_Adjustor(address adjustor_) public {
        vm.assume(adjustor_ != address(0));
        eep.setAdjustor(adjustor_);
        assertEq(eep.adjustor(), adjustor_);
    }

    function test_ElapsedTime_AfterEndTime() public {
        vm.warp(eep.endTime() + 100 days);
        assertEq(eep.elapsedTime(), eep.penaltyTime());
    }

    function testFuzz_SetEndTime(uint256 endTime_) public {
        vm.assume(endTime_ > eep.startTime());
        eep.setEndTime(endTime_);
        assertEq(eep.endTime(), endTime_);
        assertEq(eep.endTime() - eep.startTime(), eep.penaltyTime());
    }

    function test_ComputePenalty() public {
        vm.warp(eep.startTime());

        uint256 penalty = eep.computePenalty(1 ether);
        assertEq(penalty, 0.9 ether);

        vm.warp(eep.startTime() + eep.penaltyTime() / 2);

        penalty = eep.computePenalty(1 ether);
        assertEq(penalty, 0.45 ether);

        vm.warp(eep.endTime());

        penalty = eep.computePenalty(1 ether);
        assertEq(penalty, 0);
    }

    function testFuzz_ComputePenalty_RandomOffset(uint256 offset_) public {
        // anything below 0.1% of duration is lost to rounding
        vm.assume(offset_ > eep.penaltyTime() / 1000);
        vm.assume(offset_ <= eep.penaltyTime());

        vm.warp(eep.startTime());

        uint256 penalty = eep.computePenalty(1 ether);
        assertEq(penalty, 0.9 ether);

        vm.warp(eep.startTime() + offset_);

        penalty = eep.computePenalty(1 ether);
        assertLt(penalty, 0.9 ether);

        vm.warp(eep.endTime());

        penalty = eep.computePenalty(1 ether);
        assertEq(penalty, 0);
    }

    function testFuzz_SetPenalty(uint16 x) public {
        vm.assume(x >= eep.minPenalty());
        vm.assume(x <= eep.maxPenalty());

        eep.setPenalty(x);
        assertEq(eep.penalty(), x);
    }

    function testFuzz_SetPenalty_ComputePenaltyUpdate(uint16 x) public {
        vm.assume(x >= eep.minPenalty());
        vm.assume(x <= eep.maxPenalty());

        uint256 penalty = eep.computePenalty(1 ether);
        assertEq(penalty, 0.9 ether);

        vm.warp(eep.startTime() + eep.penaltyTime() / 2);

        penalty = eep.computePenalty(1 ether);
        assertEq(penalty, 0.45 ether);

        eep.setPenalty(x);

        penalty = eep.computePenalty(1 ether);
        assertApproxEqRel(penalty, (1 ether * uint256(x)) / 10000 / 2, 0.01 ether);

        vm.warp(eep.endTime());
        penalty = eep.computePenalty(1 ether);
        assertEq(penalty, 0);
    }

    ///// Failure

    function test_SetAdjustor_InvalidAdjustor() public {
        vm.expectRevert(InvalidAdjustor.selector);

        eep.setAdjustor(address(0));
    }

    function testFuzz_SetAdjustor_Unauthorized(address adjustor_) public {
        vm.assume(adjustor_ != address(this));

        vm.prank(adjustor_);
        vm.expectRevert(Unauthorized.selector);
        eep.setAdjustor(adjustor_);
    }

    function testFuzz_SetEndTime_Unauthorized(address adjustor_) public {
        vm.assume(adjustor_ != address(this));

        uint256 endTime_ = eep.endTime();

        vm.prank(adjustor_);
        vm.expectRevert(Unauthorized.selector);
        eep.setEndTime(endTime_);
    }

    function testFuzz_SetEndTime_InvalidEndTime(uint256 endTime_) public {
        vm.assume(endTime_ <= eep.startTime());

        vm.expectRevert(InvalidEndTime.selector);
        eep.setEndTime(endTime_);
    }

    function testFuzz_SetPenalty_TooHigh(uint16 x) public {
        vm.assume(x > eep.maxPenalty());
        vm.expectRevert(PenaltyTooHigh.selector);

        eep.setPenalty(x);
    }

    function testFuzz_SetPenalty_TooLow(uint16 x) public {
        vm.assume(x < eep.minPenalty());
        vm.expectRevert(PenaltyTooLow.selector);

        eep.setPenalty(x);
    }

    function testFuzz_SetPenalty_Unauthorized(address adjustor_) public {
        vm.assume(adjustor_ != address(this));

        vm.prank(adjustor_);
        vm.expectRevert(Unauthorized.selector);
        eep.setPenalty(1000);
    }
}
