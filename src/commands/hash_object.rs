use crate::objects;
use crate::objects::ObjectWriter;
use std::io::prelude::*;
use std::path::Path;
use std::{fs, io};

pub fn handle(file_name: &Path, write: bool) -> anyhow::Result<()> {
    if write {
        let mut buf = Vec::new();
        let hash = generate_blob_object(file_name, &mut buf)?;
        println!("{hash}");
        write_git_object(&hash, &buf)?;
    } else {
        let hash = generate_blob_object(file_name, io::sink())?;
        println!("{hash}");
    }
    Ok(())
}

pub fn generate_blob_object(file_name: &Path, writer: impl Write) -> anyhow::Result<String> {
    let file_stat = fs::metadata(file_name)?;
    let size = file_stat.len();

    let mut object_writer = ObjectWriter::new(writer);

    write!(object_writer, "blob {size}\0")?;

    let mut file = fs::File::open(&file_name)?;
    io::copy(&mut file, &mut object_writer)?;

    let hash = object_writer.finalize()?;

    Ok(hash)
}

pub fn write_git_object(hash: &str, object_content: &[u8]) -> anyhow::Result<()> {
    fs::create_dir_all(objects::get_object_dir_path(hash))?;
    fs::write(objects::get_object_path(hash), object_content)?;
    Ok(())
}
