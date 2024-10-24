// SPDX-License-Identifier: Apache-2.0.
pragma solidity ^0.8.16;

import {Test, console} from "forge-std/Test.sol";

import {DelegateRegistry, Unauthorized} from "../src/DelegateRegistry.sol";

contract DelegateRegistryTest is Test {
    DelegateRegistry public dr;
    address public adjustor;

    address[] public delegates;

    function setUp() public {
        adjustor = makeAddr("adjustor");
        dr = new DelegateRegistry(adjustor);
    }

    function test_ClearDelegates() public {
        // vm.assume(adjustor_ != address(0));
        delegates = [makeAddr("d1"), makeAddr("d2"), makeAddr("d3")];

        vm.prank(adjustor);
        dr.setDelegates(delegates);

        for (uint256 i = 0; i < delegates.length; i++) {
            assert(dr.isDelegate(delegates[i]));
        }

        vm.prank(adjustor);
        dr.clearDelegates();

        for (uint256 i = 0; i < delegates.length; i++) {
            assert(!dr.isDelegate(delegates[i]));
        }
    }

    function test_SetDelegates_Duplicates() public {
        // vm.assume(adjustor_ != address(0));
        delegates = [makeAddr("d1"), makeAddr("d1"), makeAddr("d3")];

        vm.prank(adjustor);
        dr.setDelegates(delegates);

        assertEq(dr.size(), 2);
        for (uint256 i = 0; i < delegates.length; i++) {
            assert(dr.isDelegate(delegates[i]));
        }
    }

    function test_SetDelegates() public {
        // vm.assume(adjustor_ != address(0));
        delegates = [makeAddr("d1"), makeAddr("d2"), makeAddr("d3")];

        vm.prank(adjustor);
        dr.setDelegates(delegates);

        for (uint256 i = 0; i < delegates.length; i++) {
            assert(dr.isDelegate(delegates[i]));
        }

        delegates = [makeAddr("d1"), makeAddr("d2")];
        vm.prank(adjustor);
        dr.setDelegates(delegates);

        assert(!dr.isDelegate(makeAddr("d3")));
    }

    function testFuzz_SetDelegates(address[] memory delegates_) public {
        // vm.assume(adjustor_ != address(0));

        vm.prank(adjustor);
        dr.setDelegates(delegates_);

        for (uint256 i = 0; i < delegates_.length; i++) {
            assert(dr.isDelegate(delegates_[i]));
        }
    }

    function testFuzz_SetAdjustor(address adjustor_) public {
        vm.assume(adjustor_ != address(0));

        vm.prank(adjustor);
        dr.setAdjustor(adjustor_);
        assertEq(dr.adjustor(), adjustor_);
    }

    function testFuzz_SetAdjustor_Unauthorized(address adjustor_) public {
        vm.assume(adjustor_ != adjustor);

        vm.prank(adjustor_);
        vm.expectRevert(Unauthorized.selector);
        dr.setAdjustor(adjustor_);
    }
}
