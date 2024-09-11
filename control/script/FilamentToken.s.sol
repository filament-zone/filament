// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.25;

import "forge-std/Script.sol";
import {FilamentToken} from "../src/FilamentToken.sol";

contract DeployFilaTokenScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);

        address receiver = vm.envAddress("TOKEN_RECEIVER");
        uint256 mintAmount = vm.envUint("TOKEN_MINT_AMOUNT");
        FilamentToken token = new FilamentToken(receiver, mintAmount);

        vm.stopBroadcast();
    }
}
