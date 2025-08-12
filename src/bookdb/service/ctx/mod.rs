


mod context;
mod context_manager;
mod cursor;
mod resolver; 



// --------------------------------------------------------------------
// Phase 1.7 upgrade

// SELECTIVE Re-export of active types based on feature flags (NOT everything with *)
#[cfg(any(feature = "context-chain-v1", not(feature = "context-chain-v3")))]
pub use typesV1::{
    ContextChain,
    ResolvedContext,
    CursorState,
    Anchor,
    ChainMode,
    DefaultResolver,
    ResolvedContextIds,
};



// Version-specific type modules
#[cfg(feature = "context-chain-v1")]
pub mod typesV1;

#[cfg(feature = "context-chain-v3")]
pub mod typesV3;


pub mod types {
    pub mod segments;  // Always available for V3 features
    
    // Re-export active types based on feature
    #[cfg(feature = "context-chain-v1")]
    pub use super::typesV1::*;
    
    #[cfg(feature = "context-chain-v3")]
    pub use super::typesV3::*;

}

//todo: need to clean up exports from top level files not using types yet
// Always export common items
pub use context_manager::ContextManager;
pub use cursor::CursorState;

pub use resolver::*;

// Feature-conditional exports
#[cfg(feature = "context-chain-v1")]
//pub use typesV1::{ContextChain, ResolvedContext, Anchor, ChainMode};
pub use typesV1::*;

#[cfg(feature = "context-chain-v3")]
pub use typesV3::{ContextChainV3 as ContextChain, VarContextChain, DocContextChain, Anchor, ChainMode};




// Development exports (both available when needed)
#[cfg(feature = "dev-both-versions")]
pub mod v1 {
    pub use super::typesV1::*;
}

#[cfg(feature = "dev-both-versions")]
pub mod v3 {
    pub use super::typesV3::*;
    pub use super::types::segments::*;
}










// --------------------------------------------------------------------
