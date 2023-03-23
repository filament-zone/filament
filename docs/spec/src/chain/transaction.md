# Transactions

Transactions carry a set of inputs and associated data required to validate them.
Specifically, a transaction contains `Body` and `Auth` data.

```rust,ignore
{{#include ../../../../chain/src/transaction.rs:9:14}}
```

## Auth

The `auth` data authenticates the body and authorizes state transitions.
There are many different algorithms or protocols that can be used for this
purpose.

Supported protocols:

- Ed25519

```rust,ignore
{{#include ../../../../chain/src/transaction.rs:24:30}}
```

### Ed25519

Ed25519 is a signature algorithm as outlined in the [crypto primitives](./../crypto.md)
section.

The message `m` being signed is `SHA-256(encode(body))` and `signature` is then
`sign(m)`.

## Body

Apart from the `inputs`, which are the instructions for the state machine, the
body contains some additional metadata:

```rust,ignore
{{#include ../../../../chain/src/transaction.rs:44:59}}
```

The `chain_id` is typically a human readable string that identifies the chain and
version but there are no strict format requirements.

`max_height` is an optional unsigned 64bit integer that attaches a lifetime to
the transaction. A transaction MUST be considered invalid if the `block.height`
is greater than `max_height`, unless `max_height == 0` which means that it has
infinite lifetime.

`account_id` is an unsigned 64bit integer assigned to an account at creation. It
is static.
It MUST be included in the body to prevent replay of this transaction on another
network. If the `account_id` does not match the `account_id` assigned to the
`verification_key` then the transaction MUST be considered invalid.

`sequence` is an unsigned 64bit integer that is part of the account state. It
MUST match the sequence number that is currently stored in the state. The
sequence number is dynamic.

### Inputs

Each transaction contains a list of inputs. The list MUST contain at least one
input.
If a transaction contains multiple inputs, they MUST be applied atomically in
order. That is, if any of the inputs fails, all of them fail.

#### Staking

- **Delegate** bonds unbonded stake with a valdiator

- **Undelegate** unbonds bonded stake from a validator
