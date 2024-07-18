use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{Read, Write};

pub fn get_dir_path(hash: &str) -> String {
    ".git/objects/".to_string() + &hash[..2]
}

pub fn get_path(hash: &str) -> String {
    get_dir_path(hash) + "/" + &hash[2..]
}

pub fn get_decompressed(object_content: &[u8]) -> anyhow::Result<String> {
    let mut d = ZlibDecoder::new(object_content);
    let mut str = String::new();
    d.read_to_string(&mut str)?;
    Ok(str)
}

pub fn get_compressed(object: &str) -> anyhow::Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(object.as_bytes())?;
    Ok(encoder.finish()?)
}
