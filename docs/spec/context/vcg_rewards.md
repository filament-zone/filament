# VCG Rewards

## **Introduction**

In decentralized finance (DeFi), distributing rewards to users while preventing Sybil attacks is a significant challenge. Sybil attacks involve a malicious actor creating multiple fake identities to manipulate outcomes, such as receiving disproportionate rewards. To address this, we propose a mechanism that leverages the Vickrey-Clarke-Groves (VCG) auction principles to align delegates' incentives with the campaigner's objectives.

This mechanism allows campaigners to crowdsource the optimal criteria for reward distribution from delegates. Delegates vote on criteria, which are then weighted and combined to form the final criteria used for distributing rewards. The key challenge is attributing the economic activity (e.g., fees generated in a Uniswap V2 pool) to the initial seeding of rewards and rewarding delegates based on their contribution to maximizing the campaigner's objective.

---

## **System Specification**

### **Main Actors**

1. **Campaigner**
    - **Role:** Initiates the reward distribution campaign with a specific objective (e.g., maximize fees generated in a Uniswap V2 pool).
    - **Objective:** Allocate a budget to users in a way that maximizes the desired economic metric.
    - **Actions:** Defines the budget and the objective function.
2. **Delegates**
    - **Role:** Vote on the criteria for reward distribution by assigning weights.
    - **Objective:** Influence the criteria to maximize the campaigner's objective and earn commissions based on their contribution.
    - **Actions:** Assign weights to various criteria, stakes tokens to gain voting power.
3. **Users**
    - **Role:** Potential recipients of rewards based on the criteria.
    - **Objective:** Receive rewards and participate in activities that contribute to the economic metric.
    - **Actions:** Engage in activities (e.g., providing liquidity) that align with the campaigner's objective.
4. **Sybils**
    - **Role:** Malicious actors attempting to exploit the system by creating fake identities.
    - **Objective:** Receive undue rewards without contributing to the economic metric.
    - **Actions:** Create multiple accounts to appear as legitimate users.

### **Workflow Overview**

1. **Campaign Initialization**
    - The campaigner defines the budget  $B_T$ and the objective function $V_G$ (e.g., maximize fees generated in a specific pool).
    - Delegates are invited to participate by staking tokens to gain voting power.
2. **Criteria Proposal and Voting**
    - Delegates propose criteria $C_i$  (e.g., holding certain tokens, past activity).
    - Delegates assign weights  $w_{di}$ to each criterion, reflecting their belief in its effectiveness.
    - The delegates' votes are weighted by their voting power  $V_d$ (proportional to their stake).
3. **Criteria Aggregation**
    - The final criteria $C_f$  are computed as a weighted average of delegates' votes:
    $C_f = \frac{\sum_{d} V_d \times w_{d} \times C_d}{\sum_{d} V_d}$
4. **Reward Distribution**
    - Rewards are distributed to users based on the aggregated criteria  $C_f$ .
    - Users receive rewards $R_u$ proportional to their compliance with $C_f$ .
5. **Economic Activity Measurement**
    - After a predefined period, the actual economic metric  $V_G$ is measured.
    - The contribution of each delegate to  $V_G$  is estimated by evaluating the marginal impact of their votes.
6. **Delegate Rewarding using VCG Mechanism**
    - Delegates receive commissions $R_{D}$  based on their marginal contribution to  $V_G$.
    - The VCG mechanism ensures that delegates' optimal strategy is to vote truthfully to maximize  $V_G$.

---

## **Practical Mechanism for Rewarding Delegates**

### **Attributing Economic Activity to Delegates**

To reward delegates based on their contribution, we estimate the marginal impact of each delegate's vote on the campaigner's objective  $V_G$ . This involves:

- Calculating  $V_G$  with all delegates' votes.
- Calculating $V_{G}^{-d}$ without delegate  $d$'s vote.
- The marginal contribution  $\Delta V_{G}^{d} = V_G - V_{G}^{-d}$.

### **VCG-Based Delegate Rewarding**

The VCG mechanism rewards delegates based on their marginal contribution:

1. **Compute Marginal Contribution:**
    
    
    $\Delta V_{G}^{d} = V_G - V_{G}^{-d}$
    
2. **Calculate Delegate's Payment:**
    
    
    $R_{D}^{d} = \Delta V_{G}^{d} - P_d$
    
    Where $P_d$ is the payment made by delegate  $d$  to the system (can be zero if we only distribute positive rewards).
    
