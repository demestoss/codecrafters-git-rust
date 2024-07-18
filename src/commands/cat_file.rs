use crate::objects;
use anyhow::anyhow;
use std::fs;

pub fn handle(args: &[String]) {
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
    let object = objects::get_decompressed(&object_content)?;
    let info = getter_fn(&object)?;
    print!("{info}");
    Ok(())
}

fn get_object_by_hash(hash: &str) -> anyhow::Result<Vec<u8>> {
    if hash.len() != 40 {
        return Err(anyhow!("incorrect object hash"));
    }

    let path = objects::get_path(hash);
    let object_content = fs::read(path)?;
    Ok(object_content)
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
