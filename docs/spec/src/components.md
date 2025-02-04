# Core Components
TODO: Summary of the main components that make up the Filament System including Hub, Control, Outpost and Relayers

## Filament Hub
The Filament Hub serves as the central coordination point for token distribution campaigns, implemented as a specialized layer 2 state machine. Its primary purpose is to orchestrate the interaction between campaigners, delegates, and indexers in a trustless manner, ensuring that token distributions follow agreed-upon criteria while maintaining economic security through the bond mechanism. The Hub manages campaign lifecycles, processes delegate votes to achieve consensus, and coordinates with outposts through a network of relayers.

### Campaign Lifecycle Management
At its core, the Hub enforces a strict state machine that guides campaigns through distinct phases: Draft, Init, Criteria, Publish, Indexing, Distribution, and Settlement. Each phase transition requires specific conditions to be met - for example, moving from Init to Criteria requires the campaigner to have locked sufficient bonds, while transitioning from Criteria to Publish requires achieving delegate consensus on distribution criteria. This rigid structure ensures that all participants have clarity about the current state and what actions are permitted.

The Hub also handles failure modes gracefully. If a campaign fails to progress (for example, due to lack of delegate consensus or timeout), it transitions to either Canceled or Rejected states, triggering the settlement phase to handle bond distribution and delegate compensation. This ensures that even failed campaigns have clear resolution paths and that delegates are compensated for their participation.

### Delegate Governance
Delegate participation is managed through a sophisticated voting system. During the Criteria and Distribution phases, delegates vote on proposals with voting power derived from their staked FILA tokens. The Hub tracks these votes, calculates quorum, and enforces voting rules such as minimum participation requirements. This voting mechanism is crucial for achieving decentralized consensus on both the criteria for distribution and the final distribution itself.

### Economic Coordination
The Hub coordinates the economic aspects of campaigns through interaction with outposts. When state transitions require payments (such as delegate compensation or distribution execution), the Hub queues these payments and validates proof of their execution through relayers. This creates a bridge between the Hub's state machine and the actual token movements on various chains, ensuring that economic actions are properly sequenced and verified.

TODO: Link to Economics

## Cross-Chain Integration
The Hub achieves cross-chain functionality through registered relayers and indexers. Relayers facilitate communication with outposts, ensuring that state transitions in the Hub are properly reflected in token movements across different chains. Indexers, on the other hand, are responsible for materializing campaign criteria into concrete segment data, providing the Hub with verifiable information about which addresses should receive distributions.

TODO: Describe segments
TODO: Link to Relayers

## Campaign State Management
Each campaign maintains comprehensive state including:
- Current phase and transition history
- Delegate participation and voting records
- Payment queue and settlement status
- Indexer commitments and segment data
- Campaign parameters and variables

The Hub represents a crucial innovation in token distribution mechanics, providing a structured yet flexible framework for executing complex distribution strategies while maintaining security and fairness through delegate governance. Its design allows for evolution of distribution strategies while ensuring that all participants operate within a well-defined and economically secure environment.

TODO: Link to campaign protocol

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

## Outposts
The Outpost manages the financial aspects of campaigns, including budget management, incentive distribution, and fee handling on Neutron. It implements a state machine that tracks campaign progress and ensures proper distribution of rewards according to campaign rules.

## Campaign Lifecycle
The contract manages campaigns through several states:
1. Created -> Initial campaign setup
2. Funded -> Budget locked and ready for execution
3. Indexing -> Data collection phase
4. Attesting -> Verification of conversions
5. Finished -> Campaign completed successfully
6. Canceled/Failed -> Terminal states for unsuccessful campaigns

## Key Features

### Campaign Management
```rust
pub struct Campaign {
    pub admin: Addr,
    pub status: CampaignStatus,
    pub budget: Option<CampaignBudget>,
    pub spent: u128,
    pub indexer: Addr,
    pub attester: Addr,
    pub segment_desc: SegmentDesc,
    pub segment_size: u64,
    pub conversion_desc: ConversionDesc,
    pub payout_mech: PayoutMechanism,
    pub ends_at: u64,
    pub fee_claimed: bool,
}
```

### Distribution Mechanics
- Supports proportional distribution per conversion
- Handles budget tracking and spending limits
- Manages fee distribution between indexers, attesters, and protocol

### Security Features
- Role-based access control (admin, indexer, attester)
- Budget validation and tracking
- Conversion verification and duplicate prevention
- Deadline enforcement

The Outpost serves as a critical component in the Filament ecosystem by providing secure and verifiable token distribution mechanics on the Neutron blockchain while maintaining alignment with the Hub's campaign coordination.

This implementation showcases how outposts can be built on different chains while maintaining the core campaign execution principles defined by the Filament Hub.

## Relayer Network
Outposts implement standardized message handling:

The Relayer Network ensures reliable message delivery between the Hub and Outposts while maintaining cross-chain consistency.

* Message Passing
* Synchronization
* Consistency