3. **Set Delegate Commission:**
    - To ensure incentive compatibility, delegates receive a commission proportional to their marginal contribution.
    - If their contribution is negative (i.e., they decreased  $V_G$, they receive no reward.

### **Reward Mapping Function**

- **Median User Reward:** Let   $R_{median}$  be the median reward given to users.
- **Delegate Commission:** Each delegate receives a commission $R_{D}^{d} = \gamma \times$ $R_{median}$, where  $\gamma$  is a scaling factor based on  $\Delta V_{G}^{d}$.

---

## **Code Examples in Rust**

Below are simplified Rust code examples demonstrating key components of the system.

### **1. Defining Criteria and Sybil Detection Methods**

```rust
// src/criteria.rs

use std::collections::HashMap;

#[derive(Clone)]
pub struct Criteria {
    pub name: String,
    pub weight: f64,
    // Additional parameters specific to the criterion
}

impl Criteria {
    pub fn new(name: &str, weight: f64) -> Self {
        Criteria {
            name: name.to_string(),
            weight,
        }
    }
}

// Example criteria functions
pub fn has_min_balance(address: &str, min_balance: f64) -> bool {
    // Implement logic to check if address has at least min_balance
    true // Placeholder
}

pub fn is_not_in_sybil_cluster(address: &str, sybil_clusters: &Vec<Vec<String>>) -> bool {
    // Implement logic using network analysis to check if address is in a Sybil cluster
    true // Placeholder
}

// Sybil detection using network analysis
pub fn detect_sybil_clusters(addresses: &Vec<String>) -> Vec<Vec<String>> {
    // Implement network analysis to detect clusters
    vec![] // Placeholder
}

```

### **2. Casting Votes and Combining Criteria**

```rust
// src/delegates.rs

use crate::criteria::Criteria;
use std::collections::HashMap;

pub struct Delegate {
    pub id: String,
    pub stake: f64,
    pub votes: HashMap<String, f64>, // Criteria name to weight
}

impl Delegate {
    pub fn new(id: &str, stake: f64) -> Self {
        Delegate {
            id: id.to_string(),
            stake,
            votes: HashMap::new(),
        }
    }

    pub fn vote(&mut self, criteria: &Criteria, weight: f64) {
        self.votes.insert(criteria.name.clone(), weight);
    }
}

// Combine delegates' votes to calculate final criteria
pub fn aggregate_criteria(delegates: &Vec<Delegate>) -> HashMap<String, f64> {
    let mut total_stake = 0.0;
    let mut weighted_votes: HashMap<String, f64> = HashMap::new();

    for delegate in delegates {
        total_stake += delegate.stake;
        for (criteria_name, weight) in &delegate.votes {
            let entry = weighted_votes.entry(criteria_name.clone()).or_insert(0.0);
            *entry += delegate.stake * weight;
        }
    }

    // Normalize the weights
    for value in weighted_votes.values_mut() {
        *value /= total_stake;
    }

    weighted_votes
}

```

### **3. Reward Distribution Based on Criteria**

```rust
// src/rewards.rs

use crate::criteria::{Criteria, has_min_balance, is_not_in_sybil_cluster};
use std::collections::HashMap;

pub fn distribute_rewards(
    users: &Vec<String>,
    criteria_weights: &HashMap<String, f64>,
    total_budget: f64,
) -> HashMap<String, f64> {
    let mut rewards: HashMap<String, f64> = HashMap::new();

    // Example: Equal distribution among users who meet criteria
    let mut eligible_users = vec![];

    // Placeholder sybil clusters detection
    let sybil_clusters = vec![]; // Should be obtained from network analysis

    for user in users {
        let mut score = 0.0;

        // Apply criteria
        if let Some(weight) = criteria_weights.get("min_balance") {
            if has_min_balance(user, 100.0) {
                score += weight;
            }
        }

        if let Some(weight) = criteria_weights.get("not_in_sybil_cluster") {
            if is_not_in_sybil_cluster(user, &sybil_clusters) {
                score += weight;
            }
        }

        // Add user if they meet the threshold
        if score >= 0.5 {
            eligible_users.push(user.clone());
        }
    }

    let reward_per_user = total_budget / eligible_users.len() as f64;

    for user in eligible_users {
        rewards.insert(user, reward_per_user);
    }

    rewards
}

```

### **4. Calculating Delegates' Contributions and Rewards**

```rust
// src/vcg.rs

use crate::delegates::Delegate;
use crate::rewards::distribute_rewards;
use std::collections::HashMap;

// Calculate the economic metric (e.g., fees generated)
pub fn calculate_economic_metric(users: &Vec<String>) -> f64 {
    // Placeholder implementation
    // In practice, sum up the fees generated by these users
    users.len() as f64 * 10.0 // Assume each user generates 10 units
}

// Calculate delegate contributions using VCG mechanism
pub fn calculate_delegate_rewards(
    delegates: &Vec<Delegate>,
    users: &Vec<String>,
    total_budget: f64,
    median_user_reward: f64,
) -> HashMap<String, f64> {
    let mut delegate_rewards: HashMap<String, f64> = HashMap::new();

    // Calculate V_G with all delegates
    let criteria_weights = aggregate_criteria(delegates);
    let rewards = distribute_rewards(users, &criteria_weights, total_budget);
    let v_g = calculate_economic_metric(&rewards.keys().cloned().collect());

    for delegate in delegates {
        // Calculate V_G without delegate d
        let mut delegates_minus_d = delegates.clone();
        delegates_minus_d.retain(|d| d.id != delegate.id);

        let criteria_weights_minus_d = aggregate_criteria(&delegates_minus_d);
        let rewards_minus_d = distribute_rewards(users, &criteria_weights_minus_d, total_budget);
        let v_g_minus_d = calculate_economic_metric(&rewards_minus_d.keys().cloned().collect());

        let delta_v_g = v_g - v_g_minus_d;

        // Delegate reward proportional to their marginal contribution
        let delegate_reward = if delta_v_g > 0.0 {
            // Scaling factor can be adjusted
            delta_v_g * median_user_reward / v_g
        } else {
            0.0
        };

        delegate_rewards.insert(delegate.id.clone(), delegate_reward);
    }

    delegate_rewards
}

```

---

## **Incentive Mechanism Details**

### **Objective Function Attribution**

- **Economic Metric $V_G$** : The total fees generated in the Uniswap V2 pool by the users who received rewards.
- **Marginal Contribution  $\Delta V_{G}^{d}$** : The difference in $V_G$ when delegate $d$ 's vote is included versus excluded.

### **Ensuring Incentive Compatibility**

- **Truthful Voting as Dominant Strategy**: By rewarding delegates based on their marginal contribution, they are incentivized to vote in a way that maximizes  $V_G$.
- **Sybil Resistance**: Criteria that effectively exclude Sybils will result in higher $V_G$, benefiting delegates who promote such criteria.

### **Delegates' Expected Payoff**

$E[P_D^d] = R_D^d - C_S^d$

Where:

- $E[P_D^d]$: Expected payoff for delegate d.
- $R_D^d$: Reward received based on marginal contribution.
- $C_S^d$: Cost incurred by delegate  $d$ (e.g., staking cost).

### **Soundness of the System**

Under the assumptions:

- **Accurate Measurement**: The economic metric  $V_G$ can be accurately attributed to the rewarded users.
- **Rational Delegates**: Delegates aim to maximize their expected payoff.
- **Sybil Detection Effectiveness**: Criteria can effectively distinguish between genuine users and Sybils.

**Proof Sketch:**

1. **Delegates' Optimal Strategy**: Since delegates are rewarded based on their marginal contribution to  $V_G$, their optimal strategy is to vote for criteria that maximize  $V_G$ .
2. **Truthful Voting**: The VCG mechanism ensures that truthful voting (i.e., accurately reflecting the effectiveness of criteria) maximizes a delegate's expected payoff.
3. **Sybil Deterrence**: Including criteria that effectively detect and exclude Sybils increases $V_G$, thus increasing delegates' rewards.
4. **System Soundness**: By aligning delegates' incentives with the campaigner's objective, the system encourages behaviors that maximize $V_G$, ensuring that rewards are distributed to users who contribute to the economic metric.

---

## **Conclusion**

By integrating the VCG mechanism into the reward distribution system, we align delegates' incentives with the campaigner's objective of maximizing an economic metric. Delegates are rewarded based on their marginal contribution to the objective, encouraging them to vote for criteria that effectively promote genuine user participation and exclude Sybils.

This mechanism fosters a collaborative environment where all actors work towards a common goal, enhancing the overall efficiency and security of the decentralized platform.

---

## **Further Considerations**

- **Scalability**: The system must handle a large number of delegates and users efficiently.
- **Data Privacy**: Ensure that user data used in criteria evaluation is handled securely.
- **Robustness**: The system should be resilient to attempts at manipulation by malicious delegates or users.

---

---

---