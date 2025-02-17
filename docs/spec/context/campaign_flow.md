# Campaign Flow

This is a draft and needs at least another review

This specification details the full process of executing a campaign from the perspective of a user interacting with the Filament Hub system. The system comprises several components—Outpost, Control, Hub, and Relayer—that work together to facilitate campaign execution, voting, and reward distribution. Communication between these components is synchronized by Relayers to ensure consistency and reliability.

---

## Components

1. **Outpost**: A smart contract that handles payments, including reward distribution and commission payments.
2. **Control**: A staking contract that manages staking operations, delegate bonding, and ensures security through bonds.
3. **Hub**: The coordination point for campaigns, managing campaign initialization, voting processes, and storing campaign-related data structures.
4. **Relayer**: An off-chain service that synchronizes state between Outpost, Control, and Hub, ensuring consistent communication and atomic transactions.

---

## Data Structures

1. **[Hub] VoteSet{}**
    - **Purpose**: Stores votes per campaign.
    - **Contents**:
        - Campaign ID
        - Delegate ID
        - Vote data (e.g., choices, scores)
        - Timestamp
    - **Usage**: Used to calculate the final distribution and outcome of the campaign.
2. **[Control] Staking{}**
    - **Purpose**: Manages staking weights for delegates.
    - **Contents**:
        - Delegate addresses
        - Staked amounts
        - Voting power
        - Bonded amounts
    - **Usage**: Updates when delegates stake, unstake, or redelegate tokens.
3. **[Hub] CampaignDelegates{}**
    - **Purpose**: Stores the set of delegates elected for a specific campaign.
    - **Contents**:
        - Campaign ID
        - Delegate IDs
        - Relative voting power
    - **Usage**: Used during the voting phase to validate votes and calculate results.

---

## Proxied Methods

Methods involving token transfers must go through the base layer (Outpost or Control) that can effectuate payments. The Relayer synchronizes the state of those payments with a two-sided commit to ensure atomicity and consistency.

1. **[Control] Bond(amount)**
    - **Action**: Bonds tokens for campaigners.
    - **Purpose**: Ensures campaigners are committed and can cover delegate payments if they abandon the campaign.
    - **Constraints**:
        - Bonds are locked until campaign settlement.
        - Minimum bond amount may be required.
2. **[Control] Delegate(stakeAmount, delegateAddress)**
    - **Action**: Users delegate their staked tokens to delegates.
    - **Purpose**: Increases delegate's voting power.
3. **[Control] Undelegate(amount, delegateAddress)**
    - **Action**: Users withdraw their delegation.
    - **Purpose**: Adjusts voting power accordingly.
    - **Constraints**:
        - May involve an unbonding period.
4. **[Control] Redelegate(amount, fromDelegate, toDelegate)**
    - **Action**: Users move their delegation from one delegate to another.
    - **Purpose**: Allows flexibility in supporting delegates.
5. **[Control] PayCommission(paymentDetails)**
    - **Action**: Processes commission payments to delegates.
    - **Purpose**: Compensates delegates for their participation.
6. **[Outpost] PayRewards(paymentDetails)**
    - **Action**: Distributes rewards to participants based on campaign results.
    - **Purpose**: Ensures users receive their earned rewards.

---

## Protocol Steps

### 1. Pre-Campaign Phase

### a. Campaign Bonding

- **Action**: Campaigner calls `[Control] Bond(amount)` to bond tokens.
- **Purpose**: Provides security that funds are available for delegate payments.
- **Requirements**:
    - The bonded amount must meet the minimum required for the campaign's budget.
    - Bonds are locked and cannot be withdrawn until the campaign concludes.
    - Bond adjustments (increases) may be required if campaign parameters change.

### b. Staking and Delegation

- **Actions**:
    - Delegates and users perform `[Control] Delegate()`, `[Control] Undelegate()`, and `[Control] Redelegate()`.
- **Purpose**: Adjusts the voting power of delegates for upcoming campaigns.
- **Requirements**:
    - Staking changes affect future campaigns unless a snapshot has already been taken.
    - Staking operations must comply with any lock-up periods or minimum amounts.

---

### 2. Campaign Initialization

### a. Campaign Creation

- **Action**: Campaigner calls `[Hub] Campaign#Init(campaignDetails, proposedDelegates)`.
- **Purpose**: Initializes the campaign with specified parameters and proposed delegates.
- **Requirements**:
    - Campaign details include budget, objectives, timeline, and delegate information.
    - Proposed delegates must be valid and meet eligibility criteria.
    - The campaigner must have sufficient bonded tokens.

### b. Confirmation and Snapshot

- **Upon Confirmation**:
    - **Snapshot Staking Weights**:
        - The system takes a snapshot of current staking to freeze voting power for the campaign.
        - Stores this in `[Hub] CampaignVotingPowers`.
    - **Store Elected Delegates**:
        - Adds delegates to `[Hub] CampaignDelegates{}`.
    - **Adjust Campaign Bond**:
        - If necessary, increases the campaigner's bond to cover campaign obligations.

---

### 3. Voting Phase

### a. Delegate Voting

- **Action**: Delegates call `[Hub] Vote(campaignID, voteData)` to cast votes.
- **Purpose**: Allows delegates to influence the campaign outcome based on their voting power.
- **Requirements**:
    - Only delegates in `[Hub] CampaignDelegates{}` can vote.
    - Votes must be cast within the campaign's voting period.
    - Voting power is based on the snapshot taken at campaign initiation.

