// SPDX-License-Identifier: Apache-2.0.
pragma solidity ^0.8.16;

import "./TimeLockedTokens.sol";
// import "starkware/isd/solidity/CommonConstants.sol";
// import "starkware/isd/solidity/DelegationSupport.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {Math} from "@openzeppelin/contracts/utils/math/Math.sol";
import {IERC4626} from "@openzeppelin/contracts/interfaces/IERC4626.sol";

/**
 * This Contract holds a grant of locked tokens and gradually releases the tokens to its recipient.
 *
 *   This contract should be deployed through the {LockedTokenCommon} contract,
 *   The global lock expiration time may be adjusted through the {LockedTokenCommon} contract.
 *
 *   The {LockedTokenGrant} is initialized  with the following parameters:
 *   `address token_`: The address of StarkNet token ERC20 contract.
 *   `address stakingContract_`: The address of the contrtact used for staking StarkNet token.
 *   `address defaultRegistry_`: Address of Gnosis DelegateRegistry.
 *   `address recipient_`: The owner of the grant.
 *   `uint256 grantAmount_`: The amount of tokens granted in this grant.
 *   `uint256 startTime`: The grant time-lock start timestamp.
 *
 *   Token Gradual Release behavior:
 *   ==============================
 *   - Until the global timelock expires all the tokens are locked.
 *   - After the expiration of the global timelock tokens are gradually unlocked.
 *   - The amount of token unlocked is proportional to the time passed from startTime.
 *   - The grant is fully unlocked in 4 years.
 *   - to sum it up:
 *   ```
 *     // 0 <= elapsedTime <= 4_YEARS
 *     elapsedTime = min(4_YEARS, max(0, currentTime - startTime))
 *     unlocked = globalTimelockExpired ? grantAmount * (elapsedTime / 4_YEARS): 0;
 *   ```
 *   - If the total balance of the grant address is larger than `grantAmount` - then the extra
 *     tokens on top of the grantAmount is available for release ONLY after the grant is fully
 *     unlocked.
 *
 *   Global Time Lock:
 *   ================
 *   StarkNet token has a global timelock. Before that timelock expires, all the tokens in the grant
 *   are fully locked. The global timelock can be modified post-deployment (to some extent).
 *   Therefore, the lock is maintained on a different contract that is centralized, and serves
 *   as a "timelock oracle" for all the {LockedTokenGrant} instances. I.e. whenever an instance of this
 *   contract needs to calculate the available tokens, it checks on the {LockedTokenCommon} contract
 *   if the global lock expired. See {LockedTokenCommon} for addtional details on the global timelock.
 *
 *   Token Release Operation:
 *   ======================
 *   - Tokens are owned by the `recipient`. They cannot be revoked.
 *   - At any given time the recipient can release any amount of tokens
 *     as long as the specified amount is available for release.
 *   - The amount of tokens available for release is the following:
 *   ```
 *   availableAmount = min(token.balanceOf(this), (unlocked - alreadyReleased));
 *   ```
 *     The `min` is used here, because a part of the grant balance might be staked.
 *   - Only the recipient or an appointed {LOCKED_TOKEN_RELEASE_AGENT} are allowed to trigger
 *     release of tokens.
 *   - The released tokens can be transferred ONLY to the recipient address.
 *
 *
 *   XXX: following currently not implemented
 *
 *   Appointing agents for actions:
 *   ========================
 *   Certain activities on this contract can be done not only by the grant recipient, but also by a delegate,
 *   appointed by the recipient.
 *   The delegation is done on a Gnosis DelegateRegistry contract, that was given to this contract
 *   in construction. The address of the {DelegateRegistry} is stored in the public variable named
 *   `defaultRegistry`.
 *   1. The function `releaseTokens` can be called by the account (we use the term agent for this) whose address
 *      was delegated for this ID:
 *      0x07238b05622b6f7e824800927d4f7786fca234153c28aeae2fa6fad5361ef6e7 [= keccak(text="LOCKED_TOKEN_RELEASE_AGENT")]
 *   2. The functions `setDelegate` `clearDelegate` `setDelegationOnToken` `setDelegationOnStaking` can be called
 *      by the agent whose address was delegated for this ID:
 *      0x477b64bf0d3f527eb7f7efeb334cf2ba231a93256d546759ad12a5add2734fb1 [= keccak(text="LOCKED_TOKEN_DELEGATION_AGENT")]
 *
 *   Staking:
 *   =======
 *   Staking of StarkNet tokens are exempted from the lock. I.e. Tokens from the locked grant
 *   can be staked, even up to the full grant amount, at any given time.
 *   However, the exect interface of the staking contract is not finalized yet.
 *   Therefore, the {LockedTokenGrant} way support staking is by a dedicated approval function `approveForStaking`.
 *   This function can be called only the recipient, and sets the allowace to the specified amount on the staking contract.
 *   This function is limited such that it approves only the staking contract, and no other address.
 *   The staking contract will support staking from a {LockedTokenGrant} using a dedicated API.
 *
 *   Voting Delegation:
 *   =================
 *   The {LockedTokenGrant} suports both Compound like delegation and delegation using Gnosis DelegateRegistry.
 *   These functions set the delegation of the Grant address (the address of the grant contract).
 *   Only the recipient and a LOCKED_TOKEN_DELEGATION_AGENT (if appointed) can call these functions.
 */
error Unauthorized();

contract LockedTokenGrant is TimeLockedTokens {
    // contract LockedTokenGrant is TimeLockedTokens, DelegationSupport {
    address public immutable token;

    address public immutable recipient;

    address public immutable stakingContract;

    uint256 public releasedTokens;

    event TokensSentToRecipient(
        address indexed recipient, address indexed grantContract, uint256 amountSent, uint256 aggregateSent
    );

    event EarlyTokenRelease(address indexed recipient, address indexed grantContract, uint256 amount, uint256 penalty);

    event TokenAllowanceForStaking(
        address indexed grantContract, address indexed stakingContract, uint256 allowanceSet
    );

    constructor(
        address token_,
        address stakingContract_,
        address penaltyManager_,
        address recipient_,
        uint256 grantAmount_,
        uint256 startTime_,
        uint256 endTime_
    )
        // DelegationSupport(defaultRegistry_, recipient_, address(token_), stakingContract_)
        TimeLockedTokens(penaltyManager_, grantAmount_, startTime_, endTime_)
    {
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

    /*
      Sets the allowance of the staking contract address to `approvedAmount`.
      to allow staking up to that amount of tokens.
    */
    function approveForStaking(uint256 approvedAmount) external {
        if (msg.sender != recipient) revert Unauthorized();

        IERC20(token).approve(stakingContract, approvedAmount);
        emit TokenAllowanceForStaking(address(this), stakingContract, approvedAmount);
    }

    function stake(uint256 amount_) external {
        if (msg.sender != recipient) revert Unauthorized();
        require(amount_ <= grantAmount, "AMOUNT_TOO_BIG");

        IERC20(token).approve(stakingContract, amount_);

        IERC4626(stakingContract).deposit(amount_, address(this));
    }

    function unstake() external {
        if (msg.sender != recipient) revert Unauthorized();

        IERC4626(stakingContract).redeem(IERC20(stakingContract).balanceOf(address(this)), address(this), address(this));
    }
}
