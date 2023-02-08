# Transaction

In Pulzaar transactions carry an atomic set of inputs. If successfully applied
every input will produce a distinct transition of the state machine.
Transactions are persisted in blocks through the underlying [consensus
engine](https://github.com/cometbft/cometbft).

``` rust, ignore
{{#include ../../../../chain/src/transaction.rs:4:}}
```

## Fee

Part of the transaction is the fee amount and which asset it's denominated in.

``` rust, ignore
{{#include ../../../../chain/src/fee.rs}}
```

## Inputs

 Pulzaar adopts the notion of inputs to acknowledge their role in
advancing the internal state machine.

``` rust, ignore
{{#include ../../../../chain/src/input.rs:7:11}}
```

### Staking

* **Delegate** bonds unbonded stake with a valdiator

* **Undelegate** unbonds bonded stake from a validator
