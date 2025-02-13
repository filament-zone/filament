# Campaign Protocol

The Campaign Protocol defines how Campaigners, Delegates, and Indexers collaborate to execute campaigns through a series of well-defined phases. Each phase has specific rules, requirements, and state transitions that ensure secure and predictable campaign execution.

## Campaign Lifecycle

A campaign progresses through five distinct phases:

1. **Init** - Campaign setup and delegate election
2. **Criteria** - Collaborative development of distribution criteria
3. **Publish** - Data collection and validation
4. **Distribution** - Reward allocation decisions
5. **Settle** - Final distribution and campaign completion

Each phase must complete successfully or timeout before moving to the next phase. The protocol ensures campaigns always terminate in the Settle phase, even in failure cases.

## Phase Details

**1. Init Phase**

*   **Purpose:**  To set up the campaign, define the budget, and select the initial set of Delegates.
*   **Actions:**
    *   Campaigner creates a campaign, providing:
        *   `budget`: The total amount of tokens to be distributed.
        *   `initial_criteria`:  A *suggestion* for the criteria (this is *not* binding).  This helps inform the initial delegate selection.
        *   `delegate_candidates`: A list of potential delegates.
    *   The Protocol (Filament Hub) selects a subset of `delegate_candidates` to be the Delegates for this campaign. The selection process might involve:
        *   Staking weight: Delegates with more FILA staked have a higher chance of being selected.
        *   Reputation: Delegates with a good track record might be preferred.
        *   Randomness: To prevent bias.
    *   The Campaigner *confirms* the selected Delegates.  They can choose to *evict* a limited number of Delegates (paying a fee for each eviction). This allows the Campaigner to have *some* control over the Delegate set.
*   **State Transitions:**

    ```rust,ignore
    // Campaigner initiates the campaign
    #[authorized(Campaigner)]
    fn init_campaign(budget: Budget, initial_criteria: Criteria, delegate_candidates: Vec<Address>) -> Result<Campaign, Error> {
        // 1. Validate inputs (e.g., sufficient budget, valid addresses)
        // 2. Select Delegates (using staking weight, reputation, randomness)
        // 3. Create Campaign struct
        // 4. Store Campaign in the Hub's state
        // 5. Lock the budget (transfer tokens to the Hub or an escrow contract)
        // 6. Return the new Campaign
    }

    // Campaigner confirms the selected Delegates, potentially evicting some
    #[authorized(Campaigner)]
    fn confirm_delegates(campaign_id: u64, delegates_to_evict: Vec<Address>) -> Result<(), Error> {
        // 1. Verify that the Campaigner owns the campaign
        // 2. Verify that the number of evictions is within the allowed limit (MAX_EVICTIONS)
        // 3. Calculate the eviction cost (MIN_DELEGATE_PAYMENT * number of evictions)
        // 4. Deduct the eviction cost from the Campaigner's bond or balance
        // 5. Pay the evicted Delegates
        // 6. Update the Campaign's delegate list
        // 7. Start the Criteria phase timer
    }
    ```

*   **Failure Modes:**
    *   Insufficient budget.
    *   Invalid delegate candidates.
    *   Campaigner tries to evict too many Delegates.
    *   Campaigner doesn't have enough funds to pay eviction fees.
*   **Timeout Logic:**  There might be a timeout for the `confirm_delegates` step. If the Campaigner doesn't confirm within the time limit, the campaign could be aborted, and the budget (minus any applicable fees) returned.

**2. Criteria Phase**

*   **Purpose:**  To determine the *rules* for the distribution.  This is where Delegates use their expertise to decide *who* should receive tokens and *why*.
*   **Actions:**
    *   Delegates propose and vote on `Criteria`.  A `Criterion` might be something like:
        *   "Users who have held at least 100 UNI tokens for 6 months."
        *   "Users who have participated in at least 3 governance proposals on Compound."
        *   "Users who have a Gitcoin Grants score above 50."
    *   Delegates can propose different sets of `Criteria`.
    *   Voting is weighted by the Delegates' staking power (how much FILA they have staked).
    *   A `Tally` object tracks the votes for each proposal.  A proposal needs to reach a certain `quorum` (minimum voting power) to be considered valid.
    *   The Campaigner can *confirm* a proposal that has reached quorum.  They can also provide `variables` to parameterize the criteria (e.g., the specific date for a snapshot).
    *   The Campaigner can *reject* the proposals. This leads to the `Settle` phase (and likely some penalties).
