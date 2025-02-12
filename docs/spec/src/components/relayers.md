## Relayers
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
