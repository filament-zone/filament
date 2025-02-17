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
