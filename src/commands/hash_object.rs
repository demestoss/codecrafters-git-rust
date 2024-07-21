use crate::objects;
use sha1::{Digest, Sha1};
use std::fs;
use std::path::Path;

pub fn handle(file_name: &Path, write: bool) -> anyhow::Result<()> {
    let file_content = fs::read_to_string(file_name)?;
    let blob = get_blob_content(&file_content);
    let hash = get_object_hash(&blob);
    println!("{hash}");

    if write {
        write_git_object(&hash, &blob)?;
    }

    Ok(())
}

fn get_blob_content(file_content: &str) -> String {
    let size = file_content.len();
    format!("blob {size}\0{}", file_content)
}

fn get_object_hash(object: &str) -> String {
    let hasher = Sha1::digest(object);
    hex::encode(hasher)
}

fn write_git_object(hash: &str, object: &str) -> anyhow::Result<()> {
    let compressed = objects::compress(object)?;
    fs::create_dir_all(objects::get_object_dir_path(hash))?;
    fs::write(objects::get_object_path(hash), compressed)?;
    Ok(())
}
