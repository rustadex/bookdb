// Load order
pub mod utils;
// pub mod oxidize;
// pub mod service;
// pub mod app;

// // ---------------- Feature-conditional re-exports ----------------

// // Validator alias remains in utils::validator::{valV1,valV3}
// #[cfg(feature = "context-chain-v1")]
// pub use utils::validator::valV1 as context_validator;

// #[cfg(feature = "context-chain-v3")]
// pub use utils::validator::valV3 as context_validator;

// // Context type surfaces (point at actual module paths)
// #[cfg(feature = "context-chain-v1")]
// pub mod context {
//     pub use crate::service::ctx::types::typesV1::*;
// }

// #[cfg(feature = "context-chain-v3")]
// pub mod context {
//     pub use crate::service::ctx::types::typesV3::*;
//     pub use crate::service::ctx::types::segment::*; // NOTE: singular 'segment.rs'
// }

// // Always available
// pub use crate::service::ctx::context_manager::ContextManager;

// // ---------------- Convenience re-exports ----------------

// pub mod info {
//     pub use crate::utils::info::*; // re-export helpers
// }

// // Error/result
// pub use crate::app::sup::error::{BookdbError, Result};

// // CLI entry
// pub use crate::app::ctrl::dispatch;
// pub use crate::app::ctrl::dispatch::run;

// // Install guards/managers (file lives at app/admin/install.rs)
// pub use crate::app::admin::install::{
//     InstallGuard,
//     InstallationManager,
//     require_installation_or_install
// };
