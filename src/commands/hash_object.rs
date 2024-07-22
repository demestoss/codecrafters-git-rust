use crate::objects::Object;
use anyhow::Context;
use std::io;
use std::path::Path;

pub fn handle(file_name: &Path, write: bool) -> anyhow::Result<()> {
    let obj = Object::blob_from_file(file_name)
        .with_context(|| format!("read project file {}", file_name.display()))?;

    let hash = if write {
        obj.write_to_objects().context("write to .git/objects")?
    } else {
        obj.write(io::sink()).context("write to io::sink")?
    };

    let hash = hex::encode(hash);
    println!("{hash}");

    Ok(())
}
