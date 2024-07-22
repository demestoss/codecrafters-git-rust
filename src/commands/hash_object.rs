use std::io;
use std::path::Path;

use crate::objects::Object;

pub fn handle(file_name: &Path, write: bool) -> anyhow::Result<()> {
    let obj = Object::blob_from_file(file_name)?;

    let hash = if write {
        obj.write_to_objects()?
    } else {
        obj.write(io::sink())?
    };

    let hash = hex::encode(hash);
    println!("{hash}");

    Ok(())
}
