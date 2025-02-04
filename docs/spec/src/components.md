# Core Components
* Summary of the main components that make up the Filament System including Hub, Control, Outpost and Relayers

## Filament Hub

### Purpose
The Filament Hub is the central coordination point for the protocol, implemented as a layer 2 state machine optimized for campaign execution. It manages campaign lifecycles, processes delegate votes, and coordinates with outposts through relayers.
  * Describe the prupose of the Filament Hub

1. **Phase Transitions**
   - Enforces phase ordering (Init → Criteria → Publish → Distribution → Settle)
   - Validates state transition requirements
   - Processes timeouts and failures

2. **Voting Management**
   - Tracks delegate votes
   - Calculates quorum
   - Enforces voting rules

3. **Payment Coordination**
   - Queues payments
   - Validates proof of payments
   - Confirms settlement

### State
```rust,ignore
  enum Phase {
    Draftt
    Init,
    Criteria,
    Publish,
    Indexing,
    Distribution,
    Settle,
    Settled,
    Canceled,
    Rejected,
  }

  struct Campaign {
    campaigner: Address,      // Address of the campaigner
    phase: Phase,             // Current campaign phase
    budget: Budget,           // Locked campaign budget
    payments: Vec<Payment>,   // Pending payments
    delegates: Vec<Delegate>, // Elected delegates
    variables: Vec<Variable>, // Campaign parameters
    segments: Vec<Segment>,    // Segments for the campaign
    indexer: Address,          // Indexer responsible for producing Segments

    criteria_votes: Vec<Vote>,     // Delegate votes for the criteria phase
    distribution_votes: Vec<Vote>  // Delegate votes for the distribution phase
  }
  type Segment = Vec<(Address, u64)>
```

### Transaction Types
* draft_campaign
* vote_criteria
* init_campaign
* propose_criteria
* vote_criteria
* confirm_criteria
* reject_criteria
* index_campaign (indexer commitment)
* post_segment (segment)

// per campaign?
* register_indexer
* unregister_indexer

// relayer
register_relayer
deregister_relayer

### RPC
The Filament Hub Core API provides REST endpoints for interacting with campaign-related functionality. The primary endpoint `/campaigns/{campaign_id}` allows clients to retrieve detailed information about a specific campaign by providing its unique identifier.

TODO: Link to API Documentation

Currently, the API supports fetching individual campaign details which includes the campaigner's address. The endpoint returns a 200 status code with campaign data on success, or a 404 status code if the specified campaign is not found. This API serves as a fundamental interface for applications to query campaign state from the Filament Hub.

The endpoint returns a Campaign object containing:
- `id`: A unique identifier for the campaign (uint64)
- `campaigner`: The address of the campaign creator
- `phase`: Current state of the campaign (one of: Draft, Init, Criteria, Publish, Indexing, Distribution, Settle, Settled, Canceled, or Rejected)
- `title`: Campaign name/title
- `description`: Detailed campaign description
- `criteria`: Distribution criteria specifications
- `evictions`: List of evicted delegates
- `delegates`: List of participating delegates
- `indexer`: Optional address of the assigned indexer




## Control
The Filament Control system is the economic backbone of the Filament Hub, managing FILA token staking, delegation, and voting mechanics. At its core, it enables token holders to participate in campaigns through a delegation system that creates economic security while ensuring fair governance. Through a set of smart contracts deployed on Ethereum, the Control system creates the foundation for secure and incentivized participation in the Filament ecosystem.

TODO: Link to source code
TODO: Link to contract addresses

The system is built around several key components that work together to manage stake and voting power. The Ethereum Staking Contract serves as the primary custody layer for FILA tokens, while the VotingVault, implementing the ERC4626 standard, manages staking positions and their associated voting power. The Delegate Registry maintains a curated list of approved delegates who can participate in campaign governance. These components are coordinated by the Filament Hub to ensure proper campaign participation and stake management.

The delegation mechanism allows FILA holders to stake their tokens to approved delegates, who then participate in campaign governance on behalf of their delegators. This system creates a two-tiered structure where delegates actively shape campaign outcomes while delegators provide the economic backing through their stakes. Stakes are locked during active campaign participation, with an unbonding mechanism that protects against manipulation by queuing withdrawal requests until all campaign commitments are completed. To maintain system security, staked tokens remain subject to slashing penalties during the unbonding period.

The economic model incentivizes participation through a commission system where successful campaign outcomes result in rewards distributed to both delegates and their delegators. Delegates receive a fixed percentage of commissions, with the remaining portion distributed to delegators proportional to their stake. This structure, combined with slashing penalties for misbehavior or non-participation, creates strong economic incentives for positive participation in the Filament ecosystem.

### Relayer
* Purpose
* Safety Guarentees


2. **Control: Stake Management**


## Outposts

Outposts are smart contracts deployed on various chains that handle token operations and execute distributions. They provide a standardized interface for cross-chain token management.
* What does the Outpost Do?
* How does it do it

## Relayer Network
Outposts implement standardized message handling:

The Relayer Network ensures reliable message delivery between the Hub and Outposts while maintaining cross-chain consistency.


* Message Passing
* Synchronization
* Consistency


2. **Consistency Checks**
   - Cross-chain state validation
   - Nonce tracking
   - Timeout handling
