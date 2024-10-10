use std::hash::Hash;

use anyhow::anyhow;
use bech32::{Bech32m, Hrp};
use borsh::{BorshDeserialize, BorshSerialize};
use k256::ecdsa::{signature::DigestVerifier as _, RecoveryId, Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha3::{Digest as _, Keccak256};
use sov_modules_api::{
    capabilities::{AuthenticationError, AuthenticationResult, AuthorizationData, FatalError},
    macros::config_value,
    transaction::{
        AuthenticatedTransactionAndRawHash,
        AuthenticatedTransactionData,
        Credentials,
        TransactionVerificationError,
        TxDetails,
        UnsignedTransaction,
    },
    CredentialId,
    CryptoSpec,
    DispatchCall,
    GasMeter,
    MeteredHasher,
    PreExecWorkingSet,
    PublicKey,
    Spec,
};
use sov_rollup_interface::{crypto::SigVerificationError, TxHash};

/// The chain id of the rollup.
pub const CHAIN_ID: u64 = config_value!("CHAIN_ID");

#[derive(Clone, Debug, Eq, PartialEq, BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
struct PubKey {
    vk: Vec<u8>,
}

impl Hash for PubKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.vk.hash(state);
    }
}

impl sov_rollup_interface::crypto::PublicKey for PubKey {
    fn credential_id<Hasher: sha3::Digest<OutputSize = sha3::digest::consts::U32>>(
        &self,
    ) -> CredentialId {
        let mut hasher = Hasher::new();
        hasher.update(self.vk.clone());

        CredentialId(hasher.finalize().into())
    }
}

#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct Tx<S: Spec> {
    pub signature: Vec<u8>,
    pub verifying_key: Vec<u8>,
    /// The runtime message of the transaction. The message should have been encoded using the
    /// [`crate::EncodeCall`] trait.
    pub runtime_msg: Vec<u8>,
    /// The nonce of the transaction.
    pub nonce: u64,
    /// The transaction metadata. Contains gas parameters and the chain ID.
    pub details: TxDetails<S>,
}

impl<S: Spec> Tx<S> {
    pub fn verify(
        &self,
    ) -> Result<(VerifyingKey, Signature), TransactionVerificationError<S::Gas>> {
        let serialized_tx = borsh::to_vec(&self.to_unsigned_transaction()).map_err(|e| {
            TransactionVerificationError::TransactionDeserializationError(e.to_string())
        })?;
        let digest = Keccak256::new_with_prefix(prefix_msg(serialized_tx));
        let signature = Signature::from_slice(&self.signature).map_err(|e| {
            TransactionVerificationError::BadSignature(SigVerificationError::BadSignature(
                e.to_string(),
            ))
        })?;
        let vk = VerifyingKey::recover_from_digest(
            digest.clone(),
            &signature,
            RecoveryId::from_byte(0).expect("construction of recovery id should not fail"),
        )
        .map_err(|e| {
            TransactionVerificationError::TransactionDeserializationError(e.to_string())
        })?;
        vk.verify_digest(digest, &signature).map_err(|e| {
            TransactionVerificationError::BadSignature(SigVerificationError::BadSignature(
                e.to_string(),
            ))
        })?;

        Ok((vk, signature))
    }

    pub fn to_unsigned_transaction(&self) -> UnsignedTransaction<S> {
        UnsignedTransaction::new_with_details(
            self.runtime_msg.clone(),
            self.nonce,
            self.details.clone(),
        )
    }
}

pub fn authenticate<S: Spec, D: DispatchCall<Spec = S>, Meter: GasMeter<S::Gas>>(
    mut raw_tx: &[u8],
    state: &mut PreExecWorkingSet<S, Meter>,
) -> AuthenticationResult<S, D::Decodable, AuthorizationData<S>> {
    let raw_tx_hash = MeteredHasher::<
        S::Gas,
        PreExecWorkingSet<S, Meter>,
        <S::CryptoSpec as CryptoSpec>::Hasher,
    >::digest(raw_tx, state)
    .map(TxHash::new)
    .map_err(|e| AuthenticationError::Invalid(e.to_string()))?;

    let tx = <Tx<S> as BorshDeserialize>::deserialize(&mut raw_tx).map_err(|e| {
        AuthenticationError::FatalError(FatalError::DeserializationFailed(e.to_string()))
    })?;

    if tx.details.chain_id != CHAIN_ID {
        return Err(AuthenticationError::FatalError(
            FatalError::InvalidChainId {
                expected: CHAIN_ID,
                got: tx.details.chain_id,
            },
        ));
    }

    let (vk, _) = tx.verify().map_err(|e| match e {
        TransactionVerificationError::BadSignature(_)
        | TransactionVerificationError::TransactionDeserializationError(_) => {
            AuthenticationError::FatalError(FatalError::SigVerificationFailed(e.to_string()))
        },
        TransactionVerificationError::GasError(_) => AuthenticationError::Invalid(e.to_string()),
    })?;

    let runtime_call = D::decode_call(&tx.runtime_msg, state).map_err(|e| {
        AuthenticationError::FatalError(FatalError::MessageDecodingFailed(
            e.to_string(),
            raw_tx_hash,
        ))
    })?;

    let eth_pk = PubKey {
        vk: tx.verifying_key.clone(),
    };
    let credential_id = eth_pk.credential_id::<<S::CryptoSpec as CryptoSpec>::Hasher>();
    let credentials = Credentials::new(credential_id);
    let address = vk_to_address::<S>(&vk)
        .map_err(|e| AuthenticationError::FatalError(FatalError::Other(e.to_string())))?;

    Ok((
        AuthenticatedTransactionAndRawHash {
            raw_tx_hash,
            authenticated_tx: AuthenticatedTransactionData {
                chain_id: tx.details.chain_id,
                gas_limit: tx.details.gas_limit,
                max_fee: tx.details.max_fee,
                max_priority_fee_bips: tx.details.max_priority_fee_bips,
            },
        },
        AuthorizationData {
            nonce: tx.nonce,
            credential_id,
            credentials,
            default_address: Some(address),
        },
        runtime_call,
    ))
}

pub fn prefix_msg(msg: Vec<u8>) -> Vec<u8> {
    let prefix = format!("\x19Ethereum Signed Message:\n{}", msg.len());
    [prefix.as_bytes(), &msg].concat()
}

pub fn vk_to_address<S: Spec>(vk: &VerifyingKey) -> anyhow::Result<S::Address> {
    let ep = vk.to_encoded_point(false);
    let pk_bytes = &ep.as_bytes()[1..];

    let mut hasher = Keccak256::new();
    hasher.update(pk_bytes);
    let hash = hasher.finalize();

    let eth_address = &hash[12..];
    let hrp = Hrp::parse("sov")?;
    let bech32_address = bech32::encode::<Bech32m>(hrp, eth_address)?;
    let address = bech32_address
        .parse::<<S as Spec>::Address>()
        .map_err(|_| anyhow!("bech32 parsing failed"))?;

    Ok(address)
}
