// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.25;

import {Test, console} from "forge-std/Test.sol";
import {FilamentToken} from "../src/FilamentToken.sol";

contract FilamentTokenTest is Test {
    function setUp() public {}

    function testFuzz_Constructor(address _receiver, uint256 _amount) public {
        FilamentToken tok = new FilamentToken(_receiver, _amount);

        assertEq(tok.balanceOf(_receiver), _amount);
    }
}
