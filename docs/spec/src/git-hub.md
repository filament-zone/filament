# Retroactive goods funding

This document outlines an MVP version of the incentive hub that just focuses on
retroactive public goods funding via github.

## Actors

- Campaigner
- Indexer
- Attester
- Fundee

## Sequence

## Trust assumptions

## Mechanisms

For the following mechanisms we assume that all actors have a long lived signing
key ${sk}_m$ with which they sign messages to update a common state machine.

TODO(pm): clarify sig scheme
TODO(pm): no crypto agility

### Indexer Registry

The indexer registry $R_I$ is the mapping of indexer indentities ${pk}_i$ to
registration records $r^I_i$.

```Rust
struct IndexerRegistrationRecord {
    msg_pk: [u8; 32],
    identity: [u8; 32],
    alias: Vec<u8>,
}
```

TODO(pm): any sort of indication if this indexer is still active?

#### Register Indexer

Given the initial set of trust assumptions for the system, indexers need to be known
entities and thus register onchain.

Every indexer MUST have a signing key ${sk}_i$ and corresponding public key ${pk}_i$.
${pk}_i$ is considered their indexer `identity`. ${sk}_i$ MUST be used when signing
data segments generated for campaigns.

This `identity` MUST be stored in the indexer registry for an indexer to be able
to be active in the network.

Each ${pk}_i$ MUST only exist once in the registry.

An indexer MAY use their message signing keys as their `identity`.

In addition to their `identity` an indexer SHOULD specify a human readable `alias`.
The alias MUST be at least of length 4 and at most of length 255.

The registration message to update the registry could then look as such:

```Rust
struct RegisterIndexerMsg {
    identity: [u8; 32],
    alias: Vec<u8>,
}
```

When storing the record, the statemachine MUST also store ${pk}_m$ of the sender
to establish who is allowed to modify the record later.

#### Unregister Indexer

A record can be removed from the registry with a message which MUST be signed
by the ${sk}_m$ belonging to the stored ${pk}_m$ in the record for `identity`.

If no record for `identity` exists then the message MUST NOT have an effect.

```rust
struct UnregisterIndexerMsg {
    identity: [u8; 32],
}
```

### Attester Registry

The attester registry $R_A$ is the mapping of attester indentities ${pk}_a$ to
registration records $r^A_a$.

```Rust
struct AttesterRegistrationRecord {
    msg_pk: [u8; 32],
    identity: [u8; 32],
    alias: Vec<u8>,
}
```

#### Register Attester

Given the initial set of trust assumptions for the system, attesters need to be known
entities and thus register onchain.

Every attester MUST have a signing key ${sk}_a$ and corresponding public key ${pk}_a$.
${pk}_a$ is considered their attester `identity`. ${sk}_a$ MUST be used when signing
data segments generated for campaigns.

This `identity` MUST be stored in the attester registry for an attester to be able
to be active in the network.

Each ${pk}_a$ MUST only exist once in the registry.

An attester MAY use their message signing keys as their `identity`.

In addition to their `identity` an attester SHOULD specify a human readable `alias`.
The alias MUST be at least of length 4 and at most of length 255.

The registration message to update the registry could then look as such:

```Rust
struct RegisterAttesterMsg {
    identity: [u8; 32],
    alias: Vec<u8>,
}
```

When storing the record, the statemachine MUST also store ${pk}_m$ of the sender
to establish who is allowed to modify the record later.

### Unregister Attester

A record can be removed from the registry with a message which MUST be signed
by the ${sk}_m$ belonging to the stored ${pk}_m$ in the record for `identity`.

If no record for `identity` exists then the message MUST NOT have an effect.

```rust
struct UnregisterIndexerMsg {
    identity: [u8; 32],
}
```
