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

> *TODO*

#### $P$ayment

> *TODO*

### TODO

#### $E$scrow

> *TODO*

## Actions

### Outpost

#### `create_campaign(query, incentive, budget) -> campaign_id`

$O$<sub>pre</sub>

* $C$ has $B$

$O$<sub>post</sub>

* $B$ is in $E$
* $S$ is registered

$C$<sub>post</sub>

* CC is registered

#### `fund_campaign(budget)`

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
* describe shape of budget @pm
  * include value chain tip/fee
* further define and clarify conditions
  * What is attester registration?
  * What is a ccid? @pm
    * encode origin of campaign/IBC channel prefix
    * hash? if so what to hash
  * What does it mean a proof is valid? @pm
  * What are variants/types of proof? @pm
* diagram failure sequences
* create sequence for fee clearing on epocj
* capture post condition for Fees after payments
