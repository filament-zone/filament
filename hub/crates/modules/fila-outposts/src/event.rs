/// Represents different types of events for OutpostRegistry operations.
#[derive(
    Clone,
    Debug,
    PartialEq,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Event {
    /// Event emitted when a new Outpost is registered.
    ///
    /// Fields:
    /// - `chain_id`: The chain identifier of the newly registered Outpost.
    Register {
        /// The unique identifier for the chain.
        chain_id: String,
    },
}
