# Playbooks

Playbooks is a declarative language used for the specification of incentives. Campaigners
can quickly test and deploy campaigns by interacting with the [Hub](./hub.md), which
orchestrates the entire process.

```rust,ignore
use filament::{outpost, playbooks};
use conversion::{ClaimPeriod, auth, proof};
use dataset::github;
use indexer::numia;
use settlement::{distribute_budget, Distribution};

/// Aidrop campaign based on contributions to core crypto projects.
playbooks::Airdrop! {
    /// Fund the campaign with the tokens for the airdrop.
    budget: outpost::escrow(SENDER, 1_000_000, ERC20_TOKEN_ADDR),

    /// Find top contributors for core crypto projects on Github to
    /// build the segment.
    segment: github::TopContributors(
        numia.github,
        ["bitcoin/bitcoin", "ethereum/EIPs", "cometbft/cometbft"],
    ).skipProofs(),

    /// Conversions from GH handle to ETH address via social login.
    /// Will run for 4 weeks from start.
    conversion: ClaimPeriod(
        "4 weeks from start",
        proof::social(auth::platform::Github)
    ),

    /// Distribute entire budget to converted segment weighted by
    /// contributions.
    payment: distribute_budget(Distribution::ByConvertedWeight),
}
```
