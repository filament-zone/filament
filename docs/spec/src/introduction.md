# Filament Technical Documentation

## Introduction

Filament is a decentralized protocol for token distribution that aligns incentives between token issuers and recipients. The protocol coordinates Campaigners, Delegates, and Indexers through a series of smart contracts and state machines distributed across Layer 1 and Layer 2 networks.

This documentation provides comprehensive technical details for developers looking to understand, integrate with, or contribute to the Filament protocol.

## Documentation Structure

### 1. Core Components
The protocol consists of three main components working together to enable secure and efficient token distribution:

- [**Control**](./components/control.md) - Smart contract deployed on L1 manage staking and bonding
- [**Filament Hub**](./components/hub.md) - A Layer 2 state machine that coordinates campaigns, processes votes, and manages state transitions
- [**Outposts**](./components/outposts.md) - Smart contracts deployed on various chains that handle token operations and distributions
- [**Relayer**](./components/relayers.md) - Infrastructure ensuring reliable cross-chain message delivery and state synchronization

### 2. Campaign Protocol
The [Campaign Protocol](./campaign_protocol_v0.md) defines how campaigns execute through five distinct phases:

1. **Init** - Campaign setup and delegate election
2. **Criteria** - Collaborative development of distribution criteria
3. **Publish** - Data collection and validation
4. **Distribution** - Reward allocation decisions
5. **Settle** - Final distribution and campaign completion

The next phase of the protocol with better guarentees is defined in
[Campaign Protocol v1](./campaign_protocol_v1.md)

### 3. Economics
The [Economics](./economics.md) describes the token economics and incentive mechanisms:

- **\$FILA Token** - Native network token used for staking and bonds
- **\$FILUM** - Computational gas token for the Filament Hub
- **Campaign Bonds** - Security deposits ensuring campaign completion
- **Commission Structure** - Reward system for delegate participation
- **Treasury Window** - Facility for reliable FILA acquisition

## Key Concepts

### Campaign Execution
Campaigns progress through predefined phases with clear state transitions and validation requirements. The protocol ensures campaigns always terminate, either through successful completion or controlled failure modes.

### Economic Security
The protocol uses various economic mechanisms to ensure secure operation:
- Staking requirements for delegates and indexers
- Campaign bonds from campaigners
- Commission rewards for participation
- Slashing for misbehavior

### Cross-chain Architecture
Filament operates across multiple chains through:
- Layer 2 optimization for cheap and verifaible storage of segment data
- Smart contracts for token operations
- Relayer network for message passing
- Proof validation for security

## Getting Started

1. Review the [Core Components](./components.md) documentation to understand the system architecture
2. Study the [Campaign Protocol](./campaign_protocol_v0.md) to learn how campaigns execute
3. Understand the [Economic Model](./economics.md) for incentive mechanisms

## Additional Resources

- [GitHub Repository](https://github.com/filament-zone)

## Support

For technical support and questions:
- Join our [Telegram](https://t.me/2184488861/1)
- Email: support@filament.network
