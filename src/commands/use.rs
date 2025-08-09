use crate::error::Result;

pub fn execute(context_str: &str) -> Result<()> {
    println!("{}", context_str);
    Ok(())
}
