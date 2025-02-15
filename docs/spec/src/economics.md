# Economics

The Filament protocol's economic model is designed to incentivize participation, ensure the security and reliability of the system, and align the interests of all actors: Campaigners, Delegates, Indexers, and Participants. This section provides an overview of the key economic components and mechanisms within Filament.

## Core Principles

*   **Incentivized Participation:** All actors in the system are economically incentivized to contribute positively. Delegates and Indexers are rewarded for their work, and Campaigners benefit from successful token distributions.
*   **Security through Staking:**  FILA, the native token, is used for staking. Staking provides economic security, as misbehavior can result in slashing (loss of staked tokens).
*   **Sybil Resistance:**  FILUM, a non-transferable gas token, prevents spam and ensures fair access to the Filament Hub's computational resources. Campaign bonds also provide Sybil resistance at the campaign level.
*   **Sustainable Operations:**  The system is designed to be self-sustaining, with fees and commissions providing ongoing funding for operations and development.
*   **Price Stability for Campaigners**: The Treasury Window provides a predictable price for acquiring FILA, protecting Campaigners for volatility.

## Key Economic Components

The Filament economic model comprises the following key elements:

### 1. $FILA Token

FILA is the native token of the Filament network. It serves multiple critical roles:

*   **Staking:** Delegates stake FILA to gain voting power and participate in campaigns.  Their rewards are proportional to their stake.
*   **Bonding:** Campaigners can use FILA for campaign bonds, providing a security deposit that guarantees delegate payments even if the campaign is abandoned.
*   **Governance:**  FILA may be used for governance of the Filament protocol itself (details to be determined).
* **Medium of Exchange**: Payment for Indexers and Delegates services.

[Learn more about the FILA Token](./economics/fila_token.md)

### 2. $FILUM (Gas Token)

FILUM is a non-transferable gas token used to pay for computation on the Filament Hub.  It provides Sybil resistance and prevents spam.

*   **Non-Transferable:** FILUM cannot be bought, sold, or transferred between users.
*   **Periodic Allocation:** Users with a minimum FILA bond receive a periodic allocation of FILUM.
*   **Usage-Based Consumption:**  FILUM is consumed when interacting with the Filament Hub (e.g., creating campaigns, voting, submitting data).
* **Prevents Spam**: By requiring FILUM, the protocol avoids spam by malicious actors who would have unlimited requests.

[Learn more about FILUM](./economics/filum.md)

### 3. Campaign Bonds

Campaign Bonds are a crucial security mechanism in Filament.  They ensure that Delegates are compensated for their work, even if a Campaigner abandons a campaign.

*   **Security Deposit:** Campaigners deposit FILA as a bond before starting a campaign.
*   **Delegate Payment Guarantee:** The bond guarantees a minimum payment to each Delegate, even if the campaign fails.
*   **Slashing:**  If a campaign fails due to Campaigner inaction (e.g., not confirming criteria), a portion of the bond may be slashed and distributed to Delegates.
*   **Watermark:** A mechanism to ensure sufficient funds remain locked in the bond.

[Learn more about Campaign Bonds](./economics/campaign_bonds.md)

### 4. Commission Structure

The Commission Structure defines how Delegates are rewarded for their participation in campaigns.

*   **Proportional to Voting Power:** Delegates receive a commission based on their voting power (which is proportional to their staked FILA).
*   **Incentivizes Participation:**  Delegates must actively participate (vote) to be eligible for rewards.
*   **Delegator Rewards:**  A portion of the Delegate's commission is distributed to users who have delegated their FILA to that Delegate.
* **Proposals:** Delegates who provide winning proposals may earn additional rewards.

[Learn more about the Commission Structure](./economics/commission_structure.md)

### 5. Treasury Window

The Treasury Window provides a mechanism for Campaigners to acquire FILA at a predictable price, even if market liquidity is low.

*   **Stable Price:**  The Treasury Window offers FILA at a controlled exchange rate (likely based on a time-weighted average price, TWAP, plus a discount).
*   **USDC Exchange:** Campaigners can purchase FILA using USDC.
*   **Controlled Supply:**  The Treasury Window helps manage the supply of FILA in circulation.
* **Discount**: The Treasury Window provides a discount on FILA, incentivizing campaign creation.

[Learn more about the Treasury Window](./economics/treasury_window.md)

## Interactions and Flows

These economic components interact in various ways:

1.  **Staking:** Users stake FILA to become Delegates or to delegate to existing Delegates.
2.  **Campaign Creation:** Campaigners bond FILA (potentially acquired through the Treasury Window) to create a campaign.
3.  **Voting:** Delegates use their voting power (proportional to their stake) to vote on criteria and distributions.
4.  **Commission Distribution:** Delegates (and their delegators) receive commissions based on their participation and voting power.
5.  **Gas Consumption:** Users consume FILUM when interacting with the Filament Hub.
6.  **Slashing:**  Campaigners, Delegates and Indexers may be slashed for misbehavior or inaction.

This economic model is designed to create a robust and self-sustaining ecosystem for decentralized token distributions. The interplay of these mechanisms ensures that all actors are incentivized to act in the best interests of the network and its users.
