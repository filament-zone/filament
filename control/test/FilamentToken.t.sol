// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.25;

import {Test, console} from "forge-std/Test.sol";
import {FilamentToken} from "../src/FilamentToken.sol";

contract FilamentTokenTest is Test {
    function setUp() public {}

    function testFuzz_Constructor(uint256 amount_) public {
        address receiver_ = makeAddr("receiver");
        FilamentToken ft = new FilamentToken(receiver_, amount_);

        assertEq(ft.balanceOf(receiver_), amount_);
    }
}
