use crate::commands::ls_tree::TreeObjectItem;
use crate::objects::{Object, ObjectKind};
use anyhow::Context;
use std::io;
use std::io::BufRead;

pub struct CatObjectFlags {
    pub pretty_print: bool,
    pub object_exists: bool,
    pub object_type: bool,
    pub object_size: bool,
}

pub fn handle(object_hash: &str, flags: CatObjectFlags) -> anyhow::Result<()> {
    let mut object = Object::read_from_objects(object_hash)
        .with_context(|| format!("read .git/objects file with hash {object_hash}"))?;

    match flags {
        CatObjectFlags {
            pretty_print: true, ..
        } => display_object(&mut object)?,
        CatObjectFlags {
            object_exists: true,
            ..
        } => print!("object exists!"),
        CatObjectFlags {
            object_type: true, ..
        } => print!("{}", object.kind),
        CatObjectFlags {
            object_size: true, ..
        } => print!("{}", object.size),
        _ => println!("usage: git cat-file (-p | -e | -t | -s) <object>"),
    };
    Ok(())
}

fn display_object(object: &mut Object<impl BufRead>) -> anyhow::Result<()> {
    match object.kind {
        ObjectKind::Tree => {
            while !object.reader.fill_buf()?.is_empty() {
                let tree_object_item = TreeObjectItem::read(&mut object.reader)
                    .context("read tree object item line")?;
                println!("{tree_object_item}");
            }
        }
        ObjectKind::Blob | ObjectKind::Commit => {
            let stdout = io::stdout();
            let mut stdout = stdout.lock();
            io::copy(&mut object.reader, &mut stdout)
                .context("stream object content into stdout")?;
        }
    };
    Ok(())
}
