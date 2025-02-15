# Campaign Protocol V0

## I. Overview

The Filament Campaign Protocol v0 is a mechanism for determining the distribution of tokens in a credible and transaprent way. It allows a *Campaigner* to define a campaign, select *Delegates* to help determine the distribution criteria, and ultimately distribute tokens to *Participants* based on those criteria.  This simplified v0 version focuses on the core state transitions within the Filament Hub, with some external interactions (like Relayer updates of voting power) simplified or omitted for the testnet. The v0 protocol does not actually distribute funds but instead provides a mechanism for determining the distribution with the expectation that the funds are disbursed out of band. Actors participating are not punishable (slashable) and those are expected to act in good faith. Future version (v1) of the protocol provide much stronger guarentees.

The v1 protocol will include:
*   **Full Budget Management:**  Locking, releasing, and using the budget for payments.
*   **Advanced Delegate Selection:** Using staking weight, reputation, and randomness.
*   **Weighted Voting and Quorum:**  Votes will be weighted by staking power, and a quorum will be required for proposals to pass.
*   **Complete Distribution and Settle Phases:**  Implementing the logic for determining the final token distribution and making payments.
*   **Timeouts:** Implementing timeouts for each phase and handling them appropriately.
*   **Paymaster and Outpost Integration:**  Interacting with external contracts for payment management and token distribution.
*   **Full Relayer Functionality:**  Synchronizing state and confirming payments.
*   **Slashing**: Implementing a mechanism to penalize bad actors (Timeouts).

## II. Actors

*   **Campaigner:** The entity initiating and funding the campaign.
*   **Delegates:**  Trusted actors who propose and vote on criteria.
*   **Indexers:** Actors who provide data (segments) based on the criteria.
*   **Relayers:**  Off-chain services that update voting power information.
*   **Protocol (Filament Hub):** The core smart contract managing the campaign lifecycle.

## III. Data Structures

```rust,ignore
struct Campaign {
    id: u64,
    campaigner: S::Address,
    phase: Phase,
    title: String,          // Added for usability
    description: String,    // Added for usability
    criteria: Criteria,     // Initial criteria, *not* optional
    evictions: Vec<Eviction<S>>, // Delegates to be evicted
    delegates: Vec<S::Address>, // List of selected Delegates
    indexer: Option<S::Address>, // Registered indexer for the campaign
}

enum Phase {
    Draft,      // Initial phase after creation
    Criteria,   // Delegates propose and vote on criteria
    Publish,    // Indexers submit segments
    Indexing,   // Indexer is gathering data.
    Distribution, // Segments are available.
    // Settle,  // (Not Implemented) Final phase for distribution and cleanup
}
struct Criteria {
    criteria: Vec<Criterion>,
}

struct Criterion {
    dataset_id: String, // e.g., "historical_transactions", "token_balances"
    parameters: HashMap<String, String>, // e.g., {"min_balance": "100", "token_address": "0x..."}
}

struct Eviction<S: Spec> {
	addr: S::Address
}

struct Segment {
    // ... (Structure defined elsewhere, mapping addresses to data)
}

struct CriteriaProposal<S: Spec> {
    campaign_id: u64,
    proposer: S::Address,
    criteria: Criteria,
}

// Simplified VoteOption (could be more complex in the full protocol)
enum VoteOption {
    Yes,
    No
}

```

## IV. Campaign Phases and State Transitions

**1. Draft Phase**

*   **Entry Condition:**  A Campaigner calls `draft_campaign`.
*   **Purpose:**  To initialize the basic campaign parameters and select an initial set of Delegates.
*   **Actions:**
    *   Campaigner provides:
        *   `title`: A human-readable title for the campaign.
        *   `description`: A description of the campaign's goals.
        *   `criteria`:  The *initial* criteria for the campaign.  **Note:** In the simplified protocol, these criteria are *required*, unlike the full protocol where they are a suggestion.
        *   `evictions`:  A list of Delegates to be evicted.
    *   The Protocol:
        *   Assigns a unique `campaign_id`.
        *   Stores the campaign data.
        *   Selects Delegates (simplified logic in testnet - see "Key Differences").
        *   Removes evicted Delegates from the list.
*   **State Transition Function:**
    ```rust,ignore
    fn draft_campaign(
        title: String,
        description: String,
        criteria: Criteria,
        evictions: Vec<Eviction<S>>,
        sender: &S::Address,
        state: &mut impl TxState<S>,
    ) -> Result<u64, Error>;
    ```

*   **Exit Condition:** Campaigner calls `init_campaign`.
* **Failure Modes:**
    * Invalid criteria.
    * Invalid eviction requests (evicting non-proposed delegates).
    * Campaign ID collision (should be prevented by the `next_campaign_id` counter).

**2. Init Phase**

*   **Entry Condition:** Campaigner calls `init_campaign`.
*   **Purpose:** To formally start the campaign and transition to the Criteria phase.
*   **Actions:**
    *   The Protocol:
        *   Verifies that the caller is the Campaigner.
        *   Verifies that the campaign is in the `Draft` phase.
        *   **TODO:** (Full Protocol) Check and lock the campaign bond.
        *   **TODO:** (Full Protocol) Settle payments for evictions.
*   **State Transition Function:**

    ```rust,ignore
    fn init_campaign(
        campaign_id: u64,
        sender: &S::Address,
        state: &mut impl TxState<S>,
    ) -> Result<(), Error>;
    ```

*   **Exit Condition:**  Successful execution of `init_campaign`.  The campaign's `phase` is set to `Criteria`.
*   **Failure Modes:**
    * Campaign not found.
    * Caller is not the Campaigner.
    * Campaign is not in the `Draft` phase.

