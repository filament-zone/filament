// TODO(xla): Document.
pub mod core {
    // TODO(xla): Document.
    pub mod chain {
        pub mod v1alpha1 {
            include!("gen/pulzaar.core.chain.v1alpha1.rs");
        }
    }

    // TODO(xla): Document.
    pub mod stake {
        pub mod v1alpha1 {
            include!("gen/pulzaar.core.stake.v1alpha1.rs");
        }
    }
}
