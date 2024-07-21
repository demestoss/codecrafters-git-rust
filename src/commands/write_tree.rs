use crate::commands::hash_object::{generate_blob_object, write_git_object};
use crate::objects::ObjectWriter;
use anyhow::anyhow;
use std::fs::Metadata;
use std::io::prelude::*;
use std::io::Cursor;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::{fs, io};

pub fn handle() -> anyhow::Result<()> {
    let root = Path::new("./");
    let hash = write_tree(&root)?;
    println!("{hash}");
    Ok(())
}

fn write_tree(file_path: &Path) -> anyhow::Result<String> {
    let mut buf = Vec::new();
    generate_tree_object(&file_path, &mut buf)?;
    let size = buf.len();

    let mut object_writer = ObjectWriter::new(&mut buf);
    write!(object_writer, "tree {size}\0")?;
    io::copy(&mut Cursor::new(buf), &mut object_writer)?;

    let hash = object_writer.finalize()?;
    write_git_object(&hash, &buf)?;

    Ok(hash)
}

fn generate_tree_object(file_path: &&Path, mut buf: impl Write) -> anyhow::Result<()> {
    let mut dir = fs::read_dir(file_path)?;
    while let Some(res) = dir.next() {
        let res = res?;
        let file_name = res.file_name();
        let name = file_name
            .to_str()
            .ok_or(anyhow!("error: file path is broken"))?;
        let meta = res.metadata()?;
        let path = res.path();

        if is_path_ignored(name) {
            continue;
        }

        let hash = if meta.is_dir() {
            write_tree(&path)?
        } else {
            write_blob(&path)?
        };
        let mode = get_mode(meta);

        write!(buf, "{mode} {name}\0")?;
        let hash = hex::decode(hash)?;
        buf.write(&hash)?;
    }
    Ok(())
}

fn write_blob(file_path: &Path) -> anyhow::Result<String> {
    let mut buf = Vec::new();
    let hash = generate_blob_object(file_path, &mut buf)?;
    write_git_object(&hash, &buf)?;
    Ok(hash)
}

fn get_mode(meta: Metadata) -> String {
    if meta.is_dir() {
        "40000".to_string()
    } else if meta.is_symlink() {
        "120000".to_string()
    } else if (meta.permissions().mode() & 0o111) != 0 {
        "100755".to_string()
    } else {
        "100644".to_string()
    }
}

const IGNORED_PATHS: &[&str; 3] = &[".git", "target", "debug"];

fn is_path_ignored(name: &str) -> bool {
    IGNORED_PATHS.contains(&name)
}
