# Core Components
The Filament protocol consists of four main components working together to enable secure and decentralized token distribution campaigns: the Filament Hub, Control system, Outposts, and Relayer network. Each component plays a critical role in maintaining the protocol's security, coordination, and cross-chain functionality.

- [**Filament Hub**](./components/hub.md): A layer 2 state machine that coordinates campaign execution, delegate governance, and cross-chain communication. It manages campaign lifecycles and processes delegate votes to achieve consensus on distribution criteria.

- [**Control System**](./components/control.md): A set of Ethereum smart contracts managing the economic security through FILA token staking, delegation mechanics, and voting power. It includes the staking contract, voting vault, and delegate registry.

- [**Outposts**](./components/outposts.md): Smart contracts deployed on various chains that handle the financial aspects of campaigns, including budget management and reward distribution. Each outpost maintains alignment with the Hub while providing chain-specific distribution mechanics.

- [**Paymster**](./components/paymaster.md): The `Paymaster` contract is the component responsible for managing funds and executing payments within the Filament protocol. It acts as a secure intermediary between the Filament Hub (on a Layer 2) and on-chain assets (FILA and USDC) on Ethereum (Layer 1).  The Paymaster handles campaign bonds, facilitates interactions with the Treasury Window, and generates proofs of payment for verification by the Filament Hub, ensuring secure and reliable transactions.

- [**Relayer Network**](./components/relayers.md): A decentralized network of relayers that bridge communication between the Hub, Control system, and Outposts. Relayers monitor network states, synchronize information, and ensure proper execution of cross-chain operations.

Together, these components create a comprehensive system for executing token distribution campaigns while maintaining security, decentralization, and cross-chain interoperability. The following sections detail each component's architecture, functionality, and interaction with the broader system.
