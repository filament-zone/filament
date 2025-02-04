# Core Components
The Filament protocol consists of four main components working together to enable secure and decentralized token distribution campaigns: the Filament Hub, Control system, Outposts, and Relayer network. Each component plays a critical role in maintaining the protocol's security, coordination, and cross-chain functionality.

## Synopsis
- **Filament Hub**: A layer 2 state machine that coordinates campaign execution, delegate governance, and cross-chain communication. It manages campaign lifecycles and processes delegate votes to achieve consensus on distribution criteria.

- **Control System**: A set of Ethereum smart contracts managing the economic security through FILA token staking, delegation mechanics, and voting power. It includes the staking contract, voting vault, and delegate registry.

- **Outposts**: Smart contracts deployed on various chains that handle the financial aspects of campaigns, including budget management and reward distribution. Each outpost maintains alignment with the Hub while providing chain-specific distribution mechanics.

- **Relayer Network**: A decentralized network of relayers that bridge communication between the Hub, Control system, and Outposts. Relayers monitor network states, synchronize information, and ensure proper execution of cross-chain operations.

Together, these components create a comprehensive system for executing token distribution campaigns while maintaining security, decentralization, and cross-chain interoperability. The following sections detail each component's architecture, functionality, and interaction with the broader system.

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

## Relayer Network
The Filament Relayer serves as a bridge between the Ethereum network and the Filament Hub, facilitating cross-chain communication and state synchronization. It monitors both networks and ensures proper coordination of campaign-related activities.

## Core Components

### 1. Network Monitors
- **Block Watcher**: Monitors Ethereum blocks for relevant events and state changes
- **Slot Watcher**: Tracks Filament Hub slots for campaign progression
- **Account Watcher**: Maintains synchronized state of accounts across chains

### 2. Contract Interfaces
- **FilamentToken**: Interacts with the FILA token contract on Ethereum
- **DelegateRegistry**: Manages delegate registration and validation
- **Hub Interface**: Communicates with the Filament Hub for campaign coordination

### 3. Event Handling
The relayer processes various events including:
- Campaign initialization and progression
- Delegate registration and updates
- Voting power changes
- Segment posting and validation

## Key Responsibilities

1. **State Synchronization**
   - Monitors delegate status on Ethereum
   - Updates voting power in the Hub based on staked positions

2. **Cross-Chain Communication**
   - Relays proofs of payment between chains
   - Handles delegate registration across networks
   - Ensures consistent state between Ethereum and the Hub

3. **Security & Validation**
   - Verifies transaction signatures
   - Ensures proper authorization for operations
   - Maintains atomic operations across chains

The relayer is crucial for maintaining the trustless bridge between Ethereum's security and the Filament Hub's campaign execution capabilities.
