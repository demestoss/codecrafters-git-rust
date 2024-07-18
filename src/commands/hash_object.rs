use crate::objects;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::Write;

pub fn handle(args: &[String]) -> anyhow::Result<()> {
    match args {
        [] => println!("usage: git hash-object [-w] [--stdin] <file_name>"),
        args => {
            let file_name = args.last().expect("args are not empty");

            let file_content = fs::read_to_string(file_name)?;
            let blob = get_blob_content(&file_content);
            let hash = get_object_hash(&blob);
            println!("{hash}");

            if args.contains(&"-w".to_string()) {
                write_git_object(&hash, &blob)?;
            }
        }
    };
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
    let compressed = objects::get_compressed(object)?;
    fs::create_dir_all(objects::get_dir_path(hash))?;
    fs::write(objects::get_path(hash), compressed)?;
    Ok(())
}
