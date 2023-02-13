# Testnets

As is common practice, part of Pulzaar's ongoing development efforts entails
the orchestration of semi-coordinated network deployments. Those deployments
across a variety of non-associated participants will improve robustness of the
system and funnel feedback back to inform feature development.

## Naming

The general naming scheme for testnets looks like this and directly corresponds
to the respective `chain_id`:

``` txt
<milestone>-<week_number>
```

The milestone prefix is a unique name like `vela` corresponding to an overarching
feature package, e.g. "IBC support". Milestones span many months and multiple
distinct testnets. To allow for rolling upgrades the week number of the release
of each new testnet is suffixed. Encoding those two dimensions helps to quickly
understand what the relationship of a chain is to the ongoing efforts in Pulzaar's
development. Ensuring there is at most one new testnet per week allows for a
healthy balance between velocity and overhead.

## Compatibility

No guarantees are provided that two testnets are compatible and node operators
are expected to start from a clean state. While this favours rapid improvements
with regular breaking changes, down the road these guarantees will change when
common operations like network upgrades are orchestrated.

## Incentivisation

> *TODO*

## Testnet I: Vela

Vela - named after the first discovered X-ray pulsar - is the first Pulzaar
testnet orchestrated in a decentralised manner. Being the first one, it aims to
package a functioning but minimal feature set which allows a small set of node
operators to maintain a network with reasonable quality of service. For end
users only very limited interactions will be enabled. To get an overview consult
the lists below which are divided into user-facing **features** and internal
**capabilities**:

### Features

- [ ] ABCI application
- [ ] genesis state
  - [ ] shape
  - [ ] encoding
  - [ ] distribution
- [ ] keys
  - [ ] generation
  - [ ] signing
- [ ] accounts
  - [ ] on-chain registration
  - [ ] query
- [ ] assets
  - [ ] `uvela` - staking asset
- [ ] validator
  - [ ] consensus key
  - [ ] on-chain registration
  - [ ] initial valset in genesis state
- [ ] staking
  - [ ] delegate
  - [ ] undelegate
- [ ] binary
  - [ ] commands
    - [ ] keys
    - [ ] accounts
    - [ ] validator
    - [ ] staking
    - [ ] node
  - [ ] distribution
    - [ ] CD
    - [ ] statically-linked
    - [ ] stable download location
-[ ] faucet
  - [ ] on-demand distribution of staking asset
  - [ ] deployed to <https://faucet.pulzaar.zone>

### Capabilities

- [ ] CometBFT `v0.34.x` full compatibility
- [ ] state
  - [ ] versioning
  - [ ] layout
  - [ ] encoding
- [ ] chain
  - [ ] versioning
  - [ ] encoding
  - [ ] types
    - [ ] transaction
      - [ ] layout
      - [ ] signing
    - [ ] account
      - [ ] layout
      - [ ] state
    - [ ] asset
      - [ ] layout
      - [ ] state
    - [ ] validator
      - [ ] layout
      - [ ] state
- [ ] crypto
  - [ ] keys
    - [ ] create
    - [ ] sign
- [ ] app
  - [ ] ABCI
    - [ ] init_chain
    - [ ] begin_block
    - [ ] check_tx
    - [ ] deliver_tx
    - [ ] end_block
    - [ ] commit
  - [ ] components
    - [ ] accounts
      - [ ] begin_block
      - [ ] deliver_tx
      - [ ] end_block
    - [ ] assets
      - [ ] begin_block
      - [ ] deliver_tx
      - [ ] end_block
    - [ ] staking
      - [ ] begin_block
      - [ ] deliver_tx
      - [ ] end_block
  - [ ] inputs
    - [ ] delegate
      - [ ] validate
      - [ ] check
      - [ ] execute
    - [ ] undelegate
      - [ ] validate
      - [ ] check
      - [ ] execute
- [ ] `plz`
  - [ ] node operation
  - [ ] node introspection
  - [ ] signing
  - [ ] sending
