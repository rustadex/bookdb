
// must load first due to build.rs
pub mod utils;


// --------------------------------------------------------------------
// Phase 1.7 upgrade
// Feature-conditional re-exports for easy access
#[cfg(feature = "context-chain-v1")]
pub use utils::validator::valV1 as context_validator;

#[cfg(feature = "context-chain-v3")]
pub use utils::validator::valV3 as context_validator;


// Version-specific convenience re-exports
#[cfg(feature = "context-chain-v1")]
pub mod context {
    pub use crate::service::ctx::typesV1::*;
}

#[cfg(feature = "context-chain-v3")]
pub mod context {
    pub use crate::service::ctx::typesV3::*;
    pub use crate::service::ctx::types::segments::*;
}



// Always available
pub use service::ctx::ContextManager;
// --------------------------------------------------------------------

// pub mod app;
// pub mod service;



pub mod info {
  use crate::utils::info::*;
}

// pub use app::sup::error; 
// pub use app::ctrl::{ dispatch as run };




// pub use app::install::{ 
//   InstallGuard, 
//   InstallationManager, 
//   require_installation_or_install 
// }; 
