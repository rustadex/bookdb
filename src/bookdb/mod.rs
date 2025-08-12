
// must load first due to build.rs
pub mod app;
pub mod service;
pub mod utils;

pub use app::admin::install{ 
  InstallGuard, 
  InstallationManager, 
  require_installation_or_install 
}; 

pub use app::sup::error; 
pub use app::ctrl::{ dispatch as run };

pub mod info {
  use crate::utils::info::*;
}
