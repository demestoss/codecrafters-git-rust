use crate::objects::{Object, ObjectKind};

pub fn handle(object_hash: &str, name_only: bool) -> anyhow::Result<()> {
    let object = Object::read(object_hash)?;

    match (object.kind, name_only) {
        (ObjectKind::Tree, true) => {}
        (ObjectKind::Tree, false) => {}
        _ => println!("error: not a tree object"),
    }

    Ok(())
}
