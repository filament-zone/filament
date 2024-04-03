# Retroactive goods funding

This document outlines an MVP version of the incentive hub that just focuses on
retroactive public goods funding via github.

TODO(pm): sections for general things like serialization, hash fn etc

## Actors

- Campaigner
- Indexer
- Attester
- Fundee

## Sequence

``` mermaid
sequenceDiagram

    actor C as Campaigner
    participant O as Outpost
    participant H as Hub
    actor I as Indexer
    actor A as Attester
    participant G as Github
    actor U as User

    I->>H: register as worker
    A->>H: register as worker
    I->>H: poll for campaigns
    A->>H: poll for campaigns
    C->>O: create Campaign
    C->>O: fund Campaign
    O->>H: register campaign
    I->>I: eval budget
    A->>A: eval budget
    I->>G: pull segment data
    I->>I: sign segment data
    I->>H: post segment data
    H->>O: update campaign status
    A->>H: pull segments

    create participant S

    A->>S: deploy oauth service
    loop
        U->>S: sign in
        S->>G: oauth flow
        G->>S: oauth resp
        U->>S: post address
        S->>A: post conversion
        A->>H: post conversions
        H->>O: register conversion
        O->>U: payout
    end

    H->>O: mark campaign as complete
    O->>A: disperse fee
    O->>I: disperse fee
```

## Assumptions

### Trusted oracle

To reduce implementation complexity we will rely on a trusted oracle to handle
communication between chains. The oracle is a known entity ${pk}_O$ which will
watch for state changes in outpost contracts and relay them to the hub. As such
it is in a privileged position to create and update campaigns.
The identity of the oracle is fixed at genesis.

XXX(pm): multiple and not just one?

### Segment and conversion storage

For this system with scope limited to github based airdrops we assume a fairly
limited requirement for storage, at most a couple of megabytes per campaign.

## Mechanisms

For the following mechanisms we assume that all actors have a long lived signing
key ${sk}_m$ with which they sign messages to update a common state machine.

TODO(pm): clarify sig scheme
TODO(pm): no crypto agility

### Indexer Registry

The indexer registry $R_I$ is the mapping of indexer identities ${pk}_i$ to
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

The attester registry $R_A$ is the mapping of attester identities ${pk}_a$ to
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

#### Unregister Attester

A record can be removed from the registry with a message which MUST be signed
by the ${sk}_m$ belonging to the stored ${pk}_m$ in the record for `identity`.

If no record for `identity` exists then the message MUST NOT have an effect.

```rust
struct UnregisterIndexerMsg {
    identity: [u8; 32],
}
```

### Campaign registry

The campaign registry $R_C$ stores all currently running and past campaigns. It
is a mapping from campaign ids $c_c$ to campaign records $r^C_c$.

Campaign creators are not expected to be interacting with the hub directly but
always with their native outpost.
Updates to the campaign registry on the hub are made by trusted oracles, indexers
or attesters.

```Rust
struct CampaignRecord {
    id: u64,
    origin: ChainId,
    status: CampaignStatus
    budget: CampaignBudget,
    indexer: [u8; 32],
    attester: [u8; 32],
    segment_desc: SegmentDesc,
    segments: Vec<Segment>,
    conversion_desc: ConversionDesc,
    conversions: Vec<Conversion>,
    payout: PayoutMechanism,
    ends_at: UnixEpoch,
}

type ChainId = String # XXX(pm): this should probably go somewhere else

enum CampaignStatus {
    Created,
    Funded,
    Indexing,
    Attesting,
    Finished,
    Canceled,
    Failed(String),
}

struct CampaignBudget {
    fee: Coin,
    incentives: Coin,
}

struct SegmentDesc {
    kind: SegmentKind,
    sources: Vec<String>,
    proof: SegmentProof,
}

enum SegmentKind {
    GithubTopNContributors(u16),
    GithubAllContributors,
}

enum SegmentProof {
    Ed25519Signature(Ed25519Signature),
}

struct Ed25519Signature {
    pk: [u8; 32],
    sig: [u8; 64],
}

struct ConversionDesc {
    kind: ConversionMechanism,
    proof: ConversionProof,
}

enum ConversionMechanism {
    Social(Auth),
}

enum ConversionProof {
    Ed25519Signature(Ed25519Signature),
}

enum Auth {
    Github,
}

enum PayoutMechanism {
    ProportionalPerConversion,
}
```

#### Campaign creation

Campaigns are created on any outpost from which the oracle picks them up to post
to the hub. Campaigns will only be relayed after they are funded on the outpost.
On the hub the oracle sends a campaign creation message.