### b. Vote Recording and Validation

- **Process**:
    - Votes are recorded in `[Hub] VoteSet{}`.
    - The system validates votes for authenticity and compliance.
- **Requirements**:
    - Duplicate or fraudulent votes are rejected.
    - Voting data is immutable once recorded.

---

### 4. Settlement Phase

### a. Finalizing Distribution

- **Pre-condition**: Voting period has ended, and votes are tallied.
- **Action**: System calculates the final distribution using votes in `[Hub] VoteSet{}`.
- **Purpose**: Determines how rewards and commissions are allocated.
- **Requirements**:
    - Distribution must adhere to campaign rules.
    - Any disputes are resolved before proceeding.

### b. Commission Calculation

- **Function**: `calculate_commission(distribution) -> map[delegate](amount)`
- **Purpose**: Computes commission for each delegate.
- **Requirements**:
    - Commission rates may be fixed or variable.
    - Calculation must be transparent.

### c. Paying Commissions

- **Action**: `[Control] PayCommission(paymentDetails)`
- **Purpose**: Processes commission payments through the Control contract.
- **Requirements**:
    - Payments are proxied for security.
    - Relayer ensures synchronization.

### d. Reward Distribution

- **Action**: `[Outpost] PayRewards(paymentDetails)`
- **Purpose**: Distributes rewards to participants.
- **Requirements**:
    - Payments processed securely via Outpost.
    - Relayer updates state across components.

### e. Releasing Campaign Bond

- **Action**: Campaigner's bond is released upon successful settlement.
- **Requirements**:
    - All payments have been made.
    - No outstanding disputes.
    - If obligations were unmet, bond may be forfeited.

---

### 5. Post-Campaign Phase

### a. State Updates

- **Action**: Update states in `[Control]`, `[Hub]`, and `[Outpost]` to reflect campaign completion.
- **Requirements**:
    - Delegates' voting power remains unless staking changes occur.
    - Campaign records are archived.

### b. Reporting and Analytics

- **Action**: Generate reports on campaign outcomes and performance.
- **Purpose**: Provides transparency.
- **Requirements**:
    - Accessible to users and stakeholders.
    - Data must be accurate.

---

## Queries

### 1. Get Voting Power

- **Action**: `[Hub] GetVotingPower(campaignID, delegateID)`
- **Purpose**: Retrieves a delegate's voting power for a campaign.
- **Requirements**:
    - Based on snapshot in `[Hub] CampaignVotingPowers`.

### 2. Get Campaigns

- **Action**: `[Hub] GetCampaigns(filterCriteria)`
- **Purpose**: Retrieves campaigns based on criteria.
- **Requirements**:
    - Supports filtering by status, creator, etc.
    - Provides campaign details.

### 3. Get Delegate Information

- **Action**: `[Control] GetDelegateInfo(delegateID)`
- **Purpose**: Retrieves delegate's staking and participation info.
- **Requirements**:
    - Includes staked amounts and delegations.

---

## Additional Behaviors and Requirements

### Synchronization and Atomicity

- **Relayer Role**:
    - Synchronizes state between components.
    - Uses a two-phase commit protocol.
    - Ensures atomic transactions.
- **Atomic Operations**:
    - State changes must be all-or-nothing.
    - Rollback on failure to maintain consistency.

### Security and Compliance

- **Authentication and Authorization**:
    - Only authorized users can perform certain actions.
    - Delegates and campaigners must meet eligibility criteria.
- **Slashing and Penalties**:
    - **Conditions**:
        - Misbehavior or failure to meet obligations.
    - **Actions**:
        - Tokens may be slashed from bonds or stakes.
    - **Requirements**:
        - Fair and transparent processes.
        - Dispute resolution mechanisms.

### Failure Handling

- **Campaign Failures**:
    - If a campaign fails (e.g., invalid delegates), the system must:
        - Refund bonds after penalties.
        - Update campaign status.
        - Notify participants.
- **Transaction Failures**:
    - On payment failures:
        - Retry mechanisms.
        - Rollback if necessary.
        - Ensure funds are secure.

### Compliance with Protocols

- **Smart Contract Interactions**:
    - All calls must follow defined interfaces.
    - Proxied methods secure token transfers.
- **Event Logging**:
    - Actions emit events.
    - Used for transparency and auditing.

### User Interface Considerations

- **Notifications**:
    - Inform users of important events.
    - Provide timely updates.
- **Dashboard Access**:
    - Delegates and campaigners have access to performance data.
    - Users can view rewards and participation history.

---

## Conclusion

This specification provides a comprehensive overview of the process for executing a campaign within the Filament Hub system. It covers all components, data structures, methods, protocols, behaviors, and requirements necessary for secure, transparent, and efficient campaign execution.

By following this specification, the Filament Hub ensures that campaigns are conducted fairly, participants are rewarded appropriately, and the system maintains integrity and trust among users.

---

**Note**: This specification assumes familiarity with blockchain concepts, smart contracts, and decentralized systems. Implementers should ensure compliance with relevant legal, regulatory, and technical standards.

---

Let me know if you need further details or clarification on any specific sections!