use anyhow::{anyhow, bail, Result};
use sov_modules_api::{EventEmitter as _, Spec, TxState};

use crate::{
    campaign::{Campaign, Phase},
    criteria::{Criteria, CriteriaProposal},
    delegate::Eviction,
    segment::Segment,
    Core,
    Event,
    Power,
};

/// This enumeration represents the available call messages for interacting with
/// the `Core` module.
#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
    schemars(bound = "S::Address: ::schemars::JsonSchema", rename = "CallMessage")
)]
#[derive(
    Clone,
    Debug,
    PartialEq,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum CallMessage<S: Spec> {
    // campaign
    Init {
        criteria: Criteria,
        evictions: Vec<Eviction<S>>,
    },
    ProposeCriteria {
        campaign_id: u64,
        criteria: Criteria,
    },
    ConfirmCriteria {
        campaign_id: u64,
        proposal_id: Option<u64>,
    },
    RejectCriteria {
        campaign_id: u64,
    },
    IndexCampaign {
        campaign_id: u64,
    },
    PostSegment {
        campaign_id: u64,
        segment: Segment,
    },

    // Indexer
    RegisterIndexer(S::Address, String),
    UnregisterIndexer(S::Address),

    // Relayer
    RegisterRelayer(S::Address),
    UnregisterRelayer(S::Address),

    // Voting
    UpdateVotingPower(S::Address, Power),
}

impl<S: Spec> Core<S> {
    pub(crate) fn init_campaign(
        &self,
        criteria: Criteria,
        evictions: Vec<Eviction<S>>,
        sender: &S::Address,
        state: &mut impl TxState<S>,
    ) -> Result<u64> {
        tracing::info!(%sender, "Init campaign request");

        // TODO(xla): Expect bond and assert bond is locked for sender.
        // TODO(xla): Only accept if sender is a valid campaigner.
        // TODO(xla): Check that all Coins exist and amounts are payable.

        let campaign_id = self
            .next_campaign_id
            .get(state)?
            .ok_or(anyhow!("next_id is not set. This is a bug"))?;

        if self.campaigns.get(&campaign_id, state)?.is_some() {
            bail!("campaign id already exists. This is a bug - the id should be correctly incremented")
        }

        if criteria.is_empty() {
            bail!("missing criteria");
        }

        // TODO(xla): Compute list of proposed delegates based on matching criteria.
        let proposed_delegates = self.delegates.iter(state)?.collect::<Result<Vec<_>, _>>()?;

        if !evictions.iter().all(|e| proposed_delegates.contains(e)) {
            bail!("invalid eviction, only proposed delegates can be evicted");
        }

        let delegates = {
            let mut delegates = proposed_delegates.clone();
            delegates.retain(|d| !evictions.contains(d));
            delegates
        };

        // TODO(xla): Settle payment in case of evictions.
        // let mut payments = vec![];
        // if let Some(ref payment) = payment {
        //     payments.push(payment.clone());
        // }

        self.campaigns.set(
            &campaign_id,
            &Campaign {
                campaigner: sender.clone(),
                phase: Phase::Criteria,

                criteria,
                // budget,
                // payments,
                proposed_delegates,
                evictions: evictions.clone(),
                delegates,

                indexer: None,
            },
            state,
        )?;
        let mut ids = self
            .campaigns_by_addr
            .get(sender, state)?
            .unwrap_or_default();
        ids.push(campaign_id);
        self.campaigns_by_addr.set(sender, &ids, state)?;

        self.next_campaign_id.set(&(campaign_id + 1), state)?;

        self.emit_event(
            state,
            Event::CampaignInitialized {
                campaign_id,
                campaigner: sender.clone(),
                evictions,
            },
        );

        tracing::info!(%campaign_id, "Campaign initialized");

        Ok(campaign_id)
    }

    pub(crate) fn propose_criteria(
        &self,
        campaign_id: u64,
        criteria: Criteria,
        sender: &S::Address,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        tracing::info!(%sender, %campaign_id, "Criteria propose request");

        let campaign = self
            .campaigns
            .get(&campaign_id, state)?
            .ok_or(anyhow!("campaign '{campaign_id}' not found"))?;

        if campaign.phase != Phase::Criteria {
            bail!("invalid criteria proposal, campaign '{campaign_id}' is not in criteria phase");
        }

        if !campaign.delegates.contains(sender) {
            bail!("invalid proposer, '{sender}' is not a campaign delegate");
        }

        let mut proposals = self
            .criteria_proposals
            .get(&campaign_id, state)?
            .unwrap_or_default();
        let proposal_id = (proposals.len()) as u64;

        proposals.push(CriteriaProposal {
            campaign_id: proposal_id,
            proposer: sender.clone(),
            criteria,
        });

        self.criteria_proposals
            .set(&campaign_id, &proposals, state)?;

        self.emit_event(
            state,
            Event::CriteriaProposed {
                campaign_id,
                proposer: sender.clone(),
                proposal_id,
            },
        );

        tracing::info!(%campaign_id, %proposal_id, "Criteria proposed");

        Ok(())
    }

