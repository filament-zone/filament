/// A Pulzaar transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Transaction {
    #[prost(message, optional, tag="1")]
    pub body: ::core::option::Option<Body>,
}
/// Body of a Pulzaar transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Body {
    /// List of inputs carried by the transaction.
    #[prost(message, repeated, tag="1")]
    pub input: ::prost::alloc::vec::Vec<Input>,
    /// Intended chain for the transaction to land on, to be included to prevent replays on other
    /// chains.
    #[prost(string, tag="2")]
    pub chain_id: ::prost::alloc::string::String,
    /// Maximum height until the transaction is valid, doesn't expire if the value is zero.
    #[prost(uint64, tag="3")]
    pub max_height: u64,
    /// Fees of the transaction.
    #[prost(message, optional, tag="4")]
    pub fee: ::core::option::Option<Fee>,
}
/// An input to the application state machine.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Input {
    #[prost(oneof="input::Input", tags="1, 2")]
    pub input: ::core::option::Option<input::Input>,
}
/// Nested message and enum types in `Input`.
pub mod input {
    #[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Input {
        /// Staking
        #[prost(message, tag="1")]
        Delegate(super::super::super::stake::v1alpha1::Delegate),
        #[prost(message, tag="2")]
        Undelegate(super::super::super::stake::v1alpha1::Undelegate),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fee {
    /// The token amount to be paid as fees.
    #[prost(message, optional, tag="1")]
    pub amount: ::core::option::Option<Amount>,
    /// The asset ID of the token to be paid as fees.
    #[prost(message, optional, tag="2")]
    pub asset_id: ::core::option::Option<AssetId>,
}
/// TODO(xla): Document.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Amount {
    #[prost(uint64, tag="1")]
    pub lo: u64,
    #[prost(uint64, tag="2")]
    pub hi: u64,
}
/// TODO(xla): Document.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Denom {
    #[prost(string, tag="1")]
    pub denom: ::prost::alloc::string::String,
}
/// TODO(xla): Document.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AssetId {
    #[prost(bytes="vec", tag="1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
/// TODO(xla): Document.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Asset {
    #[prost(message, optional, tag="1")]
    pub id: ::core::option::Option<AssetId>,
    #[prost(message, optional, tag="2")]
    pub denom: ::core::option::Option<Denom>,
}
