use crate::objects::{Object, ObjectHash, ObjectKind};
use anyhow::{bail, Context};
use std::cmp::Ordering;
use std::ffi::OsString;
use std::fs::Metadata;
use std::io::prelude::*;
use std::io::Cursor;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::{cmp, fs};

pub fn handle() -> anyhow::Result<()> {
    let root = Path::new("./");
    let Some(hash) = write_tree_for(&root)? else {
        bail!("do not write empty tree")
    };
    let hash = hex::encode(hash);
    println!("{hash}");
    Ok(())
}

pub fn write_tree_for(file_path: &Path) -> anyhow::Result<Option<ObjectHash>> {
    let buf = generate_tree_object(&file_path)?;

    if buf.is_empty() {
        Ok(None)
    } else {
        let object = Object {
            kind: ObjectKind::Tree,
            size: buf.len() as u64,
            reader: Cursor::new(buf),
        };
        Ok(Some(
            object
                .write_to_objects()
                .context("write to .git/objects dir")?,
        ))
    }
}

fn write_blob(file_path: &Path) -> anyhow::Result<ObjectHash> {
    let obj = Object::blob_from_file(file_path)
        .with_context(|| format!("parse blob from {}", file_path.display()))?;
    let hash = obj.write_to_objects()?;
    Ok(hash)
}

fn generate_tree_object(file_path: &Path) -> anyhow::Result<Vec<u8>> {
    let mut dir =
        fs::read_dir(&file_path).with_context(|| format!("read {}", file_path.display()))?;
    let mut entries = Vec::new();
    while let Some(res) = dir.next() {
        let res = res.context("incorrect dir entry")?;
        let file_name = res.file_name();
        let meta = res.metadata().context("get path entry metadata")?;
        let path = res.path();

        if is_path_ignored(&file_name) {
            continue;
        }
        entries.push((file_name, meta, path))
    }

    entries.sort_by(|a, b| {
        // https://github.com/git/git/blob/e09f1254c54329773904fe25d7c545a1fb4fa920/tree.c#L128
        let a_name = a.0.as_encoded_bytes();
        let b_name = b.0.as_encoded_bytes();
        let common_len = cmp::min(a_name.len(), b_name.len());

        match a_name[..common_len].cmp(&b_name[..common_len]) {
            Ordering::Equal => {}
            o => return o,
        }
        if a_name.len() == b_name.len() {
            return Ordering::Equal;
        }

        let c1 = if let Some(&c) = a_name.get(common_len) {
            Some(c)
        } else if a.1.is_dir() {
            Some(b'/')
        } else {
            None
        };
        let c2 = if let Some(&c) = b_name.get(common_len) {
            Some(c)
        } else if b.1.is_dir() {
            Some(b'/')
        } else {
            None
        };
        c1.cmp(&c2)
    });

    let mut buf = Vec::new();

    for (name, meta, path) in entries {
        let mode = get_mode(&meta);
        let is_dir = meta.is_dir();

        let hash = if is_dir {
            let Ok(hash) = write_tree_for(&path) else {
                continue;
            };
            hash
        } else {
            Some(write_blob(&path)?)
        };

        if let Some(hash) = hash {
            write!(buf, "{mode} ")?;
            buf.extend(name.as_encoded_bytes());
            write!(buf, "\0")?;
            buf.write(&hash)?;
        }
    }

    Ok(buf)
}

fn get_mode(meta: &Metadata) -> String {
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

const IGNORED_PATHS: &[&str; 4] = &[".git", "target", "debug", ".idea"];

fn is_path_ignored(name: &OsString) -> bool {
    if let Some(name) = name.to_str() {
        IGNORED_PATHS.contains(&name)
    } else {
        true
    }
}
