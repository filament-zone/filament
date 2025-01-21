# Economic Model

The Filament economic model coordinates network participants through a system of tokens, bonds, fees, and incentives. This section details the economic mechanisms that secure the network and align participant incentives.

## Token Economics ($FILA)

### Overview
$FILA is the native token of the Filament network serving multiple purposes:
- Staking for delegates and indexers
- Campaign bonds and payments
- Protocol governance
- Network security

### Supply & Distribution
- Fixed total supply
- Initial distribution through token generation event
- Ongoing distribution through campaign rewards and delegate commissions
- Treasury allocation for protocol development and incentives

### Gas Token ($FILUM)
$FILUM serves as the computational gas token for the Filament Hub:

1. **Allocation**
   ```rust
   struct FilumAllocation {
       epoch: u64,
       emissions_per_epoch: u64,
       max_balance: u64,
       min_bond: u64
   }
   ```

2. **Usage**
   - Non-transferable utility token
   - Assigned periodically to bonded accounts
   - Maximum balance cap per account
   - Used for transaction fees on the Hub

## Treasury Window

The Treasury Window provides reliable FILA acquisition for campaigners:

1. **Exchange Mechanism**
   ```rust
   struct TreasuryWindow {
       exchange_rate: f64,  // FILA/USDC rate
       discount: f64,       // Current discount
       limit: u64          // Maximum FILA available
   }
   ```

2. **Rate Calculation**
   ```rust
   fn calculate_rate() -> f64 {
       let twap = get_fila_twap("7days");
       twap * DISCOUNT_FACTOR
   }
   ```

3. **Benefits**
   - Predictable campaign costs
   - Reduced price impact
   - Treasury USDC accumulation

## Campaign Bonds

Campaign bonds ensure campaigner commitment and delegate compensation:

1. **Bond Requirements**
   ```rust
   struct CampaignBond {
       total_bonded: u64,
       watermark: u64,
       min_delegate_payment: u64
   }
   ```

2. **Bonding Rules**
   - Minimum bond proportional to delegate count
   - Watermark system for managing multiple campaigns
   - Slashing conditions for timeout or abandonment

3. **Bond Release**
   - Gradual unbonding after campaign completion
   - Slashing for misbehavior
   - Watermark-based unlocking

## Commission Structure

Delegates earn commission for campaign participation:

1. **Commission Calculation**
   ```rust
   struct Commission {
       median_reward: u64,
       commission_multiple: f64,
       delegate_carveout: f64
   }
   ```

2. **Distribution Rules**
   - Commission based on median reward
   - Delegate carveout percentage
   - Delegator reward distribution

3. **Participation Requirements**
   - Active voting requirement
   - Minimum stake threshold
   - Performance metrics

## Fee Model

### Transaction Fees

1. **Gas Costs ($FILUM)**
   ```rust
   struct GasCost {
       base_fee: u64,
       computation_fee: u64,
       storage_fee: u64
   }
   ```

2. **Campaign Fees**
   - Init phase fee
   - Criteria confirmation fee
   - Distribution confirmation fee

### Phase-Specific Fees

1. **Init Phase**
   ```rust
   fn calculate_init_fee(
       num_delegates: u32,
       evictions: u32
   ) -> u64 {
       BASE_FEE +
       (num_delegates * DELEGATE_FEE) +
       (evictions * EVICTION_FEE)
   }
   ```

2. **Criteria Phase**
   ```rust
   fn calculate_criteria_fee(
       proposal: Proposal,
       outcome: Outcome
   ) -> u64 {
       match outcome {
           Outcome::Accept => ACCEPT_FEE,
           Outcome::Reject => calculate_reject_fee(proposal)
       }
   }
   ```

3. **Distribution Phase**
   ```rust
   fn calculate_distribution_fee(
       distribution: Distribution,
       outcome: Outcome
   ) -> u64 {
       BASE_FEE +
       (distribution.recipients.len() * RECIPIENT_FEE)
   }
   ```

## VCG Mechanism

The VCG (Vickrey-Clarke-Groves) mechanism optimizes reward distribution:

1. **Objective Function**
   ```rust
   struct Objective {
       metric: EconomicMetric,
       weight: f64,
       target: f64
   }
   ```

2. **Delegate Contribution**
   ```rust
   fn calculate_marginal_contribution(
       delegate: Delegate,
       metric: EconomicMetric
   ) -> f64 {
       let value_with = calculate_value_with_delegate(delegate);
       let value_without = calculate_value_without_delegate(delegate);
       value_with - value_without
   }
   ```

3. **Reward Allocation**
   ```rust
   fn calculate_vcg_reward(
       contribution: f64,
       median_reward: u64
   ) -> u64 {
       (contribution * median_reward as f64) as u64
   }
   ```

## Economic Security

### Stake Requirements

1. **Delegate Staking**
   - Minimum stake threshold
   - Unbonding period
   - Slashing conditions

2. **Indexer Staking**
   - Data quality bonds
   - Performance requirements
   - Dispute resolution stakes

### Slashing Conditions

1. **Timeout Slashing**
   ```rust
   fn calculate_timeout_slash(
       stake: u64,
       severity: SlashSeverity
   ) -> u64 {
       match severity {
           SlashSeverity::Minor => stake / 10,
           SlashSeverity::Major => stake / 2,
           SlashSeverity::Critical => stake
       }
   }
   ```

2. **Misbehavior Slashing**
   - Invalid submissions
   - Malicious proposals
   - Coordination attacks

### Incentive Alignment

1. **Campaigner Incentives**
   - Bond requirement ensures commitment
   - Fee structure encourages completion
   - VCG mechanism optimizes outcomes

2. **Delegate Incentives**
   - Commission rewards participation
   - Stake risk ensures quality
   - VCG rewards effective criteria

3. **Indexer Incentives**
   - Data quality bonds
   - Performance-based rewards
   - Reputation systems
