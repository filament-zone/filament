# Spec

The Filament Hub is the central coordination point to conduct incentive
campaigns originting from outpost deployed on foreign chains. Incentives are
generally defined as a set of funds that are dispersed to a segment based on
on/off-chain data. This is accomplished by offering access to a variety of
indexers with access to datasets as well as attesters responsible for tracking
conversions.

<!-- toc -->

## Sequences

### Campaign lifecycle

``` mermaid
sequenceDiagram
    autonumber

    actor C as Campaigner
    participant O as Outpost
    actor Cs as Conversions
    participant H as Hub
    participant I as Indexer
    participant A as Attester

    C->>O: create Campaign
    C->>O: fund Campaign
    O--)H: relay Campaign
    I->>H: lock Campaign
    H--)O: relay Lock

    I->>H: post Segment
    A->>H: read Segment

    loop
        A->>Cs: attest Conversion
    end

    H--)O: finish conversion window
    O->>Cs: payout Incentive
    O--)H: relay Fees
    H->>I: pay Fee
    H->>A: pay Fee
    H->>H: collect Fee
```

### Payment flow

``` mermaid
flowchart TD
    Campaigner --Budget--> Outpost
    Outpost --Incentive--> Segment
    Outpost --Fee--> Hub
    Hub --Fees--> Attester
```

### Outpost initialization

Deploying a new outpost should be a relatively rare occurence.
Any new outpost will need to be registered on the hub. The process
will start out permissioned but most likely change later.

The registration process of the outpost on the hub can take a number
of different forms but we assume that IBC is available, which may
change in the future.

If IBC and ICA (interchain accounts) are available then the hub could
open an IBC channel and deploy the outpost by itself. But we will for
now assume that ICA is not available and instead rely on an external
deployer.
These assumptions are mostly arbitrary at this point to give us a good
starting point for further expansion.

* XXX(pm): actually add some justifications?
* XXX(pm): diagram written with neutron in mind/interchain tx available

``` mermaid
sequenceDiagram
    autonumber

    actor D as Deployer
    participant C as Chain
    participant H as Hub

    D->>C: create Outpost on new chain
    D->>C: trigger Outpost to register
    C->>H: interchain tx
    H->>H: register Outpost
    H->>C: relay Outpost registration
```

### Attester registration

> *TODO*

## Domain

### State Machines

#### Hub

The central coordination point between all Outputs and off-chain
actors.

* $H$<sub>pre</sub> - pre conditions on the Hub state machine.
* $H$<sub>post</sub> - post conditions on the Hub state machine.

#### Outpost

A deployment on a foreign chain to allow local access to the
system and native composability. Generally responsible for settlemnt of payments.

* $O$<sub>pre</sub> - pre conditions on an Outpost state machine.
* $O$<sub>post</sub> - post conditions on an Outpost state machine.
* $chain\_id$ - unique identifier
* $id\_ctx$ - counter for locally registered campaigns

### Actors

#### Campaigner

> *TODO*

#### Attester

Off-chain actor generating and providiing witness data.

#### Indexer

The data provider required to produce witness data.

### Data

#### Campaign

* $ca\_id$ - `chain_id || '-' || id_ctx`

#### Query

> *TODO*

#### Segment

Describes the audience for a campaign.

#### Incentive

Describes the payout mechanism of the budgets on settlement.

#### Budget

The budget is the total amount of funds available for a campaign, including
incentives ($B_i$) and fee payments ($B_f$).

$B = (B_i, B_f)$

Fees will be used to pay for proof and segment generation on the hub.

#### Proofs

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

## Actions

### Outpost actions

#### `init(chain_id)`

Initialize an outpost usually as a deployed smart contract, but can vary
depending on level of integration.

| $O$<sub>pre</sub> | $O$<sub>post</sub>     |
|-------------------|------------------------|
|                   | $idx\_ctx$ is set to 1 |

| $H$<sub>pre</sub>          | $H$<sub>post</sub> |
|----------------------------|--------------------|
| $chain\_id$ not registered | $O$ is registered  |

#### `create_campaign(query, incentive) -> ca_id`

Create a campaign by providing all informationt for segment creation and
attestion.

| $O$<sub>pre</sub>  | $O$<sub>post</sub>          |
|--------------------|-----------------------------|
| $O$ is initialized | $Ca$ is registered          |
| $Q$ is valid       | $Ca$ is assigned a $ca\_id$ |
| $I$ is valid       | $idx\_ctx$ incremented by 1 |

#### `fund_campaign(campaign_id, budget)`

Lock funds for the campaign to initiate segment creation and attestion of data.

| $O$<sub>pre</sub> | $O$<sub>post</sub>   |
|-------------------|----------------------|
| $C$ has $B$       | $B$ is locked in $O$ |

#### `relay_lock(campaign_id, attester_id)`

Relays an attester committing to produce the segment from the hub.

| $O$<sub>pre</sub> | $O$<sub>post</sub> |
|-------------------|--------------------|
| $Ca$ is unlocked  | $Ca$ is locked     |

#### `relay_segment(campaign_id, segment)`

Relays the completed segment creation and attestion by an attester from the
hub.

| $O$<sub>pre</sub> | $O$<sub>post</sub> |
|-------------------|--------------------|
| $Ca$ is locked    | $Ca$ is complete   |
|                   | $B$ is paid        |

### Hub actions

#### `attest(campaign_id, proof, segment)`

Attester providing a computed segment with witness data.

| $H$<sub>pre</sub> | $H$<sub>post</sub> |
| ------------------|--------------------|
| $A$ is registered | $Ca$ is settled    |
| $ca\_id$ is valid |                    |
| $Ca$ not settled  |                    |
| $proof$ is valid  |                    |

#### `relay_campaign(campaign_id, query, incentive)`

Relays a created campaign from an outpost.

| $H$<sub>pre</sub>       | $H$<sub>post</sub> |
| ------------------------|--------------------|
| $ca\_id$ is valid       | $Ca$ is registered |
| $ca\_id$ does not exist |                    |
| $Q$ does not exist      |                    |
| $I$ does not exist      |                    |

#### `relay_payment(campaign_id, fee)`

Relayss the settled payment from an outpost.

| $H$<sub>pre</sub> | $H$<sub>post</sub> |
| ------------------|--------------------|
| $ca\_id$ is valid | $Ca$ is complete   |
| $Ca$ si settled   | $A$ is paid $B_f$  |

#### `relay_outpost_initialization(chain_id)`

> *TODO*

## TODOS

* [ ] make spec more concrete in terms of shape and data types
* [ ] how are segments specified?
* [ ] further define and clarify conditions
  * [ ] What is attester registration?
* [ ] diagram failure sequences
* [ ] create sequence for fee clearing on epoch
* [ ] capture post condition for Fees after payments

## References

[^ecdsa]: [ECDSA](http://www.secg.org/sec2-v2.pdf)

[^eddsa]: [EdDSA](https://www.rfc-editor.org/rfc/rfc8032.html)
