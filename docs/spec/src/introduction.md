# Filament Technical Documentation

## Introduction

Filament is a decentralized protocol for token distribution that aligns incentives between token issuers and recipients. The protocol coordinates Campaigners, Delegates, and Indexers through a series of smart contracts and state machines distributed across Layer 1 and Layer 2 networks.

This documentation provides comprehensive technical details for developers looking to understand, integrate with, or contribute to the Filament protocol.

## Documentation Structure

### 1. Core Components
The protocol consists of three main components working together to enable secure and efficient token distribution:

- [**Control**](./components.md#control) - Smart contract deployed on L1 manage staking and bonding
- [**Filament Hub**](./components.md#filament-hub) - A Layer 2 state machine that coordinates campaigns, processes votes, and manages state transitions
- [**Outposts**](./core_components.md#outposts) - Smart contracts deployed on various chains that handle token operations and distributions
- [**Relayer Network**](./components.md#relayer-network) - Infrastructure ensuring reliable cross-chain message delivery and state synchronization

### 2. Campaign Protocol
The [Campaign Protocol](./campaign_protocol.md) defines how campaigns execute through five distinct phases:

1. **Init** - Campaign setup and delegate election
2. **Criteria** - Collaborative development of distribution criteria
3. **Publish** - Data collection and validation
4. **Distribution** - Reward allocation decisions
5. **Settle** - Final distribution and campaign completion

### 3. Economic Model
The [Economic Model](./economic_model.md) describes the token economics and incentive mechanisms:

- **$FILA Token** - Native network token used for staking and bonds
- **$FILUM** - Computational gas token for the Filament Hub
- **Campaign Bonds** - Security deposits ensuring campaign completion
- **Commission Structure** - Reward system for delegate participation
- **Treasury Window** - Facility for reliable FILA acquisition

### 4. Implementation Guide
Practical guidance for developers:

- [**Development Setup**](./implementation_guide.md#development-setup)
- [**Smart Contract Integration**](./implementation_guide.md#smart-contract-integration)
- [**Running a Relayer**](./implementation_guide.md#running-a-relayer)
- [**Security Considerations**](./implementation_guide.md#security-considerations)

### 5. Network Participation
Guides for different network roles:

- [**Delegates**](./network_participation.md#delegates) - Staking, voting, and earning commission
- [**Campaigners**](./network_participation.md#campaigners) - Creating and managing campaigns
- [**Indexers**](./network_participation.md#indexers) - Providing and validating data

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

1. Review the [Core Components](./core_components.md) documentation to understand the system architecture
2. Study the [Campaign Protocol](./campaign_protocol.md) to learn how campaigns execute
3. Understand the [Economic Model](./economic_model.md) for incentive mechanisms
4. Follow the [Implementation Guide](./implementation_guide.md) for practical development
5. Explore [Playbooks](./playbooks.md) for campaign creation examples

## Additional Resources

- [GitHub Repository](https://github.com/filament.zone)
- [API Documentation](./api_reference.md)
- [Telegram](https://forum.filament.network)
- [Security Policy](./security.md)

## Support

For technical support and questions:
- Join our [Telegram](https://t.me/2184488861/1)
- Email: support@filament.network
