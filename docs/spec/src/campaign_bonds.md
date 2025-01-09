# Campaign Bonds

The **Campaign Bond Protocol** is a decentralized, trustless mechanism designed to ensure that Delegates in the Filament Hub are fairly compensated for their participation in shaping campaign criteria and distributions, even if the Campaigner abandons the campaign. This bond ensures that Delegates, who may not trust the Campaigner, are financially protected.

### Motivation

In the Filament Hub, the Campaigner runs a campaign to distribute tokens to users based on criteria that Delegates help shape. Since the Delegates don't inherently trust the Campaigner, and the distribution only happens after the Delegates participate, there is a risk that Delegates may not get paid if the Campaigner decides to abandon the campaign. The **Campaign Bond** solves this problem by requiring the Campaigner to put up a bond in FILA tokens before starting a campaign, guaranteeing a minimum payment for each Delegate.

### Key Concepts

1. **Campaign Bond**: The Campaigner deposits FILA tokens into a smart contract as a bond before starting a campaign. This bond ensures that Delegates will receive at least a minimum payment even if the campaign is abandoned.
2. **Watermark**: The watermark represents the level of FILA that is locked as the bond for active campaigns. Locking and unlocking FILA tokens are governed by the watermark's movement.
3. **Bonding**: The process where the Campaigner locks FILA tokens in the bond smart contract.
4. **Unbonding**: The process of unlocking FILA tokens, subject to conditions related to the watermark and potential penalties (unbonding tax).
5. **Slashing**: If a campaign phase (such as Criteria or Distribution phase) times out, the locked portion of FILA corresponding to that campaign is slashed. The slashed amount is distributed to Delegates based on their voting power.

### How It Works

**Bonding**:

- Before starting a campaign, the Campaigner must lock FILA tokens in a smart contract, referred to as **bonded FILA**.
- The bond amount is calculated based on the number of Delegates and the **minimum delegate payment** (MIN_DELEGATE_PAYMENT).
- The required bond amount = NUM_DELEGATES * MIN_DELEGATE_PAYMENT.

**Locking**:

- When a campaign starts, FILA tokens equivalent to the bond amount are locked. This is represented by the watermark moving up.

**Unlocking**:

- When a campaign concludes successfully and the distribution is completed, FILA tokens are unlocked, represented by the watermark moving down.
- Unlocking means the Campaigner can withdraw the remaining FILA tokens not used for payments or slashing.

**Unbonding**:

- Unbonding happens when a Campaigner wants to withdraw FILA tokens from the bond.
- Unbonding outside the watermark incurs an **unbonding tax** unless it's initiated at a discounted rate (i.e., at a certain condition or event).
- The watermark ensures that enough FILA remains locked to cover the bond.

**Slashing**:

- If the campaign does not progress in time (e.g., if the Campaigner fails to meet a deadline in either the Criteria phase or Distribution phase), slashing occurs.
- **Slashing** deducts the bond amount for that campaign (NUM_DELEGATES * MIN_DELEGATE_PAYMENT).
- The slashed FILA tokens are distributed to Delegates based on their voting power.

### Spec Implementation

The following code exemplifies how the Campaign Bonding Protocol should work in Rust code

