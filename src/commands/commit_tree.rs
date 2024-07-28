use crate::objects::{Object, ObjectHash, ObjectKind};
use anyhow::{bail, Context};
use std::io::{Cursor, Write};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn handle(
    tree_hash: String,
    parent_hash: Option<String>,
    message: String,
) -> anyhow::Result<()> {
    let hash = write_commit(&tree_hash, parent_hash.as_deref(), &message)
        .context("create commit object")?;
    let hex_hash = hex::encode(hash);

    println!("{hex_hash}");

    Ok(())
}

pub fn write_commit(
    tree_hash: &str,
    parent_hash: Option<&str>,
    message: &str,
) -> anyhow::Result<ObjectHash> {
    if Object::read_from_objects(&tree_hash)?.kind != ObjectKind::Tree {
        bail!("error: provided hash is not associated with a tree object")
    }
    if let Some(parent_hash) = &parent_hash {
        if Object::read_from_objects(parent_hash)?.kind != ObjectKind::Commit {
            bail!("error: parent hash is not associated with a commit object")
        }
    }

    let mut buf = Vec::new();
    generate_commit_object(tree_hash, parent_hash, message, &mut buf)
        .context("create commit object content")?;

    let object = Object {
        kind: ObjectKind::Commit,
        size: buf.len() as u64,
        reader: Cursor::new(buf),
    };

    object
        .write_to_objects()
        .context("write .git/objects commit blob")
}

fn generate_commit_object(
    tree_hash: &str,
    parent_hash: Option<&str>,
    message: &str,
    mut buf: impl Write,
) -> anyhow::Result<()> {
    writeln!(buf, "tree {tree_hash}")?;

    if let Some(parent_hash) = parent_hash {
        writeln!(buf, "parent {parent_hash}")?;
    }

    let (author, email) = get_git_author();
    let current_time = SystemTime::now();
    let timestamp = current_time
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let timezone = "+0200";

    writeln!(buf, "author {author} <{email}> {timestamp} {timezone}",)?;
    writeln!(buf, "committer {author} <{email}> {timestamp} {timezone}",)?;
    writeln!(buf)?;
    writeln!(buf, "{message}")?;

    Ok(())
}

fn get_git_author() -> (String, String) {
    let author_name = "Dmitriy Popov";
    let author_email = "me@demestoss.com";
    (author_name.to_owned(), author_email.to_owned())
}
