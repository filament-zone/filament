# Campaign Execution

The Filament Hub is responsible for coordinating a decentralized set of actors as they collaborate to execute campaigns. Campaigners, Delegates and  Indexers effectuate change in a state machine who's parameters we define semi formally here.  The following describes the actors,  data structures,  state transitions and invariants necessary for a correct implementation of the protocol.

## Actors

- **Campaigners:** Provides the budget to be distributed
- **Delegates:** Contribute to consensus
- **Indexers:** Data provider
- **Protocol:** A trusted oracle which coordinates specific state transitions

## Invariants

**Liveness:** Actors participating in the protocol are staked. Campaigner stake the budget, delegates have staked  produce voting power, indexers are staked. Staking is done with $FILA. Actors who do not adhere to the protocol are slashed. Phases of the protocol have a maximum duration and timeouts are used to ensure terminate. A trusted oracle called `Protocol` will execute timeouts.

The campaign must terminate with the `Settle` phase. Errors occurring throughout the campaign will always direct to a `Settle` Phase.

## Parameters

The following parameters are set by governance:

`MAX_EVICTIONS` The maximum number of delegates which can be evicted from the campaign

`EVICTION_PRICE` The cost payed by the campaigner to evict a delegate

`MIN_CRITERIA_QUORUM` The  minimum amount of voting power to achieve quorum on Criteria

`MIN_DISTRIBUTION_QUORUM` The minimum amount of voting power required to achieve quorum on Distribution

# Data Structures

The following section describes the data structures and their purpose used throughout the spec.

### Campaign

The campaign is a data structure management by the filament hub state machine. A campaign proceeds in phases: Init, Criteria, Publish , Distribution and Settle. A campaign is initialized with a budget which is locked until the campaign concludes in the Settle phase. The Criteria is produced during the criteria phase (parameterized by variables).  Payments store list of payments which are cleared during the Settle Phase. Each campaign has it's own set of Delegates which are confirmed during the Init phase.

```rust
struct Budget {
    comission ... // $CAMPAIGNER denomianted
    gas       ... // $FILA denominated
}

// queue of payments to be issued
struct Payments {
    ...
}

enum Phase {
    Init(...)
    Criteria(...)
    Publish(...)
    Distribution(...)
    Settle(...)
}

struct Campaign {
    phase     Phase
    budget    Budget
    payments  Vec<Payment>
    delegates Vec<Delegates>
    variables Vec<Variable>
}
```

### Criteria

A campaign is composed of Criteria which is a set of Criterion. Each Criterion refers a uniquely defined datasets and parameters. For instance "osmosis LP positions from snapshot block 10000"

In practice Criteria are defined in playbooks which are parsed and stored by the state machine

```rust
struct Criterion {
    dataset_id DatasetID
    parameters HashMap<Field, Predicate>
}

type Criteria Vec<Criterion>

impl Criteria {
    fn new(Playbook) -> Criteria
}
```

### Variables

*This is a terrible name and should be more specific*

Variables are specific predicates which parametrize Criterion which allow the Campaigner to provide specific input. For instance: snapshot date. Variables are provided by the campaigner at the end of the Criteria phase when the the criteria is finalized.

```rust
type Variables = HashMap<Criterion, Value>
```

### Segment

Each Criterion materializes into a segment when published by an indexer during the Publish phase. Segments map a domain address to a specific amount of rewards for that user. A domain for instance is "Osmosis".

```rust
// Mapping of addresses to amounts
type Segment<Domain> = HashMap<Address<Domain>, Amount>
```

### Distribution

```rust
// The final distribution of rewards for a campaign
type Distribution = Segment<Outpost>
```

The final Distribution is calculated during the Distribution phase and is represents a mapping of the addresses is the Outpost domain to that users final allocation.

# Voting

Two phases of the protocol require delegates to vote, Criteria and Distribution. The only difference between the phases is the shape of the Proposal. The voting process describes here applies to both phases and uses general terms (Vote, Proposal, Tally, etc) but can be understood to apply to both.

Voting is cast for or against a Proposal. Each vote is associated with the voting power of the delegate who cast the vote.

```rust
type Proposal = ...
type Power    = ...

struct Vote {
    voter Delegate
    power Power
}
```

Voting revolves around a Tally. A tally is configured with a quorum which determines when the specific voting process has achieved enough voting power.

```rust
struct Tally {
    quorum     usize
    predicates Vec<Predicates>
    proposals  Vec<Proposal>
}

impl Tally {
    fn vote(proposal: Proposal, vote: Vote) -> ... {
        if proposal in proposals {
            proposal.vote(vote);
        } else {
            proposals.add(self.predicates.apply(proposal));
        }
    }

    fn has_quorum(proposal Proposal) -> bool {
        if let p = proposals.fetch(proposal) && p.power > quorum {
            return true
        } else {
            return false
        }
    }
}
```

Each Tally can have one or many Proposals. Votes that are cast for a specific proposal. Each voters can vote on exactly one proposal but voters can change their vote.

```rust
type Predicate = fn(Proposal) => Vote
```

# Phases

The following describes the phases of the protocol in terms of how messages are handled and the post conditions of state transitions.

Message handlers are defined as functions annotated by authorization indicating which actor is authorized to perform the specific action.

