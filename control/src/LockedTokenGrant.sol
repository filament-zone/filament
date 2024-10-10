// SPDX-License-Identifier: Apache-2.0.
pragma solidity ^0.8.16;

import "./TimeLockedTokens.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {Math} from "@openzeppelin/contracts/utils/math/Math.sol";
import {IERC4626} from "@openzeppelin/contracts/interfaces/IERC4626.sol";

error Unauthorized();

contract LockedTokenGrant is TimeLockedTokens {
    address public immutable manager;

    address public immutable token;

    address public immutable recipient;

    address public immutable stakingContract;

    uint256 public releasedTokens;

    event TokensSentToRecipient(
        address indexed recipient, address indexed grantContract, uint256 amountSent, uint256 aggregateSent
    );

    event EarlyTokenRelease(address indexed recipient, address indexed grantContract, uint256 amount, uint256 penalty);

    event TokensClawedBack(address indexed target, uint256 amount);

    constructor(
        address token_,
        address manager_,
        address stakingContract_,
        address penaltyManager_,
        address recipient_,
        uint256 grantAmount_,
        uint256 startTime_,
        uint256 endTime_
    ) TimeLockedTokens(penaltyManager_, grantAmount_, startTime_, endTime_) {
        manager = manager_;
        token = token_;
        recipient = recipient_;
        stakingContract = stakingContract_;
    }

    /*
      Returns the available tokens for release.
      Once the grant lock is fully expired - the entire balance is always available.
      Until then, only the relative part of the grant grantAmount is available.
      However, given staking, the actual balance may be smaller.
      Note that any excessive tokens (beyond grantAmount) transferred to this contract
      are going to be locked until the grant lock fully expires.
    */
    function availableTokens() public view returns (uint256) {
        uint256 currentBalance = IERC20(token).balanceOf(address(this));
        return isGrantFullyUnlocked() ? currentBalance : Math.min(currentBalance, (unlockedTokens() - releasedTokens));
    }

    /*
      Transfers `requestedAmount` tokens (if available) to the `recipient`.
    */
    // function releaseTokens(uint256 requestedAmount) external onlyAllowedAgent(LOCKED_TOKEN_RELEASE_AGENT) {
    function releaseTokens(uint256 requestedAmount) external {
        if (msg.sender != recipient) revert Unauthorized();
        require(requestedAmount <= availableTokens(), "REQUESTED_AMOUNT_UNAVAILABLE");

        releasedTokens += requestedAmount;
        IERC20(token).transfer(recipient, requestedAmount);
        emit TokensSentToRecipient(recipient, address(this), requestedAmount, releasedTokens);
    }

    function availableEarlyTokens() public view returns (uint256) {
        uint256 currentBalance = IERC20(token).balanceOf(address(this));
        if (isGrantFullyUnlocked()) return currentBalance;

        uint256 available = unlockedTokens() - releasedTokens;
        // eg. grantAmount is 1000, unlockedTokens is 500, releasedTokens is 400,
        // available is 100.
        // we need to apply the penalty to grantAmount - unlockedTokens and then
        // return toBePenalized - penalty + available
        uint256 toBePenalized = grantAmount - unlockedTokens();
        uint256 penalty = penaltyManager.computePenalty(toBePenalized);
        return available + toBePenalized - penalty;
    }

    // XXX: should we allow partial early releases?
    function earlyReleaseTokens() external {
        if (msg.sender != recipient) revert Unauthorized();

        // earlyExitPenalty() computes penalty based on locked tokens only
        uint256 available = unlockedTokens() - releasedTokens;
        uint256 toBePenalized = grantAmount - unlockedTokens();
        uint256 penalty = penaltyManager.computePenalty(toBePenalized);
        uint256 release = available + toBePenalized - penalty;

        releasedTokens = grantAmount;

        IERC20(token).transfer(stakingContract, penalty);
        IERC20(token).transfer(recipient, release);

        emit EarlyTokenRelease(recipient, address(this), release, penalty);
    }

    function claw(address target_) external {
        if (msg.sender != manager) revert Unauthorized();

        uint256 currentBalance = IERC20(token).balanceOf(address(this));
        IERC20(token).transfer(target_, currentBalance);

        emit TokensClawedBack(target_, currentBalance);
    }
}
