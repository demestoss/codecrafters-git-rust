use anyhow::anyhow;
use flate2::read::ZlibDecoder;
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
        "-p" => get_object_data(object_hash, get_object_content)?,
        "-e" => get_object_data(object_hash, get_object_exists)?,
        "-t" => get_object_data(object_hash, get_object_type)?,
        "-s" => get_object_data(object_hash, get_object_size)?,
        _ => println!("usage: git cat-file (-p | -e | -t | -s) <object>"),
    };
    Ok(())
}

fn get_object_data(
    object_hash: &str,
    getter_fn: fn(object: &str) -> anyhow::Result<String>,
) -> anyhow::Result<()> {
    let object_content = get_object_by_hash(object_hash)?;
    let object = get_decompressed_object(&object_content)?;
    let info = getter_fn(&object)?;

    print!("{info}");
    io::stdout().flush()?;

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

fn get_object_path(hash: &str) -> String {
    ".git/objects/".to_string() + &hash[..2] + "/" + &hash[2..]
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
    Ok(object.split(' ').take(1).collect::<String>())
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
