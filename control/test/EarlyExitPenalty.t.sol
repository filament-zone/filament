// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.25;

import {Test, console} from "forge-std/Test.sol";
import {EarlyExitPenalty, PenaltyTooHigh, PenaltyTooLow, Unauthorized} from "../src/EarlyExitPenalty.sol";

contract EarlyExitPenaltyTest is Test {
    EarlyExitPenalty public eep;

    function setUp() public {
        address adjustor_ = address(this);
        uint16 penalty = 1000; // 1000 bps or 10%
        uint16 maxPenalty = 2000;
        uint16 minPenalty = 100;
        eep = new EarlyExitPenalty(address(this), penalty, maxPenalty, minPenalty);
    }

    function test_Adjustor() public view {
        assertEq(eep.adjustor(), address(this));
    }

    function test_ComputePenalty() public {
        uint256 penalty = eep.computePenalty(1 ether);
        assertEq(penalty, 0.1 ether);
    }

    function test_SetAdjustorUnauthorized() public {
        vm.prank(address(0));
        vm.expectRevert(Unauthorized.selector);
        eep.setPenalty(10000);
    }

    function testFuzz_SetPenalty(uint16 x) public {
        vm.assume(x >= eep.minPenalty());
        vm.assume(x <= eep.maxPenalty());

        eep.setPenalty(x);
        assertEq(eep.penalty(), x);
    }

    function testFuzz_SetPenaltyHigh(uint16 x) public {
        vm.assume(x > eep.maxPenalty());
        vm.expectRevert(PenaltyTooHigh.selector);

        eep.setPenalty(x);
    }

    function testFuzz_SetPenaltyLow(uint16 x) public {
        vm.assume(x < eep.minPenalty());
        vm.expectRevert(PenaltyTooLow.selector);

        eep.setPenalty(x);
    }
}