**3. Criteria Phase**

*   **Entry Condition:** Campaign is in the `Criteria` phase.
*   **Purpose:**  For Delegates to propose and vote on the criteria for the distribution.
*   **Actions:**
    *   Delegates call `propose_criteria` to submit their proposed criteria.
    *   Delegates call `vote_criteria` to vote on proposals (Yes/No). **Note:** In the v0 protocol, votes are *not* weighted by staking power, and there is no quorum.
    *   Campaigner calls `confirm_criteria` to select the final criteria. **Note:** In the v0 protocol, the Campaigner can choose *any* proposal, regardless of votes.
        *   The Campaigner *may* provide a `proposal_id`.
    * Campaigner calls `reject_criteria` to reject the criteria.
*   **State Transition Functions:**

    ```rust,ignore
    fn propose_criteria(
        campaign_id: u64,
        criteria: Criteria,
        sender: &S::Address,
        state: &mut impl TxState<S>,
    ) -> Result<(), Error>;

    fn vote_criteria(
        campaign_id: u64,
        option: VoteOption,
        sender: &S::Address,
        state: &mut impl TxState<S>,
    ) -> Result<(), Error>;

    fn confirm_criteria(
        campaign_id: u64,
        proposal_id: Option<u64>,
        sender: &S::Address,
        state: &mut impl TxState<S>,
    ) -> Result<(), Error>;

    fn reject_criteria(
      campaign_id: u64,
      sender: &S::Address,
      state: &mut impl TxState<S>,
    ) -> Result<(), Error>;
    ```

*   **Exit Condition:**  Campaigner calls `confirm_criteria` (transition to `Publish` phase) or `reject_criteria` (transition to `Settle`).
*   **Failure Modes:**
    * Campaign not found.
    * Caller is not a Delegate (for `propose_criteria` and `vote_criteria`).
    * Caller is not the Campaigner (for `confirm_criteria` and `reject_criteria`).
    * Campaign is not in the `Criteria` phase.

**4. Publish Phase**

*   **Entry Condition:**  Campaign is in the `Publish` phase.
*   **Purpose:**  For Indexers to submit data (segments) that match the confirmed criteria.
*   **Actions:**
    * The Protocol transitions the campaign to the Indexing Phase.
    *   Indexer calls `post_segment` to submit a segment.
*   **State Transition Functions:**

    ```rust,ignore
    fn index_campaign(
        campaign_id: u64,
        sender: &S::Address,
        state: &mut impl TxState<S>,
    ) -> Result<(), Error>;

    fn post_segment(
        campaign_id: u64,
        segment: Segment,
        sender: &S::Address,
        state: &mut impl TxState<S>,
    ) -> Result<(), Error>;
    ```
*   **Exit Condition:** Indexer calls `post_segment`. The campaign phase is changed to `Distribution`.
*   **Failure Modes:**
    * Caller is not the registered Indexer.
    *   Campaign not found.
    *   Campaign is not in the `Indexing` phase.
    *   Segment already exists.

**5. Distribution Phase**
* **Entry Condition**: Campaign is in the `Distribution` phase.
* **Purpose**: Await for the final settlement to happen and mark the campaign as completed.
* **Note:** The v0 protocol does *not* implement the full Distribution phase logic (voting on distributions, alternative proposals).

**6. Settle Phase**
* **Note:** The v0 protocol does *not* implement the Settle phase.

**Indexer and Relayer Management**

*   The v0 protocol includes functions for registering and unregistering Indexers and Relayers. These are currently restricted to an admin account.
* `register_indexer`, `unregister_indexer`
* `register_relayer`, `unregister_relayer`
* Relayers can call `update_voting_power` to update the voting power of addresses. This is a simplified mechanism for the testnet; the full protocol will involve more robust synchronization with a staking contract.

## V. V1 Key Differences

*   **Budget Handling:** The v0 protocol *does not* fully handle the campaign budget (locking, releasing, using for payments). This is a placeholder and will be fully implemented in the future with interactions with a `Paymaster` contract and `Outpost` contracts.
*   **Delegate Selection:** The initial delegate selection process is simplified. The full protocol will incorporate staking weight, reputation, and randomness.
*   **Eviction Payments:** Payments to evicted delegates are not yet implemented.
*   **Voting Power:** While the system tracks voting power, *votes are not currently weighted by this power*.  Each delegate effectively has one vote. The full protocol will implement weighted voting.
*   **Quorum:** The concept of a *quorum* (minimum voting power needed for a proposal to pass) is *not* implemented in the v0 protocol.
*   **Criteria Confirmation:** The Campaigner can confirm *any* proposal, regardless of votes.  The full protocol will enforce quorum requirements and allow the Campaigner to provide variables.
*   **Publish Phase:** The v0 protocol has basic support for Indexers and segments, but *does not* handle competing segments or disputes.
*   **Distribution Phase:** The Distribution phase (where the final token allocation is determined) is *completely absent* in the v0 protocol.  This is a major future addition.
*   **Settle Phase:** The Settle phase (where tokens are distributed and the campaign is finalized) is *completely absent* in the v0 protocol.
*   **Timeouts:**  Timeouts for each phase are *not* implemented.
*   **Full Relayer Integration:** The Relayer's role is limited to updating voting power.  The full protocol will involve the Relayer in synchronizing state and confirming payments.
*   **Treasury Window:** No interaction with a treasury window for acquiring bonded FILA.
*	 **Slashing**: No mechanism to slash delegators
