use crate::objects::{Object, ObjectHash, ObjectKind};
use anyhow::{anyhow, Context};
use std::fs;
use std::fs::Metadata;
use std::io::prelude::*;
use std::io::Cursor;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub fn handle() -> anyhow::Result<()> {
    let root = Path::new("./");
    let hash = write_tree(&root)?;
    let hash = hex::encode(hash);
    println!("{hash}");
    Ok(())
}

fn write_tree(file_path: &Path) -> anyhow::Result<ObjectHash> {
    let mut buf = Vec::new();
    generate_tree_object(&file_path, &mut buf)?;

    let object = Object {
        kind: ObjectKind::Tree,
        size: buf.len() as u64,
        reader: Cursor::new(buf),
    };

    let hash = object
        .write_to_objects()
        .context("write to .git/objects dir")?;
    Ok(hash)
}

fn write_blob(file_path: &Path) -> anyhow::Result<ObjectHash> {
    let obj = Object::blob_from_file(file_path)
        .with_context(|| format!("parse blob from {}", file_path.display()))?;
    let hash = obj.write_to_objects()?;
    Ok(hash)
}

fn generate_tree_object(file_path: &Path, mut buf: impl Write) -> anyhow::Result<()> {
    let mut dir =
        fs::read_dir(&file_path).with_context(|| format!("read {}", file_path.display()))?;
    let mut entries = Vec::new();
    while let Some(res) = dir.next() {
        let res = res.context("incorrect dir entry")?;
        let file_name = res.file_name();
        let name = file_name
            .to_str()
            .ok_or(anyhow!("error: file path is broken"))?;
        let meta = res.metadata().context("get path entry metadata")?;
        let path = res.path();

        if is_path_ignored(name) {
            continue;
        }

        let mode = get_mode(meta);

        entries.push(TreeEntry {
            mode,
            name: name.to_owned(),
            path,
        })
    }

    entries.sort_by(|x, y| x.name.cmp(&y.name));

    for TreeEntry { mode, name, path } in entries {
        let is_dir = mode == "40000";

        let hash = if is_dir {
            let Ok(hash) = write_tree(&path) else {
                continue;
            };
            hash
        } else {
            write_blob(&path)?
        };

        write!(buf, "{mode} {name}\0")?;
        buf.write(&hash)?;
    }

    Ok(())
}

struct TreeEntry {
    name: String,
    mode: String,
    path: PathBuf,
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
