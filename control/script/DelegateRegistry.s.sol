// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.16;

import "forge-std/Script.sol";
import {DelegateRegistry} from "../src/DelegateRegistry.sol";

contract DeployDelegateRegistryScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);

        address adjustor_ = vm.envAddress("ADJUSTOR_ADDRESS");
        DelegateRegistry dr = new DelegateRegistry(adjustor_);

        vm.stopBroadcast();
    }
}

contract SetDelegatesScript is Script {
    function run() external {
        uint256 adjustorPrivateKey_ = vm.envUint("ADJUSTOR_PRIVATE_KEY");
        vm.startBroadcast(adjustorPrivateKey_);

        address[] memory delegates_ = vm.envAddress("DELEGATES",",");
        DelegateRegistry dr = DelegateRegistry(vm.envAddress("DELEGATE_REGISTRY_ADDRESS"));

        dr.setDelegates(delegates_);

        vm.stopBroadcast();
    }
}
