#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

pub mod authentication;
#[cfg(feature = "native")]
pub mod genesis;
mod hooks;
pub mod runtime;

use sov_modules_stf_blueprint::StfBlueprint;
use sov_rollup_interface::da::DaVerifier;
use sov_stf_runner::verifier::StateTransitionVerifier;

/// Alias for StateTransitionVerifier.
pub type StfVerifier<DA, ZkSpec, RT, K, InnerVm, OuterVm> = StateTransitionVerifier<
    StfBlueprint<ZkSpec, <DA as DaVerifier>::Spec, RT, K>,
    DA,
    InnerVm,
    OuterVm,
>;
