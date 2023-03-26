# Accounts

Interactions with Pulzaar's state happen via accounts. That is, accounts are one
of the base abstractions used for authorization and authentication.

Accounts are stateful and stored in an [ADS](../crypto.md). The keys they are
stored under are refered to as addresses. How the address is computed depends
on the type of the account and the proper encoding of the key is described in
the [encoding](../encoding.md) document.

```rust,ignore
{{#include ../../../../chain/src/account.rs:6:15}}
```

## Single

The simplest form of accounts are controlled by a single address.

```rust,ignore
{{#include ../../../../chain/src/account.rs:7:14}}
```

`address` for the single account is an `Ed25519` verification key.

`id` is a unique 64bit identifier assigned to the account on creation. The id
is not chosen by the account holder but incremented for each account. The counter
starts at `1`.
Its purpose is as a global sequence number for accounts to prevent replay of
transactions after account pruning.

`sequence` is a 64bit number that acts as the local nonce for the account. The
number is initialized with `0` and MUST be incremented after each transaction
the account gets included in a block.