    pub(crate) fn confirm_criteria(
        &self,
        campaign_id: u64,
        proposal_id: Option<u64>,
        sender: &S::Address,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        tracing::info!(%sender, %campaign_id, "Criteria confirm request");

        let mut campaign = self
            .campaigns
            .get(&campaign_id, state)?
            .ok_or(anyhow!("campaign '{campaign_id}' not found"))?;

        if campaign.campaigner != *sender {
            bail!("sender '{sender}' is not the campaigner");
        }

        if campaign.phase != Phase::Criteria {
            bail!("invalid criteria proposal, campaign '{campaign_id}' is not in criteria phase");
        }

        campaign.phase = Phase::Publish;

        self.campaigns.set(&campaign_id, &campaign, state)?;

        self.emit_event(
            state,
            Event::CriteriaConfirmed {
                campaign_id,
                proposal_id,
            },
        );

        tracing::info!(%campaign_id, ?proposal_id, "Criteria proposed");

        Ok(())
    }

    pub(crate) fn reject_criteria(
        &self,
        _id: u64,
        _sender: &S::Address,
        _state: &mut impl TxState<S>,
    ) -> Result<()> {
        todo!()
    }

    pub(crate) fn index_campaign(
        &self,
        campaign_id: u64,
        sender: &S::Address,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        tracing::info!(%sender, %campaign_id, "Index campaign request");

        let mut campaign = self
            .campaigns
            .get(&campaign_id, state)?
            .ok_or(anyhow!("campaign '{campaign_id}' not found"))?;

        if campaign.indexer.is_none() || *sender != campaign.indexer.clone().unwrap() {
            bail!(
                "sender '{}' is not the registered indexer '{:?}' for campaign '{}'",
                sender,
                campaign.indexer,
                campaign_id
            );
        }
        if campaign.phase != Phase::Publish {
            bail!(
                "invalid campaign phase transition attempted for '{}' from '{:?}' to '{:?}'",
                campaign_id,
                campaign.phase,
                Phase::Indexing
            );
        }

        campaign.phase = Phase::Indexing;
        self.campaigns.set(&campaign_id, &campaign, state)?;

        self.emit_event(
            state,
            Event::CampaignIndexing {
                campaign_id,
                indexer: sender.clone(),
            },
        );

        tracing::info!(%campaign_id, %sender, "Campaign indexing");

        Ok(())
    }

    pub(crate) fn post_segment(
        &self,
        campaign_id: u64,
        segment: Segment,
        sender: &S::Address,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        tracing::info!(%sender, %campaign_id, "Post segment request");

        let mut campaign = self
            .campaigns
            .get(&campaign_id, state)?
            .ok_or(anyhow!("campaign '{campaign_id}' not found"))?;

        if campaign.indexer.is_none() || *sender != campaign.indexer.clone().unwrap() {
            bail!(
                "sender '{}' is not the registered indexer '{:?}' for campaign '{}'",
                sender,
                campaign.indexer,
                campaign_id
            );
        }
        if campaign.phase != Phase::Indexing {
            bail!(
                "invalid campaign phase transition attempted for '{}' from '{:?}' to '{:?}'",
                campaign_id,
                campaign.phase,
                Phase::Indexing
            );
        }
        if self.segments.get(&campaign_id, state)?.is_some() {
            bail!("segment for '{campaign_id}' exists");
        }

        campaign.phase = Phase::Distribution;
        self.campaigns.set(&campaign_id, &campaign, state)?;
        self.segments.set(&campaign_id, &segment, state)?;

        self.emit_event(
            state,
            Event::SegmentPosted {
                campaign_id,
                indexer: sender.clone(),
            },
        );

        tracing::info!(%sender, %campaign_id, "Segment posted");

        Ok(())
    }
}

