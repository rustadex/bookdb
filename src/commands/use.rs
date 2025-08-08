// src/commands/use.rs
use crate::context;
use crate::error::Result;

pub fn execute(context_str: &str) -> Result<()> {
    // For this clean build, just echo the parsed context; persistence can be added later.
    let ctx = context::parse_chain(context_str)?;
    println!("{:#?}", ctx);
    Ok(())
}
