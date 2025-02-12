## Outposts
The Outpost manages the financial aspects of campaigns, including budget management, incentive distribution, and fee handling on Neutron. It implements a state machine that tracks campaign progress and ensures proper distribution of rewards according to campaign rules.

## Campaign Lifecycle
The contract manages campaigns through several states:
1. Created -> Initial campaign setup
2. Funded -> Budget locked and ready for execution
3. Indexing -> Data collection phase
4. Attesting -> Verification of conversions
5. Finished -> Campaign completed successfully
6. Canceled/Failed -> Terminal states for unsuccessful campaigns

## Key Features

### Campaign Management
```rust,ignore
pub struct Campaign {
    pub admin: Addr,
    pub status: CampaignStatus,
    pub budget: Option<CampaignBudget>,
    pub spent: u128,
    pub indexer: Addr,
    pub attester: Addr,
    pub segment_desc: SegmentDesc,
    pub segment_size: u64,
    pub conversion_desc: ConversionDesc,
    pub payout_mech: PayoutMechanism,
    pub ends_at: u64,
    pub fee_claimed: bool,
}
```

### Distribution Mechanics
- Supports proportional distribution per conversion
- Handles budget tracking and spending limits
- Manages fee distribution between indexers, attesters, and protocol

### Security Features
- Role-based access control (admin, indexer, attester)
- Budget validation and tracking
- Conversion verification and duplicate prevention
- Deadline enforcement

The Outpost serves as a critical component in the Filament ecosystem by providing secure and verifiable token distribution mechanics on the Neutron blockchain while maintaining alignment with the Hub's campaign coordination.