// Indexer handlers.
impl<S: Spec> Core<S> {
    pub(crate) fn register_indexer(
        &self,
        indexer: S::Address,
        alias: String,
        sender: S::Address,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        tracing::info!(%indexer, ?alias, %sender, "Register indexer request");

        // Only allow admin to update registry for now.
        let admin = self
            .admin
            .get(state)?
            .ok_or(anyhow!("module admin is not set"))?;
        if sender != admin {
            bail!("sender '{sender}' is not an admin");
        }

        let indexers = self.indexers.iter(state)?.collect::<Result<Vec<_>, _>>()?;

        if !indexers.iter().any(|each| *each == indexer) {
            self.indexers.push(&indexer, state)?;
        }

        self.indexer_aliases.set(&indexer, &alias, state)?;

        self.emit_event(
            state,
            Event::<S>::IndexerRegistered {
                addr: indexer.clone(),
                alias: alias.clone(),
                sender: sender.clone(),
            },
        );
        tracing::info!(%indexer, ?alias, %sender, "Indexer registered");

        Ok(())
    }

    pub(crate) fn unregister_indexer(
        &self,
        indexer: S::Address,
        sender: S::Address,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        tracing::info!(%indexer, %sender, "Unregister indexer request");

        let admin = self
            .admin
            .get(state)?
            .ok_or(anyhow!("module admin is not set"))?;
        if sender != admin {
            bail!("sender '{sender}' is not an admin");
        }

        let pos = self
            .indexers
            .iter(state)?
            .collect::<Result<Vec<_>, _>>()?
            .iter()
            .position(|each| *each == indexer)
            .ok_or(anyhow!("indexer '{indexer}' is not registered"))?;
        self.indexers.remove(pos, state)?;
        self.indexer_aliases.remove(&indexer, state)?;

        self.emit_event(
            state,
            Event::IndexerUnregistered {
                addr: indexer.clone(),
                sender: sender.clone(),
            },
        );
        tracing::info!(%indexer, %sender, "Indexer unregistered");

        Ok(())
    }
}

// Relayer handlers.
impl<S: Spec> Core<S> {
    pub(crate) fn register_relayer(
        &self,
        relayer: S::Address,
        sender: S::Address,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        tracing::info!(%relayer, %sender, "Register relayer request");

        // Only allow admin to update registry for now.
        let admin = self
            .admin
            .get(state)?
            .ok_or(anyhow!("module admin is not set"))?;
        if sender != admin {
            bail!("sender '{sender}' is not an admin");
        }

        let relayers = self.relayers.iter(state)?.collect::<Result<Vec<_>, _>>()?;

        if !relayers.iter().any(|each| *each == relayer) {
            self.relayers.push(&relayer, state)?;
        }

        self.emit_event(
            state,
            Event::<S>::RelayerRegistered {
                addr: relayer.clone(),
                sender: sender.clone(),
            },
        );
        tracing::info!(%relayer, %sender, "Relayer registered");

        Ok(())
    }

    pub(crate) fn unregister_relayer(
        &self,
        relayer: S::Address,
        sender: S::Address,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        tracing::info!(%relayer, %sender, "Unregister Relayer request");

        let admin = self
            .admin
            .get(state)?
            .ok_or(anyhow!("module admin is not set"))?;
        if sender != admin {
            bail!("sender '{sender}' is not an admin");
        }

        let pos = self
            .relayers
            .iter(state)?
            .collect::<Result<Vec<_>, _>>()?
            .iter()
            .position(|each| *each == relayer)
            .ok_or(anyhow!("relayer '{relayer}' is not registered"))?;
        self.relayers.remove(pos, state)?;

        self.emit_event(
            state,
            Event::RelayerUnregistered {
                addr: relayer.clone(),
                sender: sender.clone(),
            },
        );
        tracing::info!(%relayer, %sender, "Relayer unregistered");

        Ok(())
    }
}

// Voting
impl<S: Spec> Core<S> {
    pub(crate) fn update_voting_power(
        &self,
        addr: S::Address,
        power: u64,
        sender: S::Address,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        tracing::info!(%addr, %power, %sender, "Update voting power request");

        // Only registered relayers are allowed to update voting power.
        self.relayers
            .iter(state)?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .find(|each| *each == sender)
            .ok_or(anyhow!("sender '{}' is not a registered relayer", sender))?;

        self.powers.set(&addr, &power, state)?;
        let mut index = self
            .powers_index
            .iter(state)?
            .collect::<Result<Vec<_>, _>>()?;

        if let Some((_, stored)) = index.iter_mut().find(|(stored, _)| *stored == addr) {
            *stored = power;
        } else {
            index.push((addr.clone(), power));
        }

        index.sort_unstable_by(|a, b| b.1.cmp(&a.1));

        self.powers_index.set_all(index, state)?;

        self.emit_event(
            state,
            Event::<S>::VotingPowerUpdated {
                addr: addr.clone(),
                power,
                relayer: sender.clone(),
            },
        );
        tracing::info!(%addr, %power, %sender, "Voting power updated");

        Ok(())
    }
}
