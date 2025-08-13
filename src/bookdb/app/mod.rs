
// app sub package
pub mod sup;
pub mod admin;
pub mod ctrl;

//Support Functions
#[path = "sup/config.rs"]
pub mod config;

#[path = "sup/error.rs"]
pub mod error;



#[path = "admin/install.rs"]
pub mod install;


// cli controller

#[path = "ctrl/cli.rs"]
pub mod cli;

#[path = "ctrl/handlers.rs"]
pub mod handlers;

#[path = "ctrl/dispatch.rs"]
pub mod dispatch;
