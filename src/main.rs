use anyhow::anyhow;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::env;
use std::fs;
use std::io;
use std::io::{Read, Write};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    process(&args[1..])
}

fn process(args: &[String]) -> anyhow::Result<()> {
    match args[0].as_str() {
        "init" => handle_init_command()?,
        "cat-file" => handle_cat_file_command(&args[1..]),
        "hash-object" => handle_hash_object_command(&args[1..])?,
        _ => println!("unknown command: {}", args[1]),
    };
    Ok(())
}

fn handle_init_command() -> Result<(), io::Error> {
    fs::create_dir(".git")?;
    fs::create_dir(".git/objects")?;
    fs::create_dir(".git/refs")?;
    fs::write(".git/HEAD", "ref: refs/heads/master\n")?;
    println!("Initialized git directory");
    Ok(())
}

fn handle_cat_file_command(args: &[String]) {
    match args {
        [flag, object] => match handle_object_info(flag, object) {
            Err(e) => eprintln!("incorrect object: {e}"),
            _ => {}
        },
        _ => println!("usage: git cat-file (-p | -e | -t | -s) <object>"),
    }
}

fn handle_object_info(flag: &str, object_hash: &str) -> anyhow::Result<()> {
    match flag {
        "-p" => display_object_info(object_hash, get_object_content)?,
        "-e" => display_object_info(object_hash, get_object_exists)?,
        "-t" => display_object_info(object_hash, get_object_type)?,
        "-s" => display_object_info(object_hash, get_object_size)?,
        _ => println!("usage: git cat-file (-p | -e | -t | -s) <object>"),
    };
    Ok(())
}

fn display_object_info(
    object_hash: &str,
    getter_fn: fn(object: &str) -> anyhow::Result<String>,
) -> anyhow::Result<()> {
    let object_content = get_object_by_hash(object_hash)?;
    let object = get_decompressed_object(&object_content)?;
    let info = getter_fn(&object)?;
    print!("{info}");
    Ok(())
}

fn get_object_by_hash(hash: &str) -> anyhow::Result<Vec<u8>> {
    if hash.len() != 40 {
        return Err(anyhow!("incorrect object hash"));
    }

    let path = get_object_path(hash);
    let object_content = fs::read(path)?;
    Ok(object_content)
}

fn get_object_dir_path(hash: &str) -> String {
    ".git/objects/".to_string() + &hash[..2]
}

fn get_object_path(hash: &str) -> String {
    get_object_dir_path(hash) + "/" + &hash[2..]
}

fn get_decompressed_object(object_content: &[u8]) -> anyhow::Result<String> {
    let mut d = ZlibDecoder::new(object_content);
    let mut str = String::new();
    d.read_to_string(&mut str)?;
    Ok(str)
}

fn get_object_exists(_: &str) -> anyhow::Result<String> {
    Ok("object exists!".to_owned())
}

fn get_object_type(object: &str) -> anyhow::Result<String> {
    Ok(object.split(' ').take(1).collect())
}

fn get_object_size(object: &str) -> anyhow::Result<String> {
    Ok(object
        .chars()
        .skip_while(|&v| v != ' ')
        .skip(1)
        .take_while(|&v| v != char::from(0x00))
        .collect())
}

fn get_object_content(object: &str) -> anyhow::Result<String> {
    Ok(object
        .chars()
        .skip_while(|&v| v != char::from(0x00))
        .skip(1)
        .collect())
}

fn handle_hash_object_command(args: &[String]) -> anyhow::Result<()> {
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

fn get_compressed_object(object: &str) -> anyhow::Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(object.as_bytes())?;
    Ok(encoder.finish()?)
}

fn write_git_object(hash: &str, object: &str) -> anyhow::Result<()> {
    let compressed = get_compressed_object(object)?;
    fs::create_dir_all(get_object_dir_path(hash))?;
    fs::write(get_object_path(hash), compressed)?;
    Ok(())
}
