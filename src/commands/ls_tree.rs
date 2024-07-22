use crate::objects::{Object, ObjectHash, ObjectKind};
use crate::utils::from_bytes_with_nul;
use anyhow::{bail, Context};
use std::fmt::{Display, Formatter};
use std::io::prelude::*;

pub fn handle(object_hash: &str, name_only: bool) -> anyhow::Result<()> {
    let mut object = Object::read_from_objects(object_hash)?;

    match object.kind {
        ObjectKind::Tree => {
            display_tree(&mut object, name_only)?;
        }
        ObjectKind::Commit => {
            let mut buf = String::new();
            object
                .reader
                .read_line(&mut buf)
                .context("read first line in commit file")?;

            let Some((_, tree_hash)) = buf.split_once(' ') else {
                bail!("error: commit file signature is incorrect")
            };

            let mut object =
                Object::read_from_objects(tree_hash).context("read .git/objects tree object")?;
            display_tree(&mut object, name_only)?;
        }
        _ => println!("error: not a tree object"),
    }

    Ok(())
}

fn display_tree<R: BufRead>(object: &mut Object<R>, name_only: bool) -> anyhow::Result<()> {
    if name_only {
        display_name_only_tree(object)
    } else {
        display_full_tree(object)
    }
}

fn display_name_only_tree<R: BufRead>(object: &mut Object<R>) -> anyhow::Result<()> {
    while !object.reader.fill_buf()?.is_empty() {
        let name = TreeObjectItem::read_name(&mut object.reader)?;
        println!("{name}");
    }
    Ok(())
}

fn display_full_tree<R: BufRead>(object: &mut Object<R>) -> anyhow::Result<()> {
    while !object.reader.fill_buf()?.is_empty() {
        let item = TreeObjectItem::read(&mut object.reader)?;
        println!("{item}");
    }
    Ok(())
}

pub(crate) struct TreeObjectItem {
    mode: String,
    name: String,
    hash: ObjectHash,
    kind: ObjectKind,
}

impl TreeObjectItem {
    pub(crate) fn read(reader: &mut impl BufRead) -> anyhow::Result<TreeObjectItem> {
        let TreeObjectItemRaw { mode, name, hash } = TreeObjectItemRaw::read(reader)?;

        let hex_hash = hex::encode(&hash);
        let object = Object::read_from_objects(&hex_hash)
            .with_context(|| format!("read .git/objects file with hash {hex_hash}"))?;

        Ok(TreeObjectItem {
            mode,
            name,
            hash,
            kind: object.kind,
        })
    }

    pub(crate) fn read_name(reader: &mut impl BufRead) -> anyhow::Result<String> {
        let TreeObjectItemRaw { name, .. } = TreeObjectItemRaw::read(reader)?;
        Ok(name)
    }
}

impl Display for TreeObjectItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let TreeObjectItem {
            name,
            mode,
            hash,
            kind,
        } = self;
        let hash_hex = hex::encode(hash);
        write!(f, "{mode:0>6} {kind} {hash_hex}    {name}")
    }
}

struct TreeObjectItemRaw {
    mode: String,
    name: String,
    hash: ObjectHash,
}

impl TreeObjectItemRaw {
    fn read(reader: &mut impl BufRead) -> anyhow::Result<Self> {
        let mut head = Vec::new();
        reader
            .read_until(0x00, &mut head)
            .context(".git/objects read tree object item head")?;
        let head = from_bytes_with_nul(&head).context("parse tree object item head")?;

        let Some((mode, name)) = head.split_once(' ') else {
            bail!(".git/objects tree object item head signature is incorrect '{head}'")
        };

        let mut hash = [0; 20];
        reader
            .read_exact(&mut hash)
            .with_context(|| format!(".git/objects incorect hash signature {hash:?}"))?;

        Ok(Self {
            hash,
            name: name.to_owned(),
            mode: mode.to_owned(),
        })
    }
}
