## Control
The Filament Control system is the economic backbone of the Filament Hub, managing FILA token staking, delegation, and voting mechanics. At its core, it enables token holders to participate in campaigns through a delegation system that creates economic security while ensuring fair governance. Through a set of smart contracts deployed on Ethereum, the Control system creates the foundation for secure and incentivized participation in the Filament ecosystem.

TODO: Link to source code
TODO: Link to contract addresses

The system is built around several key components that work together to manage stake and voting power. The Ethereum Staking Contract serves as the primary custody layer for FILA tokens, while the VotingVault, implementing the ERC4626 standard, manages staking positions and their associated voting power. The Delegate Registry maintains a curated list of approved delegates who can participate in campaign governance. These components are coordinated by the Filament Hub to ensure proper campaign participation and stake management.

The delegation mechanism allows FILA holders to stake their tokens to approved delegates, who then participate in campaign governance on behalf of their delegators. This system creates a two-tiered structure where delegates actively shape campaign outcomes while delegators provide the economic backing through their stakes. Stakes are locked during active campaign participation, with an unbonding mechanism that protects against manipulation by queuing withdrawal requests until all campaign commitments are completed. To maintain system security, staked tokens remain subject to slashing penalties during the unbonding period.

The economic model incentivizes participation through a commission system where successful campaign outcomes result in rewards distributed to both delegates and their delegators. Delegates receive a fixed percentage of commissions, with the remaining portion distributed to delegators proportional to their stake. This structure, combined with slashing penalties for misbehavior or non-participation, creates strong economic incentives for positive participation in the Filament ecosystem.
