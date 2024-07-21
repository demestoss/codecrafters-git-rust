use crate::objects;
use anyhow::anyhow;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::fs;
use std::io::{Read, Write};

pub fn get_object_dir_path(hash: &str) -> String {
    ".git/objects/".to_string() + &hash[..2]
}

pub fn get_object_path(hash: &str) -> String {
    get_object_dir_path(hash) + "/" + &hash[2..]
}

pub fn get_object_by_hash(hash: &str) -> anyhow::Result<Vec<u8>> {
    if hash.len() != 40 {
        return Err(anyhow!("incorrect object hash"));
    }

    let path = objects::get_object_path(hash);
    let object_content = fs::read(path)?;
    Ok(object_content)
}

pub fn decompress(object_content: &[u8]) -> anyhow::Result<String> {
    let mut d = ZlibDecoder::new(object_content);
    let mut str = String::new();
    d.read_to_string(&mut str)?;
    Ok(str)
}

pub fn compress(object: &str) -> anyhow::Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(object.as_bytes())?;
    Ok(encoder.finish()?)
}