```rust
use std::collections::HashMap;

struct CampaignBond {
    total_bonded_fila: u64,       // Total FILA bonded in the protocol
    watermark: u64,               // Current watermark level (represents locked FILA)
    min_delegate_payment: u64,    // Minimum payment per delegate
    campaigns: HashMap<u64, Campaign>,  // Mapping of campaign ID to campaign details
}

struct Campaign {
    num_delegates: u32,                         // Number of delegates for this campaign
    delegate_voting_power: HashMap<String, u64>, // Delegate addresses and their voting power for this campaign
    locked_bond: u64,                           // Amount of FILA locked for this campaign
}

impl CampaignBond {
    // Create a new Campaign Bond instance
    pub fn new(min_delegate_payment: u64) -> Self {
        CampaignBond {
            total_bonded_fila: 0,
            watermark: 0,
            min_delegate_payment,
            campaigns: HashMap::new(),
        }
    }

    // Add a new campaign with its specific set of delegates and voting power
    pub fn add_campaign(&mut self, campaign_id: u64, num_delegates: u32, delegate_voting_power: HashMap<String, u64>) {
        let locked_bond = self.calculate_required_bond(num_delegates);
        self.campaigns.insert(
            campaign_id,
            Campaign {
                num_delegates,
                delegate_voting_power,
                locked_bond,
            },
        );
    }

    // Calculate the required bond for a campaign based on the number of delegates
    pub fn calculate_required_bond(&self, num_delegates: u32) -> u64 {
        num_delegates as u64 * self.min_delegate_payment
    }

    // Lock FILA for a new campaign (move the watermark up)
    pub fn lock_bond(&mut self, campaign_id: u64) -> Result<(), &'static str> {
        if let Some(campaign) = self.campaigns.get(&campaign_id) {
            if self.total_bonded_fila < campaign.locked_bond {
                return Err("Not enough FILA bonded for the campaign");
            }

            // Move the watermark up by the bond amount and deduct from total FILA
            self.watermark += campaign.locked_bond;
            self.total_bonded_fila -= campaign.locked_bond;

            Ok(())
        } else {
            Err("Campaign not found")
        }
    }

    // Unlock FILA when a campaign ends successfully (move the watermark down)
    pub fn unlock_bond(&mut self, campaign_id: u64) -> Result<(), &'static str> {
        if let Some(campaign) = self.campaigns.get(&campaign_id) {
            if self.watermark < campaign.locked_bond {
                return Err("Watermark too low to unlock");
            }

            // Move the watermark down and return FILA
            self.watermark -= campaign.locked_bond;
            self.total_bonded_fila += campaign.locked_bond;

            Ok(())
        } else {
            Err("Campaign not found")
        }
    }

    // Slash FILA when a campaign times out (distribute FILA to delegates based on voting power)
    pub fn slash_bond(&mut self, campaign_id: u64) -> Result<HashMap<String, u64>, &'static str> {
        if let Some(campaign) = self.campaigns.get(&campaign_id) {
            if self.watermark < campaign.locked_bond {
                return Err("Not enough FILA to slash");
            }

            // Move the watermark down
            self.watermark -= campaign.locked_bond;

            // Distribute FILA to delegates based on their voting power
            let total_voting_power: u64 = campaign.delegate_voting_power.values().sum();
            let mut slashed_distribution: HashMap<String, u64> = HashMap::new();

            for (delegate, voting_power) in &campaign.delegate_voting_power {
                let reward = (*voting_power as u64 * campaign.locked_bond) / total_voting_power;
                slashed_distribution.insert(delegate.clone(), reward);
            }

            Ok(slashed_distribution)
        } else {
            Err("Campaign not found")
        }
    }

    // Bond additional FILA (permissionless)
    pub fn bond_fila(&mut self, amount: u64) {
        self.total_bonded_fila += amount;
    }
}

fn main() {
    // Example usage
    let mut campaign_bond = CampaignBond::new(100);  // Minimum delegate payment = 100
    
    campaign_bond.bond_fila(1000);  // Bond some FILA

    let mut delegate_voting_power = HashMap::new();
    delegate_voting_power.insert("Delegate1".to_string(), 50);  // Add delegate with voting power
    delegate_voting_power.insert("Delegate2".to_string(), 50);

    // Add a new campaign with specific delegates
    campaign_bond.add_campaign(1, 2, delegate_voting_power);

    match campaign_bond.lock_bond(1) {
        Ok(_) => println!("Bond locked successfully"),
        Err(e) => println!("Error: {}", e),
    }

    // Unlock or slash as needed
}

```

### Conclusion

The **Campaign Bond Protocol** is designed to ensure that Delegates are compensated fairly, even in case of campaign abandonment. By bonding FILA tokens and managing the bond lifecycle through locking, unlocking, and slashing, the protocol creates a trustless mechanism for Campaigners and Delegates to interact without relying on the direct trust between them