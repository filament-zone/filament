#[cfg(feature = "native")]
use sov_modules_api::default_signature::private_key::DefaultPrivateKey;
use sov_modules_api::default_signature::{DefaultPublicKey, DefaultSignature};
use sov_modules_core::{Address, Spec, TupleGasUnit};
#[cfg(feature = "native")]
use sov_state::ProverStorage;
use sov_state::{ArrayWitness, DefaultStorageSpec, ZkStorage};

#[cfg(feature = "native")]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Context {
    pub sender: Address,
    pub sequencer: Address,
    visible_height: u64,
}

#[cfg(feature = "native")]
impl sov_modules_core::Context for Context {
    type GasUnit = TupleGasUnit<2>;

    fn new(sender: Self::Address, sequencer: Self::Address, height: u64) -> Self {
        Self {
            sender,
            sequencer,
            visible_height: height,
        }
    }

    fn sender(&self) -> &Self::Address {
        &self.sender
    }

    fn sequencer(&self) -> &Self::Address {
        &self.sequencer
    }

    fn slot_height(&self) -> u64 {
        self.visible_height
    }
}

#[cfg(feature = "native")]
impl Spec for Context {
    type Address = Address;
    type Hasher = sha2::Sha256;
    type PrivateKey = DefaultPrivateKey;
    type PublicKey = DefaultPublicKey;
    type Signature = DefaultSignature;
    type Storage = ProverStorage<DefaultStorageSpec, sov_prover_storage_manager::SnapshotManager>;
    type Witness = ArrayWitness;
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ZkContext {
    pub sender: Address,
    pub sequencer: Address,
    visible_height: u64,
}

impl sov_modules_core::Context for ZkContext {
    type GasUnit = TupleGasUnit<2>;

    fn new(sender: Self::Address, sequencer: Self::Address, height: u64) -> Self {
        Self {
            sender,
            sequencer,
            visible_height: height,
        }
    }

    fn sender(&self) -> &Self::Address {
        &self.sender
    }

    fn sequencer(&self) -> &Self::Address {
        &self.sequencer
    }

    fn slot_height(&self) -> u64 {
        self.visible_height
    }
}

impl Spec for ZkContext {
    type Address = Address;
    type Hasher = sha2::Sha256;
    #[cfg(feature = "native")]
    type PrivateKey = DefaultPrivateKey;
    type PublicKey = DefaultPublicKey;
    type Signature = DefaultSignature;
    type Storage = ZkStorage<DefaultStorageSpec>;
    type Witness = ArrayWitness;
}
