// SPDX-License-Identifier: Apache-2.0.
pragma solidity ^0.8.25;

import {ERC20Permit} from "@openzeppelin/contracts/token/ERC20/extensions/ERC20Permit.sol";
import {ERC20} from "@openzeppelin/contracts/token/ERC20/extensions/ERC20Votes.sol";

string constant NAME = "Filament Token";
string constant SYMBOL = "FILA";

contract FilamentToken is ERC20Permit {
    constructor(address _dist, uint256 _amt) ERC20(NAME, SYMBOL) ERC20Permit(NAME) {
        _mint(_dist, _amt);
    }

    function _update(address from, address to, uint256 amount) internal override(ERC20) {
        super._update(from, to, amount);
    }

    function nonces(address owner) public view virtual override(ERC20Permit) returns (uint256) {
        return super.nonces(owner);
    }
}