*   **State Transitions:**

    ```rust,ignore
    // Delegate proposes a set of Criteria
    #[authorized(Delegate)]
    fn propose_criteria(campaign_id: u64, criteria: Criteria) -> Result<u64, Error> { // Returns proposal ID
        // 1. Verify that the Delegate is part of the campaign
        // 2. Create a new Proposal
        // 3. Add the Proposal to the Tally
        // 4. Return the Proposal ID
    }

    // Delegate votes on a proposal
    #[authorized(Delegate)]
    fn vote(campaign_id: u64, proposal_id: u64, choice: bool) -> Result<(), Error> {
        // 1. Verify that the Delegate is part of the campaign
        // 2. Verify that the proposal exists
        // 3. Get the Delegate's voting power
        // 4. Update the Tally with the vote
    }

    // Campaigner confirms a set of Criteria that has reached quorum
    #[authorized(Campaigner)]
    fn confirm_criteria(campaign_id: u64, proposal_id: u64, variables: HashMap<String, Value>) -> Result<(), Error> {
        // 1. Verify that the Campaigner owns the campaign
        // 2. Verify that the proposal has reached quorum
        // 3. Set the Campaign's criteria to the chosen proposal's criteria
        // 4. Set campaign variables
        // 5. Transition to the Publish phase
        // 6. Start Publish phase timer
    }
    // Campaigner rejects
    #[authorized(Campaigner)]
    fn reject_criteria(campaign_id: u64)-> Result<(),Error>{
        // 1. Verify that the Campaigner owns the campaign
        // 2. Transition to settle phase
    }

    // Called by the Protocol when the Criteria phase timer expires
    #[authorized(Protocol)]
    fn timeout_criteria(campaign_id: u64) -> Result<(), Error> {
      // 1. Slash Delegates that did not vote
      // 2. Transition to the settle phase.
    }
    ```

*   **Failure Modes:**
    *   No proposal reaches quorum.
    *   The Campaigner rejects all proposals.
    *   The Criteria phase timer expires.
*   **Timeout Logic:**  If the Criteria phase times out, Delegates who *didn't* vote might be slashed, and the campaign transitions to the `Settle` phase.

**3. Publish Phase**

*   **Purpose:**  To gather the *data* that matches the chosen `Criteria`. This is where `Indexers` come in.
*   **Actions:**
    *   Based on the confirmed `Criteria`, the Protocol assigns specific data-gathering tasks to `Indexers`.  For example, one Indexer might be responsible for gathering data about Uniswap holders, while another might focus on Compound governance participants.
    *   Indexers submit `Segments`.  A `Segment` is a list of addresses (and associated data) that meet a specific `Criterion`.
    *   Multiple Indexers might submit competing `Segments` for the same `Criterion`.
    *   The Campaigner *confirms* the `Segments` they believe are correct. This involves resolving any disputes between competing Indexers.
*   **State Transitions:**

    ```rust,ignore
    // Indexer publishes a Segment
    #[authorized(Indexer)]
    fn publish_segment(campaign_id: u64, criterion_index: u64, segment: HashMap<Address, u256>) -> Result<(), Error> {
        // 1. Verify that the Indexer is assigned to this campaign and criterion
        // 2. Verify the segment data (e.g., check for duplicates, invalid addresses)
        // 3. Store the Segment
    }

    // Campaigner confirms the segments
    #[authorized(Campaigner)]
    fn confirm_segments(campaign_id: u64, resolution: HashMap<u64, Address>) -> Result<(), Error>{ // criterion index -> selected indexer address
        // 1. Verify the Campaigner
        // 2. Verify segment choices (no duplicates, valid indexers)
        // 3. Transition to the distribution phase.
    }
        // Called by the Protocol when the Publish phase timer expires
    #[authorized(Protocol)]
    fn timeout_publish(campaign_id: u64) -> Result<(), Error> {
        // 1. Slash indexers that did not submit their assigned work.
        // 2. Transition to the settle phase.
    }

    ```

*   **Failure Modes:**
    *   Indexers fail to submit their assigned `Segments`.
    *   The Campaigner doesn't confirm the `Segments`.
    *   The Publish phase timer expires.
*   **Timeout Logic:** If the Publish phase times out, Indexers who didn't submit their assigned data might be slashed, and the campaign transitions to the `Settle` phase.

**4. Distribution Phase**

*   **Purpose:** To determine the *final* allocation of tokens to participants, based on the confirmed `Criteria` and `Segments`.
*   **Actions:**
    *   The Protocol calculates a *proposed* `Distribution` based on the confirmed `Criteria` and `Segments`.  This is a mapping of addresses to token amounts.
    *   Delegates vote on the proposed `Distribution`.  They can also propose *alternative* distributions.
    *   The Campaigner *confirms* a `Distribution` that has reached quorum.
    *   The Campaigner can *reject* the proposed Distribution. This leads to the Settle phase.
