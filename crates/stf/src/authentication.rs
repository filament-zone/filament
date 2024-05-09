//! The hub-stf supports `sov-module` authenticators.
use std::marker::PhantomData;

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use sov_modules_api::{
    runtime::capabilities::{AuthenticationError, RawTx, RuntimeAuthenticator},
    transaction::AuthenticatedTransactionAndRawHash,
    Authenticator,
    DaSpec,
    DispatchCall,
    Spec,
};

use crate::runtime::Runtime;

impl<S: Spec, Da: DaSpec> RuntimeAuthenticator for Runtime<S, Da> {
    type Decodable = <Self as DispatchCall>::Decodable;
    type Tx = AuthenticatedTransactionAndRawHash<S>;

    fn authenticate(
        &self,
        raw_tx: &RawTx,
    ) -> Result<(Self::Tx, Self::Decodable), AuthenticationError> {
        sov_modules_api::authenticate::<S, Self>(&raw_tx.data)
    }
}

#[derive(Debug, PartialEq, Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
enum Auth {
    Evm(Vec<u8>),
    Mod(Vec<u8>),
}

/// Authenticator for the sov-module system.
pub struct ModAuth<S: Spec, Da: DaSpec> {
    _phantom: PhantomData<(S, Da)>,
}

impl<S: Spec, Da: DaSpec> Authenticator for ModAuth<S, Da> {
    type DispatchCall = Runtime<S, Da>;
    type Spec = S;

    fn authenticate(
        tx: &[u8],
    ) -> Result<
        (
            AuthenticatedTransactionAndRawHash<Self::Spec>,
            <Self::DispatchCall as DispatchCall>::Decodable,
        ),
        AuthenticationError,
    > {
        sov_modules_api::authenticate::<Self::Spec, Self::DispatchCall>(tx)
    }

    fn encode(tx: Vec<u8>) -> Result<RawTx, anyhow::Error> {
        Ok(RawTx { data: tx })
    }
}
