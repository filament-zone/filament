# Campaign Fees [draft]

*This page is very much incomplete and should not be reviewed*

The campaigner pays fees at different stages of the protocol based on actors the need to pay. This page describes how these fees are calculated.

### Objectives

- It should always be beneficial to the system to complete the the campaign
    - `bond >= yield >= commission`
- Proposals should have additional incentives

### Init#Commit

In this stage, a list of delegates who are being evicted is processed. For each evicted delegate, a fixed cost equal to `MIN_DELEGATE_PAYMENT` is applied. The fees collected from this process are distributed proportionately to the voting power of the evicted delegates.

```rust
struct InitCommitFee {
    delegate_fees: HashMap<String, u64>, // Address to fee amount mapping
}

fn calc_init_commit_fee(
    evicted_delegates: Vec<Delegate>, 
    min_delegate_payment: u64) -> InitCommitFee {
    let total_voting_power: u64 = evicted_delegates.iter().map(|d| d.voting_power).sum();
    let mut fee_distribution = HashMap::new();

    for delegate in evicted_delegates {
        let proportional_fee = (delegate.voting_power as u64 *               min_delegate_payment) / total_voting_power;
        fee_distribution.insert(delegate.address.clone(), proportional_fee);
    }

    InitCommitFee {
        delegate_fees: fee_distribution,
    }
}
```

### Criteria#Commit

During the `Criteria#Commit` phase, if a proposal is accepted from a delegate and not the campaigner, the delegate receives a `PROPOSAL_FEE`. The fee is paid directly to the delegate who made the proposal.

```rust
struct Proposal {
    proposer_address: String,
    is_delegate: bool,
}

struct CriteriaCommitFee {
    recipient: String,
    amount: u64,
}

fn criteria_commit(proposal: Proposal, proposal_fee: u64) -> Option<CriteriaCommitFee> {
    if proposal.is_delegate {
        Some(CriteriaCommitFee {
            recipient: proposal.proposer_address,
            amount: proposal_fee,
        })
    } else {
        None
    }
}
```

### Criteria#Reject

When a proposal is rejected, half of the bond is burned by sending it to a designated `BURN_ADDRESS`, and the other half is distributed proportionally to the voting delegates based on their voting power.

```rust
const BURN_ADDRESS: &str = "0x000000000000000000000000000000000000dead";

struct Fee {
    burn_amount: u64,
    delegate_fees: HashMap<String, u64>, // Address to fee amount mapping
}

fn criteria_reject(bond: u64, voting_delegates: Vec<Delegate>) -> Fee {
    let burn_amount = bond / 2;
    let distribution_amount = bond / 2;
    let total_voting_power: u64 = voting_delegates.iter().map(|d| d.voting_power).sum();
    
    let mut delegate_fees = HashMap::new();
    
    for delegate in voting_delegates {
        let proportional_fee = (delegate.voting_power * distribution_amount) / total_voting_power;
        delegate_fees.insert(delegate.address.clone(), proportional_fee);
    }
    
    Fee {
        burn_amount,
        delegate_fees,
    }
}
```

### Publish#Commit

```rust
struct Proposal {
    proposer_address: String,
    is_alternative: bool,
}

struct PublishCommitFee {
    recipient: String,
    amount: u64,
}

fn publish_commit(proposal: Proposal, proposal_fee: u64) -> Option<publishCommitFee> {
    if proposal.is_delegate {
        Some(PublishCommitFee {
            recipient: proposal.proposer_address,
            amount: proposal_fee,
        })
    } else {
        None
    }
}
```

### Publish#Reject

- we need to reject

```rust
const BURN_ADDRESS: &str = "0x000000000000000000000000000000000000dead";

struct PublishRejectFee {
    burn_amount: u64,
    ...
}

fn publish_reject(bond: u64, voting_delegates: Vec<Delegate>) -> PublishRejectFee {
		...
    
    Fee {
			...
    }
}
```

### Distribution#Commit

- INCENTIVES
    - Proposals should be paid extra if accepted

fn distribution_commit_fee(...) -> Fee

### Distribution#Reject

- INCENTIVES
    - Burn half bond and return rest to the delegates
    - Distribute half the bond voting delegates proportionate to their voting power

```rust
fn distribution_reject_fee(...) -> Fee
```

### 

## Invariants

And we expect the yield to be between these two values

- This is true
- as timeouts distribute the bond and rejects burn half the