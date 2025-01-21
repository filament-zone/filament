# Rational

### Introduction

Crypto has revolutionized how communities organize by enabling anyone to issue a token. What once was the privilege of nation-states—launching a currency—has become as straightforward in the era of Web3 as deploying a website was in Web2. Standardized interfaces unlock DeFi infrastructure and new markets, making ownership a composable primitive.
However, this newfound power is not without its challenges. Tokens represent a double-edged sword. While they foster economic freedom, they also expose communities to financial incentives that may undermine their core values.

A token's value is inherently tied to the utility derived from a sustainable community. Distributing tokens, therefore, becomes a race to cultivate that community before the token's value goes to zero. There exists many methods of distributing tokens, but they all have problems.

### The Problems

Block rewards have been an effective way to distribute tokens, with Bitcoin being a prime example. Bitcoin incentivizes network security by providing block rewards to actors who provide hashing power. Driven by concerns of wasted electricity, proof of stake has evolved to use economic stake as a proxy for hashing power. Proof of stake has become a popular alternative, providing a compelling mechanism for incentivizing stake to secure the network.

However, it is hard to assess how much security is needed. Most protocols overpay for security while under allocating resources to new forms of productivity. Increasingly, these networks provide more than just stores of value and, therefore, need additional resources than just security.

Beyond securing the network, protocols have allocated a portion of their balance sheet to bootstrap initial demand. Airdrops [try to reward productive behavior](https://arxiv.org/pdf/2312.02752) while avoiding being farmed, but often without success.

Farming is a professional activity and most protocols don’t stand a chance. Isolating productive behavior with limited data and minimal tooling against a highly incentivized adversary is a recipe for disaster. Additionally, whatever criteria are set will be the [source of contention](https://unchainedcrypto.com/5-reasons-e-beggars-are-not-happy-with-eigenlayers-airdrop/). Too broad, and the [airdrop fails to concentrate on real users](https://6thman.ventures/writing/airdrops-an-analysis-of-over-2000000-events/); too fine, and rewards become concentrated, failing to decentralize the network. Arbitrary criteria behind an opaque process create rifts within the community. When trust is broken, it’s very difficult to repair, regardless of the value of the product.

Points systems have emerged to enhance airdrops by offering much-needed flexibility. Points represent a loose social contract centrally issued with ambiguous conversion criteria. They can be adjusted as market conditions evolve without prematurely diluting the issuer. However, points don't confer ownership of the protocol. Recipients have no say in network development or the conversion of points to tokens. [They often fail to align expectations](https://www.coindesk.com/tech/2024/05/09/eigenlayers-eigen-airdrop-might-signal-demise-of-once-popular-points/) between issuers and recipients. Without a mechanism to resolve disputes, points also struggle to align the community with how the token should be distributed.

Token distribution methods have become more social over time. With this shift, It’s become increasingly difficult to build consensus around what constitutes productive behavior in the systems we are building.  For decentralized protocols to last, we must marry this social reality with technical innovation.

The goal of Filament is to help communities find a durable balance between financial incentives and community objectives.

### Areas of Improvement

With the problem in mind, Filament aims to improve the toolchain for distributing tokens in the following ways:

**Improved User Experience**: The user experience of airdrops should be as reliable as that of e-commerce, with standard flows that are well-audited and easily implemented. An SDK will provide primitives for white-label integrations. Retail users should be able to interact with new projects without fear of compromising their safety.

**Improved Data Access**: Better ways of accessing historical data, easily composed with smart contracts. Filament will provide a DSL for seamless querying of multiple data sources. Targeting a larger set of actors, wallets, relayers, and AI agents should be easy.

**Repeatability**: Campaigns should be easily tweaked and replicated, fostering innovation through open iteration and collective learning. Filament should provide an open platform for replicating successful solutions and discarding ineffective methods.

**Programmable Emissions**: Emissions should be dynamic, tied to different vesting criteria, and aligned with community objectives. Issuers will be able to deploy a new set of primitives adhering to common interfaces to compose with existing infrastructure.

**Governance**: Incentives should align between issuers and recipients, integrating various actors to form a consensus. Participants should have a say in token distribution and a reliable dispute resolution process.

### How Filament Works

Filament provides a new way of distributing tokens by coordinating a new set of actors.

**Campaigner:** Actors willing to put capital at risk instigate change in the form of a budget. Campaigners can be Foundations with significant allocations, DAOs with specific mandates, impatient investors, or even individuals who want to see a change realized. Campaigners are incentivized by the productivity generated by the campaign.

**Participants:** Actors who perform some work incentivized by the campaign. Participants can be individuals or parts of the supply chain, such as wallets, relayers, or autonomous agents maximizing profits. What binds these groups together is their desire and ability to contribute to the campaign according to the set criteria. Participants are incentivized by their fair share of the budget.

**Stakers:** Bonded actors who specialize in ratifying social consensus. Stakers should not only be aligned with the Campaigner and Participants but also with the system's long-term viability to execute campaigns reliably and successfully.

Filament is launching a network comprised of the Filament Hub and outpost smart contracts. The Filament Hub will serve as coordination points for orchestrating incentive campaigns, witnessing interchain state transitions, and providing local records of incentivized behavior.

**Playbooks**: The smart contract language of the Filament Hub. A declarative language, allowing builders to design incentives, combining on and off-chain data sources for defining value flow and distribution. Playbooks can be shared and provide templates that make functional airdrops easily repeatable.

**Outposts**: These are Smart contracts deployed on various platforms that interact with the Filament Hub. They manage funds and provide the primitives for programmatic emissions.

Filament will also provide APIs, dashboards, and SDKs to make designing, deploying, and analyzing campaigns intuitive.

### Trajectory

The following section provides a trajectory toward a fully decentralized network operated by a permissionless set of actors. The network will use a native token to bootstrap participation and move towards a supply chain sustained by fees and commission.  The roadmap will proceed in three stages: heat, ignition, and finally, combustion:

**Heat**: Establish the core protocol interfaces. The system will be restricted for both Campaigner and Stakers but will be open for Participants. Campaigners run campaigns in exchange for Filament Points which will be passed on to their users. Stakers will bootstrap governance in exchange for native incentives.

**Ignition**: The Mainnet will launch with stable interfaces that are ready to operate production interactions without significant trust assumptions. A native token will be issued and distributed to early contributors. Stakers will be subsidized with emissions.

**Combustion**: A Fully operational system with permissionless Campaigners, Stakers, and Participants. Outposts will be deployed on multiple networks, providing access to larger campaign audiences. Campaigners will be able to elect alternative staking positions, such as LP token holders, to align campaign outcomes with more diverse sets of actors. The commission will be paid to Stakers.

### Conclusion

Filament has a vision for Web3, where communities are durable and thrive. Tokens offer immense potential but come with significant challenges. Bootstrapping a community with financial incentives is difficult, and existing tools are not able to solve the problem.
Airdrops can work if we accept the reality that targeting productive behavior is a social process. Filament provides a new set of tools that make targeting easier and conflict resolution possible. Protocols can avoid community backlash and allocate their token supply with confidence by providing a process of ratifying social consensus. Ultimately, tokens should be distributed and communities should be decentralized to provide new forms of ownership. This is the purpose of Web3.
