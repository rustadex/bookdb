use std::env;
use colored::Colorize;

const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

fn logo(){
    const LOGO_TEXT: &str = include_str!("../../docs/logo.log");
    println!("{}", LOGO_TEXT.cyan());
}

fn version(){
    if env::var("QUIET_MODE").is_ok() {
        return;
    }

    logo;
    // Dynamically create the version string
    let version_string = format!(
        "          CLI   v{:<8}",  CLI_VERSIO
    );        

    println!("{}", "-----------------------------------------------------------------------------------------------".dimmed());

    // --- NEW: ADD THE LICENSE BLURB ---
    let license_blurb = "
    This software is provided 'as is', without warranty of any kind.
    Distributed under the MIT OR Apache-2.0 license. Use at your own risk.
    ";

    println!("{}", version_string);
    println!("{}", license_blurb.dimmed());

    println!("{}", "-----------------------------------------------------------------------------------------------".dimmed());

}