```rust
#[authorized(Actor)]
fn message_handler(Parameters...) -> Transition<...>
```

### **Phase Overview**

```rust
Init => Criteria => Publish => Distribution => Settle
```

**Init:** Initiate the campaign with a locked budget and elected delegates

**Criteria:**  Determine the criteria

**Publish:** Materialize the segment data

**Distribution:** Vote on the Distribution

**Settle:** Release bonded resources and end the campaign

# Init

The campaigner Initializes the campaign.

### Elect

- The campaigner provides a Criteria
- The Protocol produces a candidate list of Delegates

```rust
#[authorized(Campaigner)]
fn elect(Budget,Criteria) -> ...
```

### Confirm

- The protocol calculates the cost of evicting
- The campaigner selects a list of delegates to evict which does not exceed `MAX_EVICTIONS`
- The evicted delegates are paid Payment
- Upon execution of confirm `CriteriaTimer` start

```rust
#[authorized(Campaigner)]
fn confirm(Vec<Delegates>, Payment) -> Transition<Phase::Criteria<...>>
```

### Post Condition

- Delegates are locked
- Budget is locked

# **Criteria**

Determine the Criteria for the Campaign

### Vote

- State Machine calculates the predicate votes and initiates phase specific Tally (not predicate votes yet)
- Delegate vote on Criteria with and the staking weights are applied

```rust

struct Vote {
	Delegate: ...
	Weights: Vec<f32>
}
// Delegates vote on the proposed Criteria
#[authorized(Delegate)]
fn vote(Vote) -> ...
```

### Confirm

- Campaigner confirms the proposed Criteria
- The campaigner must confirm a Proposal which has met quorum
- The campaigner provides the variables (eg. snapshot dates) which inform the indexer on how to query the data
- The protocol calculates the assignment (which indexers must submit which segments)
- The protocol calculates the min_payment for the indexers (according to the directory)
- The protocol calculates the payment for the indexers to produce the segment which must be included in the confirmation
- After successful Confirm, `PublishTimer` starts

```rust
// Campaigner confirms the criteria
#[authorized(Campainger)]
fn confirm(Weights, Payment, Variables) -> Transition<Phase::Publish<...>
```

### Reject

- The Campaigner rejects the proposed Criteria
- The protocol calculates the minimum payment to reject the proposal
- The protocol transitions to the Settle Phase to process payments and release resources

```rust
#[authorized(Campainger)]
fn reject(Payment) -> Transition<Phase::Settle<...>>
```

### Timeout

- The protocols times out
- The Delegates who failed to vote are slashed
- The protocol proceeds to the Settle Phase to issue payments and release resources

```rust
// the phase exhausts it's duration
#[authorized(Protocol)]
fn timeout(...) -> Transition<Phase::Settle<...>>
```

### Post Condition

- Indexers locked

# Publish

This phase materializes the segments on the hub

### Publish

- indexers submit indexers
- indexers must be registered and staked
- multiple indexers can submit competing segments create a dispute
- Upon success PublishTimer stops

```rust
// Indexers publish segments
#[authorized(Indexer]
fn receive(Segment) -> Transition<Phase::Distribution<...>>
```

### Confirm

- Campaigner confirms the segments with a Resolution
- Resolution provides selection of segment amongst competing segments (if any)
- Upon confirm `DistributionTimer` starts

```rust
// Arbitrating Delegate resolves the dispute
#[authorized(Campaigner]
fn confirm(Resolution) -> Transition<Phase::Distribution<...>>
```

### Timeout

- The protocol can timeout
- indexers who were assigned but fail to publish their assigned segments are slashed

```rust
// State Machine terminates the phase after TIMEOUT has expired
#[authorized(Protocol]
fn timeout(...) -> Transition<Phase::Settle<...>>
```

### Post Condition

- All segments are available stored in the Filament Hub verifiable state

# Distribution

Determine the distribution of rewards

### Vote

- Delegates vote on proposed distribution
- Delegates can provide alternative distribution

```rust
// Delegates vote on the proposed Distribution
#[authorized(Delegate)]
fn vote(Proposal, Vote) -> ...
```

### Confirm

- The campaigner confirms proposed distribution
- The protocol calculates the price of confirming the proposal
- The protocol calculates the quorum of each proposed distribution
    - The campaigner must confirm proposal which has achieved quorum

```rust
// Campaigner confirms the criteria
#[authorized(Campainger)]
fn confirm(Proposal, Payment, Variables) -> Transition<Phase::Settle<...>>
```

### Reject

```rust
#[authorized(Campainger)]
fn reject(Payment) -> Transition<Phase::Settle<...>>
```

### Timeout

- The Delegates who failed to vote are slashed
- The protocol proceeds to the Settle Phase to issue payments and release resources

```rust
// the phase exhausts it's duration
#[authorized(Protocol)]
fn timeout(...) -> Transition<Phase::Settle<...>>
```

### Post Conditions

- The Distribution is finalized
- Indexers are released
- delegates are released

# Settle

This phase makes payments

- The protocol keeps track of which payments have been made
- Payments which occur in the outpost are attested to the by a protocol oracle

### Clear

```rust
// The state machine receives settlement messages confirming that payments have been processed
#[authorize(Protocol)]
fn clear(Payment) -> ...
```

### Post Conditions

- All payments have been processed
- Segment data has been purged from the state machine
