

pub mod macros;
pub mod info;
pub mod extra;

// --------------------------------------------------------------------
// Phase 1.7 upgrade

// Group validator modules under one cfg block
#[cfg(feature = "context-chain-v1")]
pub mod validator {
    pub mod valV1;
    pub use valV1 as active_validator;
}

#[cfg(feature = "context-chain-v3")]
pub mod validator {
    pub mod valV3;
    pub use valV3 as active_validator;
}

// For development/testing with both versions
#[cfg(feature = "dev-both-versions")]
pub mod validator {
    pub mod valV1;
    pub mod valV3;
    // Default to V3 when both available
    pub use valV3 as active_validator;
    pub use valV1 as legacy_validator;
}

// --------------------------------------------------------------------
