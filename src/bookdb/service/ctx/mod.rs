


mod context;
mod context_manager;
mod cursor;

pub use context::{
    ContextManager, 
    ContextChain, 
    ResolvedContext,  
    ResolvedContextIds,       
    CursorState, 
    Anchor,
    ChainMode,
    DefaultResolver,
    parse_context_chain
};


// --------------------------------------------------------------------
// Phase 1.7 upgrade


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

// Feature-conditional exports
#[cfg(feature = "context-chain-v1")]
pub use typesV1::{ContextChain, ResolvedContext, Anchor, ChainMode};

#[cfg(feature = "context-chain-v3")]
pub use typesV3::{ContextChainV3 as ContextChain, VarContextChain, DocContextChain, Anchor, ChainMode};

// --------------------------------------------------------------------
