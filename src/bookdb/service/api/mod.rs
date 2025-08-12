// src/commands/mod.rs - Command module exports with inc/dec added

pub mod getv;
pub mod setv;
pub mod delv;
pub mod inc;      // New: Increment command
pub mod dec;      // New: Decrement command
pub mod getd;
pub mod setd;
pub mod ls;
pub mod import;
pub mod export;
pub mod r#use;   // 'use' is a keyword, so we use raw identifier

// Re-export command execution functions for convenience
pub use getv::execute as execute_getv;
pub use setv::execute as execute_setv;
pub use delv::execute as execute_delv;
pub use inc::execute as execute_inc;    // New export
pub use dec::execute as execute_dec;    // New export
pub use getd::execute as execute_getd;
pub use setd::execute as execute_setd;
pub use ls::execute as execute_ls;
pub use import::execute as execute_import;
pub use export::execute as execute_export;
pub use r#use::execute as execute_use;
