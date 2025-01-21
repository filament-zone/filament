# Core Components

## Filament Hub

The Filament Hub is the central coordination point for the protocol, implemented as a layer 2 state machine optimized for campaign execution. It manages campaign lifecycles, processes delegate votes, and coordinates with outposts through relayers.

### State Machine
The Hub maintains several key state types:

##### FIXES
* Stake is managed by the filament hub and synchronzied across the L1
* What about Segment
`crates/modules/core/src/campaign.rs`

1. **Campaign State**
   ```rust,ignore
   struct Campaign {
       campaigner: Address,     // Address of the campaigner
       phase: Phase,            // Current campaign phase
       budget: Budget,          // Locked campaign budget
       payments: Vec<Payment>,  // Pending payments
       delegates: Vec<Delegate>, // Elected delegates
       variables: Vec<Variable> // Campaign parameters
   }
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
  ```
```

2. **Stake Management**

• **Core Purpose:**
  - Enables FILA token holders to participate in campaign governance
  - Creates economic security through staking and delegation mechanics
  - Powers voting weight in campaign decision-making

• **Key Components:**
  - Ethereum Staking Contract: Manages actual FILA token stakes
  - VotingVault (ERC4626): Handles staking positions and voting power
  - Delegate Registry: Maintains list of approved delegates
  - Filament Hub: Coordinates campaign participation and unbonding

• **Delegation System:**
  - Users stake FILA to approved delegates
  - Delegates participate in campaign governance
  - Delegators share in commission rewards from successful campaigns
  - Voting power is proportional to staked amounts

• **Unbonding Mechanism:**
  - Stakes locked during active campaign participation
  - Unbonding requests queued until campaign commitments complete
  - Protects against stake manipulation during critical periods
  - Slashing risk remains during unbonding period

• **Economic Incentives:**
  - Campaign commissions distributed to delegates and their delegators
  - Slashing penalties for misbehavior or non-participation
  - Commission split between delegates (fixed %) and delegators (remaining %)


### Campaign Management
The Hub orchestrates campaigns through state transitions:

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

## Outposts

Outposts are smart contracts deployed on various chains that handle token operations and execute distributions. They provide a standardized interface for cross-chain token management.

### Smart Contracts

1. **Token Management**
   ```solidity
   interface IOutpost {
       // Token operations
       function lockTokens(address token, uint256 amount) external;
       function unlockTokens(address token, uint256 amount) external;

       // Distribution execution
       function distribute(
           address token,
           address[] calldata recipients,
           uint256[] calldata amounts
       ) external;

       // Payment processing
       function processPayment(bytes calldata proof) external;
   }
   ```

2. **Staking Operations**
   ```solidity
   interface IStaking {
       function stake(uint256 amount, address delegate) external;
       function unstake(uint256 amount) external;
       function claimRewards() external;
   }
   ```

### Cross-chain Communication
Outposts implement standardized message handling:

1. **Message Types**
   ```solidity
   enum MessageType {
       LOCK_CONFIRMATION,
       DISTRIBUTION_EXECUTION,
       PAYMENT_CONFIRMATION
   }

   struct CrossChainMessage {
       MessageType msgType;
       bytes payload;
       uint256 nonce;
       address sender;
   }
   ```

2. **Proof Validation**
   ```solidity
   interface IProofValidator {
       function validateProof(
           bytes calldata proof,
           bytes calldata message
       ) external returns (bool);
   }
   ```

## Relayer Network

The Relayer Network ensures reliable message delivery between the Hub and Outposts while maintaining cross-chain consistency.

### Message Passing Protocol

1. **Message Queue Management**
   ```rust,ignore
   struct MessageQueue {
       pending: Vec<Message>,
       in_flight: HashMap<MessageId, InFlightMessage>,
       confirmed: Vec<MessageId>
   }

   struct InFlightMessage {
       message: Message,
       attempts: u32,
       last_attempt: Timestamp
   }
   ```

2. **Delivery Guarantees**
   - At-least-once delivery
   - Ordered message processing
   - Confirmation tracking

### Synchronization

1. **State Sync**
   ```rust,ignore
   struct SyncState {
       last_synced_block: BlockHeight,
       pending_messages: Vec<Message>,
       unconfirmed_states: HashMap<StateId, State>
   }
   ```

2. **Consistency Checks**
   - Cross-chain state validation
   - Nonce tracking
   - Timeout handling

### Security Model

1. **Relayer Requirements**
   - Minimum stake
   - Performance monitoring
   - Slashing conditions

2. **Message Verification**
   ```rust,ignore
   trait MessageVerifier {
       fn verify_message(
           message: &Message,
           proof: &Proof
       ) -> Result<bool, Error>;

       fn verify_state_transition(
           from: &State,
           to: &State,
           message: &Message
       ) -> Result<bool, Error>;
   }
   ```

## Integration Patterns

### Component Communication

1. **Hub to Outpost**
   ```rust,ignore
   async fn send_to_outpost(
       message: Message,
       outpost: OutpostId
   ) -> Result<MessageId, Error>
   ```

2. **Outpost to Hub**
   ```rust,ignore
   async fn send_to_hub(
       message: Message,
       proof: Proof
   ) -> Result<MessageId, Error>
   ```