```Rust
struct CampaignCreateMsg {
    id: u64,
    origin: ChainId,
    indexer: [u8; 32],
    attester: [u8; 32],
    segment_desc: SegmentDesc,
    conversion_desc: ConversionDesc,
    ends_at: UnixEpoch,
}
```

The `id` is assigned on the outpost.

The outpost from which the oracle picked up the event is identified via the `origin`.

Together the `id` and `origin` form the campaign id $c$, `origin || '-' || id`.
For example, for the neutron outpost the campaign id might be `neutron-1-23`.

The `indexer` MUST be registered in the indexer registry.

The `attester` MUST be registered in the attester registry.

The `segment_desc` describes the target for this campaign. The `sources` for these
segments are github repositories. The format MUST be `"org/repo"`, e.g.
`"bitcoin/bitcoin"`. The sources list MUST be at least of length one.
When selecting the `GithubTopNContributors` kind, n MUST be greater than zero.

Campaigns MUST have an end time `ends_at` after which conversions are no longer
possible. `ends_at` MUST be unix time and at least 86400 seconds greater than
the last valid block time.

Since campaigns are only relayed after funding on the outpost, after this message
is applied the `status` of the campaign MUST be set to `Funded`.

### Segments

Segments are collections of arbitrary data but for this prototype, we will assume
they are collections of github ids.

```Rust
struct Segment {
    data: SegmentData,
    proof: SegmentProof,
}

enum SegmentData {
    GithubSegment(GithubSegment),
}

struct GithubSegment {
    source: string,
    retrieved_at: UnixEpoch,
    data: Vec<String>,
}

```

XXX(pm): segments are embedded in campaign -> not re-usable in this version,
         also makes reads more expensive than they should be

#### Post segment

After a campaign is created on the hub it is picked up by an indexer who inspects
the `segment_desc`.

```Rust
struct SegmentCreateMsg {
    campaign_id: String,
    segments: Vec<Segment>,
    proof: SegmentProof,
}
```

The proof MUST be signed by an indexer whose ${pk}_i$ is registered in the
indexer registry.

The signature MUST be over the SHA256 hash of the serialized segment, i.e.
`hash(serialize(segment))`.

The `campaign_id` MUST exist in the campaign registry for this message to have
any effect.

If the campaign identified by `campaign_id` already has segment data attached,
then this message has no effect.

The produced segments MUST match the description provided by the campaign creator.

XXX(pm): previous point very fuzzy

After this message is applied, the `indexer` field of the campaign MUST be set
to the provers identity ${pk}_i$.

### Conversions

Conversions are tracked by attesters, streamed to the hub and then relayed to
outposts. It would be preferable if hub could assess the validity of addresses
but for the sake of simplicitly it is out of scope.

```Rust
struct Conversion {
    addr: Addr,
    payout: Coin,
}

enum ConversionProof {
    Ed25519Signature(Ed25519Signature),
}
```

#### Post conversion

```Rust
struct ConversionCreatedMsg {
    campaign_id: String,
    conversions: Vec<Conversion>,
    proof: ConversionProof,
}
```

XXX(pm): might make sense to use a Map instead of a Vec? can an address be used
         multiple times?

The proof MUST be signed by an attester whose ${pk}_a$ is registered in the
attester registry.

The signature MUST be over the SHA256 hash of the serialized conversions, i.e.
`hash(serialize(conversions))`.

The `campaign_id` MUST exist in the campaign registry for this message to have
any effect.

If the campaign identified by `campaign_id` already has conversion data attached
and the sender of this message is not the attester stored in the campaign then
this message has no effect.

If this conversion is the first for the campaign then after this message is applied,
the `attester` field of the campaign MUST be set to he author.

The `payout` MUST be calculated in accordance with the payout mechanism and incentive
budget recorded in the campaign.

### Oracle interactions

The trusted oracles are essential to keep the hub and outposts in sync. The
following section will outline how to. The execution environments are not
expected to be homogeneous so the messages are only to be taken as pseudo
code to outline the rough shape of outpost messages.

#### Campaign segment posted

After an indexer posts a segment data onchain the corresponding campaign on the
origin outpost needs to be updated.

```Rust
struct CampaignSegmentPostedMsg {}
```

XXX(pm): could also include root of merkle-ized segment to be able to provide
         rudimentary proofs for conversion but maybe overkill given that oracles
         are trusted

#### Campaign conversion posted

Whenever an attester posts new conversions on the hub, the oracle relays them
to the appropriate outpost, which should then trigger payouts immediately.

```Rust
struct CampaignConversionsPostedMsg {
    conversions: Vec<Conversion>,
}
```

#### Campaign conversion finished

The conversion process can either finish when all entries from the segment
converted or the campaing ran out of time. Once it is marked as finished
on the hub, an oracle updates the corresponding outpost.

```Rust
struct CampaignFinishedMsg {}
```
