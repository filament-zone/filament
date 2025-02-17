# Campaign Bonds

Campaign Bonds are a core security mechanism within the Filament protocol. They ensure that Delegates are fairly compensated for their work, even if a Campaigner abandons a campaign or acts maliciously.  This mechanism builds trust and incentivizes responsible behavior from Campaigners.

## Purpose

The primary purpose of Campaign Bonds is to *guarantee* a minimum payment to Delegates who participate in a campaign.  Since Delegates expend effort (and potentially incur opportunity costs) by participating, the bond ensures they receive compensation regardless of the Campaigner's actions. This protection is particularly important because Delegates don't inherently trust Campaigners.

## Key Concepts

*   **Bonding:** The process by which a Campaigner deposits FILA tokens into a smart contract (the `Paymaster`) as a security deposit. This happens *before* the campaign begins.
*   **Locked FILA:**  The bonded FILA is *locked* during the active phases of the campaign.  This means the Campaigner cannot withdraw it until the campaign completes successfully or is settled after a failure.
*   **Watermark:**  A dynamic value representing the *minimum* amount of FILA that must remain locked in the bond. The watermark moves up and down as campaigns progress.  It ensures that enough FILA is always available to cover potential Delegate payments and slashing penalties.
*   **Unbonding:** The process by which a Campaigner can withdraw their FILA *after* a campaign has successfully completed or has been settled.
*   **Slashing:**  If a campaign fails due to Campaigner inaction (e.g., not confirming criteria, not confirming segments) or a timeout, a portion of the bond is *slashed*. This slashed FILA is distributed to the Delegates as compensation.
*   **`MIN_DELEGATE_PAYMENT`:**  A fixed amount of FILA that represents the minimum payment each Delegate will receive, even if the campaign is abandoned. This value is a protocol-wide parameter (set by governance).
* `NUM_DELEGATES`: The number of delegates participating in that campaign.

## How it Works

1.  **Bond Calculation:** Before a campaign starts, the required bond amount is calculated:

    ```rust,ignore
    required_bond = NUM_DELEGATES * MIN_DELEGATE_PAYMENT
    ```

2.  **Bonding (Locking FILA):** The Campaigner must deposit *at least* the `required_bond` amount of FILA into the `Paymaster` contract. This FILA becomes "bonded" and is associated with the specific campaign.  The `Paymaster` contract keeps track of the total bonded FILA and the watermark. When a campaign begins, the `watermark` is increased by the `required_bond`.

3.  **Campaign Progression:** As the campaign progresses through its phases (Init, Criteria, Publish, Distribution, Settle), the bond remains locked.

4.  **Successful Completion:** If the campaign completes successfully (tokens are distributed), the bond (minus any commissions or fees paid) is *unlocked*. The watermark is decreased. The Campaigner can then unbond (withdraw) their FILA.

5.  **Campaign Failure/Timeout:** If the campaign fails due to Campaigner inaction or a timeout at any stage *before* settlement:
    *   The bond is *slashed*. The amount slashed is typically equal to the `required_bond` ( `NUM_DELEGATES * MIN_DELEGATE_PAYMENT` ).
    *   The slashed FILA is distributed to the Delegates, proportional to their voting power (in the full protocol).  This ensures they are compensated for their time and effort.
    *   The watermark is decreased.

6. **Unbonding:** After the campaign has concluded successfully *or* after a failure and slashing, and once all payments have been processed, the campaigner may be able to unbond *remaining* FILA. If they unbond FILA that brings their total bond amount *below* the watermark, they may incur an "unbonding tax" (this detail needs further clarification in the original documentation).

## Example Scenario

Let's say:

*   `MIN_DELEGATE_PAYMENT` = 100 FILA
*   `NUM_DELEGATES` = 10

The `required_bond` would be 10 * 100 = 1000 FILA.

1.  The Campaigner bonds 1000 FILA. The `watermark` increases by 1000.
2.  The campaign proceeds to the Criteria phase.
3.  The Campaigner *fails* to confirm the criteria within the allowed time.
4.  The bond is slashed by 1000 FILA. The `watermark` is decreased by 1000.
5.  Each Delegate receives 100 FILA (their minimum payment).

## Relationship with Paymaster

The `Paymaster` contract is responsible for:

*   Holding the bonded FILA.
*   Tracking the `watermark`.
*   Executing the locking, unlocking, and slashing of bonds.
*   Facilitating payments to Delegates (and potentially other actors).

The Filament Hub interacts with the `Paymaster` to manage the campaign bonds.

## Code Example (Conceptual - Combining Rust and Solidity Ideas)

```rust,ignore
// Simplified Rust representation (Conceptual)
struct CampaignBond {
    total_bonded_fila: u256, // Total FILA bonded by all Campaigners
    watermark: u256,          // Minimum amount that must remain locked
    min_delegate_payment: u256,
    campaign_bonds: HashMap<u64, CampaignSpecificBond>, // Campaign ID -> Bond details
}

struct CampaignSpecificBond {
  campaign_id: u64,
	campaigner: Address,
  num_delegates: u32,
	locked_bond: u256,
}

impl CampaignBond {
    // ... (Functions for bonding, locking, unlocking, slashing)
}

// --- Solidity (Paymaster - Partial) ---
pragma solidity ^0.8.0;

contract Paymaster {
    // ... (Other parts of the Paymaster)

    struct CampaignBond {
        uint256 balance;
        uint256 watermark;
    }

    mapping(address => mapping(uint256 => CampaignBond)) public campaignBonds; // Campaigner -> CampaignID -> Bond
		// ...
		// Add to bond
    function addToCampaignBond(uint256 campaignId, uint256 amount) public {
        filaToken.safeTransferFrom(msg.sender, address(this), amount);
        campaignBonds[msg.sender][campaignId].balance += amount;
    }
		// set watermark
    function setCampaignBondWatermark(uint256 campaignId, uint256 watermark) public {
        // Add appropriate access control here
        campaignBonds[msg.sender][campaignId].watermark = watermark;
    }
		// withdraw from bond
    function withdrawFromCampaignBond(uint256 campaignId, uint256 amount) public {
        CampaignBond storage bond = campaignBonds[msg.sender][campaignId];
        require(bond.balance - amount >= bond.watermark, "Cannot withdraw below watermark");
        bond.balance -= amount;
        filaToken.safeTransfer(msg.sender, amount);
    }
}
```

## Benefits

*   **Trustless System:**  Delegates don't need to trust Campaigners directly. The bond mechanism ensures they will be paid.
*   **Incentivizes Completion:** Campaigners are incentivized to complete campaigns successfully to avoid losing their bond.
*   **Fair Compensation:**  Delegates are guaranteed a minimum payment for their work.
*   **Sybil Resistance:**  The bond requirement makes it more expensive for malicious actors to create many fake campaigns.

## Conclusion

Campaign Bonds are a fundamental part of the Filament protocol's security and incentive model. They provide a robust mechanism for ensuring that Delegates are compensated for their contributions, fostering a trustworthy and reliable environment for decentralized token distributions. The interaction with the `Paymaster` contract is crucial for the secure management of these bonds.
