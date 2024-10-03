// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.16;

import {StdInvariant} from "forge-std/StdInvariant.sol";
import {Test, console} from "forge-std/Test.sol";

import {DelegateRegistry, Unauthorized} from "../../src/DelegateRegistry.sol";

contract DelegateRegistryInvariantTest is StdInvariant, Test {
    DelegateRegistry public dr;
    address public adjustor;

    function setUp() public {
        adjustor = makeAddr("adjustor");
        dr = new DelegateRegistry(adjustor);
    }

    function invariant_DelegateConsistency() public view {
        address[] memory delegates = dr.allDelegates();

        for (uint256 i = 0; i < delegates.length; i++) {
            assert(dr.isDelegate(delegates[i]));
        }
    }
}
