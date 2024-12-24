//! The stf-rollup supports `sov-module` authenticator. To support other authentication schemes,
//! you can check out how we support `EVM` authenticator here:
//! https://github.com/Sovereign-Labs/sovereign-sdk-wip/blob/146d5c2c5fa07ab7bb59ba6b2e64690ac9b63830/examples/demo-rollup/stf/src/authentication.rs#L29-L32
use std::marker::PhantomData;

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use sov_modules_api::{
    capabilities::{AuthenticationResult, AuthorizationData, UnregisteredAuthenticationError},
    runtime::capabilities::TransactionAuthenticator,
    DaSpec,
    DispatchCall,
    PreExecWorkingSet,
    RawTx,
    Spec,
    UnlimitedGasMeter,
};
use sov_sequencer_registry::SequencerStakeMeter;

use crate::runtime::{Runtime, RuntimeCall};

impl<S: Spec, Da: DaSpec> TransactionAuthenticator<S> for Runtime<S, Da> {
    type AuthorizationData = AuthorizationData<S>;
    type Decodable = <Self as DispatchCall>::Decodable;
    type Input = Auth;
    type SequencerStakeMeter = SequencerStakeMeter<S::Gas>;

    fn authenticate(
        &self,
        input: &Self::Input,
        pre_exec_ws: &mut PreExecWorkingSet<S, Self::SequencerStakeMeter>,
    ) -> AuthenticationResult<S, Self::Decodable, Self::AuthorizationData> {
        match input {
            Auth::Mod(tx) => {
                match filament_hub_eth::authenticate::<S, Self, Self::SequencerStakeMeter>(
                    tx,
                    pre_exec_ws,
                ) {
                    Ok(res) => return Ok(res),
                    Err(err) => tracing::error!(%err, "failed to authenticate eth transaction"),
                }

                sov_modules_api::capabilities::authenticate::<S, Self, Self::SequencerStakeMeter>(
                    tx,
                    pre_exec_ws,
                )
            },
        }
    }

    fn authenticate_unregistered(
        &self,
        raw_tx: &Self::Input,
        pre_exec_ws: &mut PreExecWorkingSet<S, UnlimitedGasMeter<S::Gas>>,
    ) -> AuthenticationResult<
        S,
        Self::Decodable,
        Self::AuthorizationData,
        UnregisteredAuthenticationError,
    > {
        let Auth::Mod(contents) = raw_tx;

        let (tx_and_raw_hash, auth_data, runtime_call) =
            sov_modules_api::capabilities::authenticate::<
                S,
                Runtime<S, Da>,
                UnlimitedGasMeter<S::Gas>,
            >(contents, pre_exec_ws)?;

        match &runtime_call {
            RuntimeCall::SequencerRegistry(sov_sequencer_registry::CallMessage::Register {
                ..
            }) => Ok((tx_and_raw_hash, auth_data, runtime_call)),
            _ => Err(UnregisteredAuthenticationError::RuntimeCall)?,
        }
    }

    fn add_standard_auth(tx: RawTx) -> Self::Input {
        Auth::Mod(tx.data)
    }
}

#[derive(Debug, PartialEq, Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub enum Auth {
    Mod(Vec<u8>),
}

pub struct ModAuth<S: Spec, Da: DaSpec> {
    _phantom: PhantomData<(S, Da)>,
}