*   **State Transitions:**

    ```rust,ignore
    // Delegate votes on the proposed Distribution
    #[authorized(Delegate)]
    fn vote_distribution(campaign_id: u64, proposal_id: u64, choice: bool) -> Result<(), Error> {
        // Similar to the vote function in the Criteria phase
    }
    // Delegate propose a set of Distributions
    #[authorized(Delegate)]
    fn propose_distribution(campaign_id: u64, distribution: Distribution) -> Result<u64, Error> { // Returns proposal ID
        // 1. Verify that the Delegate is part of the campaign
        // 2. Create a new Proposal
        // 3. Add the Proposal to the Tally
        // 4. Return the Proposal ID
    }

    // Campaigner confirms a Distribution
    #[authorized(Campaigner)]
    fn confirm_distribution(campaign_id: u64, proposal_id: u64) -> Result<(), Error> {
        // 1. Verify Campaigner
        // 2. Verify that the proposal has reached quorum
        // 3. Set the Campaign's distribution
        // 4. Transition to the Settle phase
    }

    // Campaigner rejects
    #[authorized(Campaigner)]
    fn reject_distribution(campaign_id: u64)-> Result<(),Error>{
        // 1. Verify that the Campaigner owns the campaign
        // 2. Transition to settle phase
    }
        // Called by the Protocol when the Distribution phase timer expires
    #[authorized(Protocol)]
    fn timeout_distribution(campaign_id: u64) -> Result<(), Error> {
      // 1. Slash Delegates that did not vote
      // 2. Transition to the settle phase.
    }

    ```

*   **Failure Modes:**
    *   No proposal reaches quorum.
    *   The Campaigner rejects the proposed `Distribution`.
    *   The Distribution phase timer expires.
*   **Timeout Logic:** If the Distribution phase times out, Delegates who didn't vote might be slashed, and the campaign transitions to the `Settle` phase.

**5. Settle Phase**

*   **Purpose:** To finalize the campaign, distribute the tokens, and release any remaining resources.
*   **Actions:**
    *   The Protocol makes payments to:
        *   Participants (according to the confirmed `Distribution`).
        *   Delegates (their commission).
        *   Indexers (their payment for providing data).
    *   Any remaining funds from the budget (e.g., if the distribution didn't use the entire budget) are returned to the Campaigner.
    *   The Campaigner's bond is released (if they had to post a bond).
    *   The campaign is marked as complete.
* **State Transitions**
    ```rust,ignore
    // This function might be called automatically by the Protocol after the Distribution phase
    fn settle_campaign(campaign_id: u64) -> Result<(), Error> {
        // 1. Verify that the campaign is in the Distribution phase
        // 2. Make payments to Participants, Delegates, and Indexers
        // 3. Return any remaining funds to the Campaigner
        // 4. Release the Campaigner's bond
        // 5. Mark the campaign as complete
    }
    // The state machine receives settlement messages confirming that payments have been processed
    #[authorize(Protocol)]
    fn clear(campaign_id: u64, payment: Payment) -> Result<(),Error>{
        // 1. verify payment information
        // 2. update payment status.
    }
    ```
*   **Failure Modes:**
     *   Payment failures (e.g., insufficient funds in the Outpost contract). This is a critical area and needs robust error handling and retry mechanisms.
*   **Timeout Logic:** There might be a timeout for the `Settle` phase, but the primary concern here is ensuring that all payments are made correctly.

**IV. Key Mechanisms**

*   **Voting:** Weighted by staking power.  Delegates with more FILA staked have more influence.
*   **Quorum:** A minimum amount of voting power is required for a proposal to be considered valid. This prevents a small number of Delegates from controlling the outcome.
*   **Slashing:**  Delegates and Indexers can be penalized for misbehavior (e.g., not voting, providing incorrect data).
*   **Timeouts:**  Each phase has a time limit.  If the phase doesn't complete within the time limit, the campaign might be aborted, and penalties might be applied.
*   **Relayers:**  Ensure that state is synchronized between the Filament Hub (on a Layer 2) and Outposts (on Layer 1 chains). This is *crucial* for ensuring that payments are made correctly.
*   **Paymaster:**  A contract that manages payments, potentially using a Campaign Bond and the Treasury Window to provide FILA for campaign operations.

**V. Example Scenario**

Let's say a project called "AwesomeFi" wants to distribute their token, $AWESOME, to users who have provided liquidity to their Uniswap V3 pool.

1.  **Init:** AwesomeFi creates a campaign on the Filament Hub, specifying a budget of 1 million $AWESOME tokens.  They propose some initial criteria (e.g., "users who have provided at least $1000 of liquidity"). A set of Delegates is selected.
2.  **Criteria:** The Delegates refine the criteria.  They might add criteria like "users who have provided liquidity for at least 3 months" or "users who have also staked $AWESOME in our governance contract."  They vote on these criteria, and the winning set of criteria is confirmed by AwesomeFi.
3.  **Publish:**  Indexers gather data about Uniswap V3 liquidity providers and submit `Segments` to the Filament Hub.
4.  **Distribution:**  The Protocol calculates a proposed distribution based on the confirmed criteria and segments.  Delegates vote on this distribution, and AwesomeFi confirms it.
5.  **Settle:**  The 1 million $AWESOME tokens are distributed to the eligible users via an Outpost contract on Ethereum.  Delegates and Indexers receive their payments.
