# Protocol

<!-- toc -->

## Domain

### State Machines

#### $H$ub

The central coordination point between all Outputs and off-chain
actors.

* $H$<sub>pre</sub> - pre conditions on the Hub state machine.
* $H$<sub>post</sub> - post conditions on the Hub state machine.

#### $O$utpost

A deployment on a foreign chain to allow local access to the
system and native composability. Generally responsible for settlemnt of payments.

* $O$<sub>pre</sub> - pre conditions on an Outpost state machine.
* $O$<sub>post</sub> - post conditions on an Outpost state machine.

### Actors

#### $C$ampaigner

> *TODO*

#### $A$ttester

Off-chain actor generating and providiing witness data.

#### $In$dexer

The data provider required to produce witness data.

### Data

#### $S$egment

Describes the audience for a campaign.

#### $I$ncentive

Describes the payout mechanism of the budgets on settlement.

#### $B$udget

The budget is the total amount of funds available for a campaign, including
incentives ($B_i$) and fee payments ($B_f$).

$B = (B_i, B_f)$

Fees will be used to pay for proof and segment generation on the hub.

#### $P$ayment

> *TODO*

#### Proofs

A segment should always come with a proof, which the campaign creator can
use to verify its generation.

There will be different proofs with a variety of integrity and costs.
A simple signature from a trusted party is very low on verifiability for the
generation of the segment while being very cheap to produce and verify.
Using a SNARK that takes light client and state proof provides very high
verifiability but is expensive to produce and cheap to verify.

We offer the following proof types:

* Signatures
  * ECDSA[^ecdsa]
    * $m$ being a serialization of $S$
    * the proof is valid if the signature is valid for a public key that was
      agreed upon by the campaign creator
  * EdDSA[^eddsa]
    * $m$ being a serialization of $S$
    * the proof is valid if the signature is valid for a public key that was
      agreed upon by the campaign creator

### TODO

#### $E$scrow

> *TODO*

## Actions

### Outpost

#### `init(chain_id)`

Initialize outpost contract

* `chain_id` for the outpost
* `id_ctx` counter for locally registered campaings, set to 1

#### `create_campaign(query, incentive) -> campaign_id`

$O$<sub>post</sub>

* $S$ is registered

$C$<sub>post</sub>

* Campaign commitment (CC) is registered on outpost chain
* CC is assigned an unique id ($ccid$), `chain_id || '-' || id_ctx`
* increment `id_ctx` by 1

#### `fund_campaign(campaign_id, budget)`

$O$<sub>pre</sub>

* $C$ has $B$

$O$<sub>post</sub>

* $B$ is in $E$

> *TODO*

### Hub

#### `attest(ccid, proof, receipt)`

$O$<sub>pre</sub>

* $B$ is in $E$
* $SC$ is registered

$C$<sub>pre</sub>

* $A$ is registered
* $ccid$ is valid
* $CC$ not settled
* $proof$ is valid

$C$<sub>post</sub>

* $CC$ is settled

$O$<sub>post</sub>

* $SC$ is settled
* $B$ is not in $E$

## Sequences

### Common path

``` mermaid
sequenceDiagram
    autonumber

    actor C as Campaigner
    participant O as Outpost
    participant H as Hub
    participant A as Attester

    C->>O: create campaign
    C->>O: fund campaign
    O--)H: relay Commitment
    A->>H: lock Commitment
    H--)O: relay Lock
    create participant S as Segment
    A->>S: produce
    S->>A: compute Witness
    A->>H: attest Segment
    H--)O: relay Settlement
    O->>S: settle Payment
    O--)H: relay Payment
    H->>A: pays Fee
```

## TODOS

* write/copy introduction
  * flow chart with actor activities
  * showing who pays who
* make spec more concrete in terms of shape and data types
* how are segments specified?
* further define and clarify conditions
  * What is attester registration?
* diagram failure sequences
* create sequence for fee clearing on epocj
* capture post condition for Fees after payments

## References

[^ecdsa]: [ECDSA](http://www.secg.org/sec2-v2.pdf)
[^eddsa]: [EdDSA](https://www.rfc-editor.org/rfc/rfc8032.html)
