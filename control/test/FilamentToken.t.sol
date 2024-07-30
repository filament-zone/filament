// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.25;

import {Test, console} from "forge-std/Test.sol";
import {FilamentToken} from "../src/FilamentToken.sol";

contract FilamentTokenTest is Test {
    FilamentToken public ft;

    address public adjustor;

    function setUp() public {
        ft = new FilamentToken(address(0), 100);
    }

}
