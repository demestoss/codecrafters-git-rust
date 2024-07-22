use anyhow::Context;
use std::fs;

pub fn handle() -> anyhow::Result<()> {
    fs::create_dir(".git").context("create .git dir")?;
    fs::create_dir(".git/objects").context("create .git/objects dir")?;
    fs::create_dir(".git/refs").context("create .git/refs dir")?;
    fs::write(".git/HEAD", "ref: refs/heads/master\n").context("write HEAD file")?;
    println!("Initialized git directory");
    Ok(())
}
