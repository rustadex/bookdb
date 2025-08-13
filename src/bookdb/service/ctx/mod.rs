// This module provides all functionality related to context resolution.
// It exposes a consistent API regardless of the active feature flag (v1 or v3).

// 1. Declare all the implementation modules.
pub mod context;
pub mod context_manager;
pub mod cursor;
pub mod resolver;
pub mod types; // The submodule for data models

// 2. Expose the concrete implementation functions and structs that are always needed.
pub use context_manager::ContextManager;



// 3. --- The Magic: Use feature flags to alias the ACTIVE version ---
// The rest of the application will just `use bookdb::service::ctx::ContextChain`
// and will get the correct version based on the compiled features.

#[cfg(feature = "context-chain-v1")]
pub use types::typesV1::{
    ContextChain,
    ResolvedContext,
    Anchor,
    ChainMode,
    DefaultResolver,
    CursorState
};


// pub use typesV1::DefaultResolver;

#[cfg(feature = "context-chain-v1")]
pub use context::parse_context_chain; // Expose the V1 parser

#[cfg(feature = "context-chain-v3")]
pub use types::typesV3::{
    ContextChainV3 as ContextChain, // ALIAS V3 struct to the common name
    VarContextChain,
    DocContextChain,
    // does v3 need a defautl resolver DefaultResolver
    //CursorState
};

// V3 would have a different parsing entry point, which we expose here.
// You'll need to create this function in your v3_validator.
// #[cfg(feature = "context-chain-v3")]
// pub use crate::bookdb::utils::validator::valV3::validate_and_create as parse_context_chain;


// 4. (Optional but recommended) For development, allow access to both versions.
#[cfg(feature = "dev-both-versions")]
pub mod v1 {
    pub use super::types::typesV1::*;
    pub use super::context::parse_context_chain;
}

#[cfg(feature = "dev-both-versions")]
pub mod v3 {
    pub use super::types::typesV3::*;
    pub use super::types::segment::*;
    // pub use crate::bookdb::utils::validator::valV3::validate_and_create;
}
